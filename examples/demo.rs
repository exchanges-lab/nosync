use anyhow::Result;
use nosync::{ModuleA, ModuleB};
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure simple standard output logging
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new("info"))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("failed to set global tracing subscriber: {e}"))?;

    info!("--- Running nosync library example ---");

    // Construct the structs and execute the logic
    let processor = ModuleA::new("example-processor".to_string())?;
    let orchestrator = ModuleB::new(processor)?;

    orchestrator.run().await?;

    info!("--- Example execution completed successfully ---");
    Ok(())
}
