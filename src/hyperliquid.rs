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

            while let Some(msg) = ws_receiver.recv().await {
                match msg {
                    Message::User(user_msg) => {
                        debug!(user_msg = ?user_msg, "Received user event message");
                        if let UserData::Fills(fills) = user_msg.data {
                            for fill in fills {
                                // Parse position details
                                let start_pos: f64 = fill.start_position.parse().unwrap_or(0.0);
                                let sz: f64 = fill.sz.parse().unwrap_or(0.0);
                                let dir = if fill.side == "B" { 1.0 } else { -1.0 };
                                let change = dir * sz;
                                let end_pos = start_pos + change;

                                // Determine the trade action
                                let action = if start_pos == 0.0 {
                                    TradeAction::Open
                                } else if end_pos == 0.0 {
                                    TradeAction::Close
                                } else if start_pos.signum() != end_pos.signum() {
                                    TradeAction::Decrease
                                } else if end_pos.abs() > start_pos.abs() {
                                    TradeAction::Increase
                                } else {
                                    TradeAction::Decrease
                                };

                                info!(
                                    coin = %fill.coin,
                                    side = %fill.side,
                                    px = %fill.px,
                                    sz = %fill.sz,
                                    action = ?action,
                                    start_pos = start_pos,
                                    end_pos = end_pos,
                                    tid = fill.tid,
                                    "Wallet trade fill detected!"
                                );

                                let event = PositionTradeEvent {
                                    coin: fill.coin,
                                    side: fill.side,
                                    px: fill.px,
                                    sz: fill.sz,
                                    time: fill.time,
                                    tid: fill.tid,
                                    action,
                                    start_pos: fill.start_position.clone(),
                                    end_pos: format!("{:.5}", end_pos),
                                };

                                if let Err(e) = event_tx.send(event) {
                                    error!(
                                        error = ?e,
                                        "Failed to send PositionTradeEvent through channel"
                                    );
                                }
                            }
                        }
                    }
                    Message::HyperliquidError(err_msg) => {
                        error!(error = %err_msg, "Received error message from Hyperliquid WS");
                    }
                    Message::Pong => {
                        debug!("Received Pong from Hyperliquid WS");
                    }
                    other => {
                        debug!(msg = ?other, "Received other message from Hyperliquid WS");
                    }
                }
            }

            warn!("Hyperliquid WebSocket connection closed. Reconnecting in 5 seconds...");
            sleep(Duration::from_secs(5)).await;
        }
    }
}
