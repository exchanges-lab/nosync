use crate::structs::PositionOpenEvent;
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
        event_tx: UnboundedSender<PositionOpenEvent>,
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
                                // Check if start_position is 0, which indicates a new position is opening
                                let start_pos: f64 = fill.start_position.parse().unwrap_or(0.0);
                                if start_pos == 0.0 {
                                    info!(
                                        coin = %fill.coin,
                                        side = %fill.side,
                                        px = %fill.px,
                                        sz = %fill.sz,
                                        tid = fill.tid,
                                        "Position opening detected!"
                                    );
                                    let event = PositionOpenEvent {
                                        coin: fill.coin,
                                        side: fill.side,
                                        px: fill.px,
                                        sz: fill.sz,
                                        time: fill.time,
                                        tid: fill.tid,
                                    };
                                    if let Err(e) = event_tx.send(event) {
                                        error!(
                                            error = ?e,
                                            "Failed to send PositionOpenEvent through channel"
                                        );
                                    }
                                } else {
                                    debug!(
                                        coin = %fill.coin,
                                        start_pos = start_pos,
                                        "Ignore non-opening trade fill"
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
