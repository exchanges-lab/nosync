use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use dotenvy::dotenv;
use nosync::{HyperliquidMonitor, NotionRowData, NotionWriter};
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

    info!("Starting Hyperliquid to Notion Sync Service...");

    // Retrieve WALLET_ADDRESS and IS_TESTNET from environment
    let wallet_str =
        env::var("WALLET_ADDRESS").context("Missing WALLET_ADDRESS in environment variables")?;
    let wallet = wallet_str
        .parse::<alloy::primitives::Address>()
        .map_err(|e| anyhow::anyhow!("Invalid WALLET_ADDRESS: {e}"))?;

    let is_testnet_str = env::var("IS_TESTNET").unwrap_or_else(|_| "true".to_string());
    let is_testnet = is_testnet_str.parse::<bool>().unwrap_or(true);

    // Retrieve Notion configurations from environment
    let notion_token =
        env::var("NOTION_API_KEY").context("Missing NOTION_API_KEY in environment variables")?;
    let notion_database_id = env::var("NOTION_DATABASE_ID")
        .context("Missing NOTION_DATABASE_ID in environment variables")?;

    // Initialize monitor and writer
    let monitor = HyperliquidMonitor::new(wallet, is_testnet);
    let writer = NotionWriter::new(notion_token, notion_database_id)
        .context("Failed to initialize NotionWriter")?;

    let (tx, mut rx) = mpsc::unbounded_channel();

    // Spawn the monitor loop in the background
    tokio::spawn(async move {
        if let Err(e) = monitor.run(tx).await {
            error!("Monitor runtime error: {:?}", e);
        }
    });

    info!("Service running. Monitoring position openings and writing to Notion...");
    while let Some(event) = rx.recv().await {
        info!(
            coin = %event.coin,
            side = %event.side,
            px = %event.px,
            sz = %event.sz,
            oid = event.oid,
            "Captured position open event, writing to Notion..."
        );

        // Format Date/Time to "YYYY/MM/DD HH:MM"
        let dt = Utc
            .timestamp_millis_opt(event.time as i64)
            .single()
            .unwrap_or_else(Utc::now);
        let date_time_str = dt.format("%Y/%m/%d %H:%M").to_string();

        // Map Symbol to appends USDC
        let symbol_formatted = format!("{}USDC", event.coin);

        // Map Direction: B -> Long, S -> Short
        let direction_str = if event.side == "B" {
            "Long".to_string()
        } else {
            "Short".to_string()
        };

        // Construct Notion Row Data
        let row = NotionRowData {
            symbol: symbol_formatted,
            quantity: event.sz,
            filled_price: event.px,
            direction: direction_str,
            exchange: "Hyperliquid".to_string(), // Can be dynamically set or config-driven in the future
            date_time: date_time_str,
            time: event.time,
            order_id: event.oid,
            check: false,
        };

        // Write row to Notion
        if let Err(e) = writer.write_row(row).await {
            error!("Failed to write row to Notion: {:?}", e);
        }
    }

    Ok(())
}
