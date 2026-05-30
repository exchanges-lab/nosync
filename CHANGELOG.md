# Changelog

本文件记录项目所有值得注意的变更。
格式遵循 Keep a Changelog，版本号遵循 SemVer。

## [Unreleased]
### Added
- 实现 `NotionWriter` 模块，支持将格式化交易记录批量写入 Notion 数据库。
- 实现 `HyperliquidMonitor` 针对同一订单（`oid`）在 500 毫秒内的多成交 tick 聚合功能，防止拆单造成多笔重复写入。
- 定义 `NotionRowData` 与 `TradeAction` 结构，提供结构化的 Notion 写入数据模型。
- 新增 `examples/fetch_notion_db.rs` 用于实时抓取 Notion Database 列配置的开发脚本。
- 实现 `HyperliquidMonitor` 用于实时订阅以太坊/Hyperliquid钱包账户的交易与开仓事件。
- 添加 `alloy`、`chrono` 与 `serde_json` 库依赖。
- 新增 integration test `tests/monitor_test.rs` 与使用示例 `examples/demo.rs`。
- 添加外部参考库作为 Git 子模块：`noc` (notion-client) 与 `hype` (hyperliquid-rust-sdk)。

### Changed
- 重构 `src/main.rs`，实现从 Hyperliquid WebSocket 捕获开仓事件并自动格式化写入 Notion Database 的闭环服务。
- 将 `hyperliquid_rust_sdk` 与 `notion-client` 依赖方式更改为指向官方 Git 仓库的远程依赖，移除本地 Path 依赖。

### Removed
- 移除初始化模版中未使用的占位模块 (`ModuleA`, `ModuleB`) 及其对应测试文件。

## [0.1.0] - 2026-05-30
### Added
- 初始化 Rust 工程项目结构与核心配置
- 实现 `ModuleA` 与 `ModuleB` 业务类及接口设计
- 配置 `tokio` 异步运行时、`thiserror` 自定义错误与 `anyhow` 二进制入口错误处理
- 集成 `tracing` 结构化日志与 `dotenvy` 环境配置读取
- 新增集成测试与示例代码
