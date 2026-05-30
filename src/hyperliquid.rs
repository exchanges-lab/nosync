use crate::structs::{PositionTradeEvent, TradeAction};
use alloy::primitives::Address;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription, UserData};
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};

/// Errors specific to the HyperliquidMonitor.
#[derive(Error, Debug)]
pub enum HyperliquidMonitorError {
    #[error("Failed to parse wallet address: {0}")]
    AddressParseError(String),
}

/// Monitor for Hyperliquid wallet position openings.
pub struct HyperliquidMonitor {
    wallet_address: Address,
    is_testnet: bool,
}

impl HyperliquidMonitor {
    /// Creates a new HyperliquidMonitor instance.
    pub fn new(wallet_address: Address, is_testnet: bool) -> Self {
        Self {
            wallet_address,
            is_testnet,
        }
    }

    /// Runs the monitor WebSocket subscription in a loop, reconnecting if disconnected.
    pub async fn run(
        &self,
        event_tx: UnboundedSender<PositionTradeEvent>,
    ) -> Result<(), HyperliquidMonitorError> {
        let base_url = if self.is_testnet {
            BaseUrl::Testnet
        } else {
            BaseUrl::Mainnet
        };

        info!(
            wallet = %self.wallet_address,
            is_testnet = self.is_testnet,
            "Starting Hyperliquid monitor"
        );

        loop {
            info!("Connecting to Hyperliquid InfoClient...");
            let mut info_client = match InfoClient::new(None, Some(base_url)).await {
                Ok(client) => client,
                Err(e) => {
                    error!(
                        error = ?e,
                        "Failed to connect to Hyperliquid InfoClient, retrying in 5 seconds..."
                    );
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let (ws_sender, mut ws_receiver) = tokio::sync::mpsc::unbounded_channel();

            info!(
                "Subscribing to UserEvents for wallet: {}",
                self.wallet_address
            );
            if let Err(e) = info_client
                .subscribe(
                    Subscription::UserEvents {
                        user: self.wallet_address,
                    },
                    ws_sender,
                )
                .await
            {
                error!(
                    error = ?e,
                    "Failed to subscribe to UserEvents, retrying in 5 seconds..."
                );
                sleep(Duration::from_secs(5)).await;
                continue;
            }

            info!("Successfully subscribed to UserEvents.");

            let (timeout_tx, mut timeout_rx) = tokio::sync::mpsc::unbounded_channel::<u64>();

            struct AggregationState {
                coin: String,
                side: String,
                accumulated_sz: f64,
                accumulated_px_sz: f64,
                time: u64,
                tid: u64,
                crossed: bool,
            }

            let mut active_opening_orders =
                std::collections::HashMap::<u64, AggregationState>::new();

            loop {
                tokio::select! {
                    msg = ws_receiver.recv() => {
                        match msg {
                            Some(Message::User(user_msg)) => {
                                debug!(user_msg = ?user_msg, "Received user event message");
                                if let UserData::Fills(fills) = user_msg.data {
                                    for fill in fills {
                                        let start_pos: f64 = fill.start_position.parse().unwrap_or(0.0);
                                        let sz: f64 = fill.sz.parse().unwrap_or(0.0);
                                        let px: f64 = fill.px.parse().unwrap_or(0.0);

                                        // If it's a new open order or is part of an active opening order
                                        if start_pos == 0.0 || active_opening_orders.contains_key(&fill.oid) {
                                            match active_opening_orders.entry(fill.oid) {
                                                std::collections::hash_map::Entry::Vacant(e) => {
                                                    // First fill of the opening order: create entry and spawn timeout
                                                    e.insert(AggregationState {
                                                        coin: fill.coin.clone(),
                                                        side: fill.side.clone(),
                                                        accumulated_sz: sz,
                                                        accumulated_px_sz: px * sz,
                                                        time: fill.time,
                                                        tid: fill.tid,
                                                        crossed: fill.crossed,
                                                    });
                                                    info!(
                                                        coin = %fill.coin,
                                                        side = %fill.side,
                                                        px = %fill.px,
                                                        sz = %fill.sz,
                                                        oid = fill.oid,
                                                        "New opening order detected, starting aggregation..."
                                                    );

                                                    let tx = timeout_tx.clone();
                                                    let oid = fill.oid;
                                                    tokio::spawn(async move {
                                                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                                        let _ = tx.send(oid);
                                                    });
                                                }
                                                std::collections::hash_map::Entry::Occupied(mut e) => {
                                                    // Subsequent fill of the active opening order: aggregate it
                                                    let state = e.get_mut();
                                                    state.accumulated_sz += sz;
                                                    state.accumulated_px_sz += px * sz;
                                                    info!(
                                                        coin = %fill.coin,
                                                        side = %fill.side,
                                                        px = %fill.px,
                                                        sz = %fill.sz,
                                                        oid = fill.oid,
                                                        accumulated_sz = %state.accumulated_sz,
                                                        "Aggregated additional fill for active opening order"
                                                    );
                                                }
                                            }
                                        } else {
                                            debug!(
                                                coin = %fill.coin,
                                                start_pos = start_pos,
                                                oid = fill.oid,
                                                "Ignoring non-opening trade fill"
                                            );
                                        }
                                    }
                                }
                            }
                            Some(Message::HyperliquidError(err_msg)) => {
                                error!(error = %err_msg, "Received error message from Hyperliquid WS");
                            }
                            Some(Message::Pong) => {
                                debug!("Received Pong from Hyperliquid WS");
                            }
                            Some(other) => {
                                debug!(msg = ?other, "Received other message from Hyperliquid WS");
                            }
                            None => {
                                warn!("Hyperliquid WS receiver closed");
                                break;
                            }
                        }
                    }
                    oid = timeout_rx.recv() => {
                        match oid {
                            Some(oid) => {
                                if let Some(state) = active_opening_orders.remove(&oid).filter(|s| s.accumulated_sz > 0.0) {
                                    let avg_px = state.accumulated_px_sz / state.accumulated_sz;
                                    info!(
                                        coin = %state.coin,
                                        side = %state.side,
                                        avg_px = %avg_px,
                                        total_sz = %state.accumulated_sz,
                                        oid = oid,
                                        "Aggregated opening order completed! Sending event..."
                                    );

                                    let event = PositionTradeEvent {
                                        coin: state.coin,
                                        side: state.side,
                                        px: format!("{:.5}", avg_px),
                                        sz: format!("{:.5}", state.accumulated_sz),
                                        time: state.time,
                                        tid: state.tid,
                                        oid,
                                        action: TradeAction::Open,
                                        start_pos: "0.0".to_string(),
                                        end_pos: format!("{:.5}", state.accumulated_sz),
                                        crossed: state.crossed,
                                    };

                                    if let Err(e) = event_tx.send(event) {
                                        error!(
                                            error = ?e,
                                            "Failed to send PositionTradeEvent through channel"
                                        );
                                    }
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
            }

            warn!("Hyperliquid WebSocket connection closed. Reconnecting in 5 seconds...");
            sleep(Duration::from_secs(5)).await;
        }
    }
}
