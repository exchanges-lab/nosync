use anyhow::{Context, Result};
use dotenvy::dotenv;
use nosync::HyperliquidMonitor;
use std::env;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env if present
    let _ = dotenv();

    // Initialize tracing subscriber
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("failed to set global tracing subscriber: {e}"))?;

    info!("Starting Hyperliquid Monitor Service...");

    // Retrieve WALLET_ADDRESS from environment
    let wallet_str =
        env::var("WALLET_ADDRESS").context("Missing WALLET_ADDRESS in environment variables")?;
    let wallet = wallet_str
        .parse::<alloy::primitives::Address>()
        .map_err(|e| anyhow::anyhow!("Invalid WALLET_ADDRESS: {e}"))?;

    // Retrieve IS_TESTNET from environment, defaulting to true if not set
    let is_testnet_str = env::var("IS_TESTNET").unwrap_or_else(|_| "true".to_string());
    let is_testnet = is_testnet_str.parse::<bool>().unwrap_or(true);

    let monitor = HyperliquidMonitor::new(wallet, is_testnet);
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Spawn the monitor loop in the background
    tokio::spawn(async move {
        if let Err(e) = monitor.run(tx).await {
            error!("Monitor runtime error: {:?}", e);
        }
    });

    info!("Service running, waiting for wallet events...");
    while let Some(event) = rx.recv().await {
        info!(
            coin = %event.coin,
            side = %event.side,
            px = %event.px,
            sz = %event.sz,
            action = ?event.action,
            start_pos = %event.start_pos,
            end_pos = %event.end_pos,
            time = event.time,
            tid = event.tid,
            "Captured position trade event!"
        );
        // Notion database integration will be wired here in the future
    }

    Ok(())
}
