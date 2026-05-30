# nosync

一款高效、解耦的异步消息处理与任务编排框架。

## 1. 项目简介
`nosync` 是一个高性能的 Rust 异步消息处理和组件协作框架，专为解决多模块间松耦合通信、结构化日志追踪以及鲁棒的错误恢复机制而设计。它主要面向需要高并发、低延迟以及强模块化设计的服务端开发人员，为构建复杂微服务或本地计算引擎提供核心底座。

## 2. 核心功能
* **模块化封装架构**：遵循面向对象设计原则，将组件封装为高内聚的 `pub struct` 并持有独立状态，杜绝全局可变状态与裸函数。
* **异步事件处理**：全面支持基于 `tokio` 运行时的多任务处理，提升高负载场景下的吞吐量。
* **结构化日志监控**：集成 `tracing` 系统，提供细粒度的业务追踪与故障还原能力。
* **强类型错误管理**：利用 `thiserror` 定义清晰的组件级错误，拒绝吞掉异常，确保系统的健壮性。
* **多环境适配能力**：天然支持通过 `.env` 文件和环境变量在运行时动态配置系统属性。

## 3. 架构与模块
本项目的目录结构与模块划分如下：

```
nosync/
├── src/
│   ├── lib.rs          # 库入口，统一导出公共接口与类型
│   ├── structs.rs      # 存放跨模块共享的纯数据结构
│   ├── module_a.rs     # 业务模块 A (消息接收与底层处理)
│   └── module_b.rs     # 业务模块 B (工作流编排与核心控制)
├── examples/           # 使用示例
│   └── demo.rs         # 核心运行演示
├── tests/              # 集成测试
│   ├── module_a_test.rs
│   └── module_b_test.rs
├── references/         # Git 子模块与外部参考仓库目录
├── .env                # 实际运行环境变量配置文件 (本地开发，不提交)
├── .env.example        # 环境变量配置模板
├── Cargo.toml          # Cargo 配置文件
└── CHANGELOG.md        # 变更日志
```

* **ModuleA** (`ModuleA`)：负责消息接收、数据有效性校验与核心的异步解析处理。
* **ModuleB** (`ModuleB`)：负责编排 `ModuleA` 的执行流，充当协调器（Orchestrator）角色。
* **Structs** (`structs`)：定义了消息体 `SharedMessage` 等公共数据契约。

## 4. 环境要求
* **Rust**: `1.85.0` 或更高版本（支持最新 edition 2024）
* **OS**: Linux, macOS, Windows
* **运行时**: `tokio` (Full features)

## 5. 安装与启动
```bash
# 克隆仓库
git clone https://github.com/cathiefish/nosync.git
cd nosync

# 复制并配置环境变量
cp .env.example .env
# 可以根据需要修改 .env 中的内容

# 构建项目
cargo build --release

# 运行默认二进制应用
cargo run

# 运行使用示例
cargo run --example demo
```

## 6. 使用示例
最小可运行示例位于 [examples/demo.rs](file:///home/cathiefish/App/nosync/examples/demo.rs)：
```rust
use anyhow::Result;
use nosync::{ModuleA, ModuleB};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new("info"))
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // 初始化核心组件
    let processor = ModuleA::new("example-processor".to_string())?;
    let orchestrator = ModuleB::new(processor)?;

    // 运行工作流
    orchestrator.run().await?;
    Ok(())
}
```

## 7. 环境变量说明
| 环境变量名 | 用途 | 是否必填 | 默认值 / 示例值 |
| :--- | :--- | :--- | :--- |
| `APP_NAME` | 应用程序或当前节点的名称标识，用于日志和初始化 | 否 | `nosync-default` |
| `RUST_LOG` | 设定日志输出级别 (e.g. error, warn, info, debug, trace) | 否 | `info` |

## 8. 测试与开发
### 开发分支约定
* `dev`：主开发分支，新功能与修复首发合并至此。
* `main`：生产稳定分支，当且仅当测试、Clippy 与格式化全部通过后才合并。

### 本地验证命令
在提交代码前，**必须**运行以下命令进行本地验证：
```bash
# 自动格式化代码
cargo fmt --all

# 运行代码规范检查（不能有 warnings）
cargo clippy --all-targets --all-features -- -D warnings

# 执行单元测试与集成测试
cargo test
```

## 9. 变更日志指引
关于项目的历史演进和每个版本的详细改动，请参阅 [CHANGELOG.md](file:///home/cathiefish/App/nosync/CHANGELOG.md)。
