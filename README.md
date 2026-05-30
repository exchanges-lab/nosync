# nosync

一款高效、解耦的 Hyperliquid 钱包仓位监听与 Notion 同步工具。

## 1. 项目简介
`nosync` 是一个基于 Rust 构建的实时钱包仓位监控与分析工具。它通过 WebSocket 订阅指定的 Hyperliquid 钱包地址事件，实时抓取开仓（及后续平仓）交易，并将相关交易数据自动同步整理到 Notion 对应的数据库中，帮助用户进行自动化交易跟踪与统计。

## 2. 核心功能
* **实时钱包订阅**：使用 `hyperliquid-rust-sdk` 长连接订阅钱包的 `UserEvents` 变动，毫秒级感知仓位变化。
* **开仓事件识别**：通过解析成交明细的 `start_position` 是否为零，精准捕捉首次开仓（Position Opening）动作。
* **断线自动重连**：内部包含 robust 的重连与容错机制，确保网络抖动或服务关闭重启后自动恢复订阅。
* **Notion 数据同步**：集成 `notion-client` API，将捕捉到的开仓事件格式化为对应的属性并写入 Notion 数据库中（Notion 数据库结构由后续接口配置定义）。

## 3. 架构与模块
本项目的目录结构与模块划分如下：

```
nosync/
├── src/
│   ├── lib.rs          # 统一模块导出
│   ├── structs.rs      # 数据模型与事件定义（如 PositionOpenEvent）
│   └── hyperliquid.rs  # Hyperliquid 监听器实现（包含 HyperliquidMonitor）
├── examples/           # 使用示例
│   └── demo.rs         # 实时钱包监听演示
├── tests/              # 集成测试
│   └── monitor_test.rs # 监听器连通性与重连循环测试
├── references/         # 本地参考源（submodules）
├── .env                # 本地运行环境变量配置文件 (不提交)
├── .env.example        # 环境变量配置模板
├── Cargo.toml          # 远程 Git 依赖及配置
└── CHANGELOG.md        # 变更日志
```

* **HyperliquidMonitor** (`hyperliquid`)：长周期运行服务，维护与 Hyperliquid API 的长连接，订阅 `UserEvents`。
* **PositionOpenEvent** (`structs`)：定义了捕获到的开仓事件明细，包括币种、买卖方向、成交价格、数量、时间戳等。

## 4. 环境要求
* **Rust**: `1.85.0` 或更高版本（支持最新 edition 2024）
* **OS**: Linux, macOS, Windows

## 5. 安装与启动
```bash
# 克隆仓库
git clone https://github.com/cathiefish/nosync.git
cd nosync

# 复制并配置环境变量
cp .env.example .env
# 编辑 .env 文件，填入您需要监控的 WALLET_ADDRESS 及网络类型

# 构建项目
cargo build --release

# 运行监控服务
cargo run

# 运行本地监听 Demo
cargo run --example demo
```

## 6. 使用示例
最小可运行示例位于 [examples/demo.rs](file:///home/cathiefish/App/nosync/examples/demo.rs)：
```rust
use alloy::primitives::address;
use nosync::HyperliquidMonitor;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wallet = address!("0xc64cc00b46101bd40aa1c3121195e85c0b0918d8");
    let monitor = HyperliquidMonitor::new(wallet, true); // true 代表使用 Testnet

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

## 7. 环境变量说明
| 环境变量名 | 用途 | 是否必填 | 默认值 / 示例值 |
| :--- | :--- | :--- | :--- |
| `WALLET_ADDRESS` | 需要监控的以太坊/Hyperliquid钱包地址 | **是** | `0xc64cc00b46101bd40aa1c3121195e85c0b0918d8` |
| `IS_TESTNET` | 是否是 Testnet 环境 (`true`/`false`) | 否 | `true` |
| `RUST_LOG` | 设定日志输出级别 (e.g. error, warn, info, debug) | 否 | `info` |

## 8. 测试与开发
### 开发分支约定
* `dev`：主开发分支，新功能与修复首发合并至此。
* `main`：生产稳定分支。

### 本地验证命令
在提交代码前，**必须**运行以下命令进行本地验证：
```bash
# 自动格式化代码
cargo fmt --all

# 运行代码规范检查
cargo clippy --all-targets --all-features -- -D warnings

# 执行集成测试
cargo test
```

## 9. 变更日志指引
关于项目的详细改动，请参阅 [CHANGELOG.md](file:///home/cathiefish/App/nosync/CHANGELOG.md)。
