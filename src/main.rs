use anyhow::{Context, Result};
use dotenvy::dotenv;
use nosync::{ModuleA, ModuleB};
use std::env;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env if present
    let _ = dotenv();

    // Initialize tracing subscriber with settings from environment (RUST_LOG)
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("failed to set global tracing subscriber: {e}"))?;

    info!("Starting nosync application...");

    // Retrieve APP_NAME from environment variables, defaulting if not found
    let app_name = env::var("APP_NAME").unwrap_or_else(|_| "nosync-default".to_string());
    info!(app_name = %app_name, "Environment configured");

    // Initialize ModuleA and ModuleB
    let module_a = ModuleA::new(app_name).context("Failed to initialize ModuleA")?;
    let module_b = ModuleB::new(module_a).context("Failed to initialize ModuleB")?;

    // Run ModuleB logic
    module_b
        .run()
        .await
        .context("Error occurred during execution")?;

    info!("nosync application finished successfully!");
    Ok(())
}
