# nosync

An efficient, decoupled Hyperliquid wallet position monitor and Notion database logger.

## 1. Introduction
`nosync` is a real-time wallet position monitoring and analysis tool built with Rust. It subscribes to a specified Hyperliquid wallet address via WebSockets, captures position-opening trades in real-time, aggregates multi-tick partial fills of a single order (using a 500ms debounce aggregation mechanism), and automatically synchronizes the formatted trade data to a Notion database.

## 2. Key Features
* **Real-time Wallet Subscription**: Uses `hyperliquid-rust-sdk` to establish a persistent WebSocket connection to the wallet's `UserEvents`, sensing position changes at millisecond latency.
* **Aggregated Fills via `oid`**: Aggregates multi-tick partial fills belonging to the same order (`oid`) within a 500ms window into a single unified row, preventing split trades from generating duplicate database entries.
* **Opening Event Capture**: Identifies position opening actions by validating if the starting position (`start_position`) is zero. Currently captures only opening positions (ignores closes to prevent tick-splitting complexity).
* **Order Type Detection**: Auto-detects order execution types (`MARKET` for taker fills with `crossed == true`, `LIMIT` for maker fills with `crossed == false`).
* **Robust Reconnection**: Implements a robust connection loop that automatically reconnects after network glitches or WebSocket drops.
* **Notion Integration**: Integrates `notion-client` API to write structured rows (`Symbol`, `Quantity`, `Filled Price`, `Direction`, `Exchange`, `DataTime`, `Order ID`, `Order Type`, `Check`) directly to a target Notion database.

## 3. Architecture & Modules
The directory structure and modules are as follows:

```
nosync/
├── src/
│   ├── lib.rs          # Unified module exports
│   ├── structs.rs      # Data models and event definitions (e.g., NotionRowData)
│   ├── hyperliquid.rs  # Hyperliquid WebSocket monitor implementation
│   └── notion.rs       # Notion Database writer implementation
├── examples/           # Development examples
│   ├── demo.rs         # Real-time wallet monitoring demo
│   └── fetch_notion_db.rs # Dev script to fetch Notion Database schema properties
├── tests/              # Integration and unit tests
│   └── monitor_test.rs # Monitor connectivity and reconnection loop tests
├── references/         # Local reference submodules
├── .env                # Local environment variables configuration (ignored by git)
├── .env.example        # Environment configuration template
├── Cargo.toml          # Cargo package file with remote Git dependencies
└── CHANGELOG.md        # Change log
```

* **HyperliquidMonitor** (`src/hyperliquid.rs`): Long-running service maintaining the WebSocket connection to the Hyperliquid API and emitting aggregated trade events.
* **NotionWriter** (`src/notion.rs`): Formats and writes the captured row data to the Notion database.
* **structs** (`src/structs.rs`): Defines data models like `PositionTradeEvent` and `NotionRowData`.

## 4. Requirements
* **Rust**: `1.85.0` or higher (supports the 2024 edition)
* **OS**: Linux, macOS, Windows

## 5. Getting Started
```bash
# Clone the repository
git clone https://github.com/cathiefish/nosync.git
cd nosync

# Copy and configure environment variables
cp .env.example .env
# Edit the .env file with your WALLET_ADDRESS, Notion integration token, and database ID

# Build the project
cargo build --release

# Run the monitoring sync service
cargo run

# Run the local monitor demo
cargo run --example demo
```

## 6. Code Example
A minimal running example for the monitor is located at [examples/demo.rs](file:///home/cathiefish/App/nosync/examples/demo.rs):
```rust
use alloy::primitives::address;
use nosync::HyperliquidMonitor;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wallet = address!("0xc64cc00b46101bd40aa1c3121195e85c0b0918d8");
    let monitor = HyperliquidMonitor::new(wallet, true); // true for Testnet

    let (tx, mut rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let _ = monitor.run(tx).await;
    });

    while let Some(event) = rx.recv().await {
        println!("Captured Position Opening Event: {:?}", event);
    }
    Ok(())
}
```

## 7. Environment Variables
| Variable | Description | Required | Default / Example Value |
| :--- | :--- | :--- | :--- |
| `WALLET_ADDRESS` | The Ethereum/Hyperliquid wallet address to monitor | **Yes** | `0xc64cc00b46101bd40aa1c3121195e85c0b0918d8` |
| `IS_TESTNET` | Connect to Hyperliquid Testnet (`true`/`false`) | No | `true` |
| `NOTION_API_KEY` | Notion integration token (internal secret) | **Yes** | `secret_xxxxxx...` |
| `NOTION_DATABASE_ID` | Notion Database ID | **Yes** | `2b08f81ac37083389c5c01242f3c1557` |
| `RUST_LOG` | Logging verbosity level (error, warn, info, debug) | No | `info` |
| `ENABLE_SCREENSHOT` | Enable TradingView chart screenshots in Notion pages (`true`/`false`) | No | `false` |
| `TRADESNAP_URL` | The API endpoint URL of the TradeSnap service | No | `http://tradesnap:8003` |
| `BTCUSDT_SNAPSHOT` | Use Binance USDT perps (`true`) instead of USDC perps (`false`) for screenshots | No | `false` |
| `SYMBOL_15M_SNAPSHOT` | Capture and insert 15m interval screenshot (`true`/`false`) | No | `false` |
| `SYMBOL_1H_SNAPSHOT` | Capture and insert 1h interval screenshot (`true`/`false`) | No | `false` |
| `SYMBOL_4H_SNAPSHOT` | Capture and insert 4h interval screenshot (`true`/`false`) | No | `false` |
| `SYMBOL_1D_SNAPSHOT` | Capture and insert 1D interval screenshot (`true`/`false`) | No | `false` |


## 8. Development & Testing
### Git Branching Strategy
* `dev`: Active development branch. New features and fixes are merged here first.
* `main`: Stable production-ready branch.

### Local Quality Checks
Before submitting pull requests, you **must** run:
```bash
# Auto-format codebase
cargo fmt --all

# Run lint checks
cargo clippy --all-targets --all-features -- -D warnings

# Execute test suite
cargo test
```

## 9. Changelog
For a detailed log of project updates, please refer to [CHANGELOG.md](file:///home/cathiefish/App/nosync/CHANGELOG.md).
