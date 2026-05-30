# Changelog

本文件记录项目所有值得注意的变更。
格式遵循 Keep a Changelog，版本号遵循 SemVer。

## [Unreleased]
### Added
- 实现 `HyperliquidMonitor` 用于实时订阅以太坊/Hyperliquid钱包账户的交易与开仓事件。
- 新增 `PositionOpenEvent` 数据结构，当检测到持仓从0变动为非0时触发。
- 添加 `alloy` 库依赖，用于强类型地址解析。
- 新增 integration test `tests/monitor_test.rs` 与使用示例 `examples/demo.rs`。
- 添加外部参考库作为 Git 子模块：`noc` (notion-client) 与 `hype` (hyperliquid-rust-sdk)。

### Changed
- 将 `hyperliquid_rust_sdk` 与 `notion-client` 依赖方式更改为指向官方 Git 仓库的远程依赖，移除本地 Path 依赖。
- 重构 `src/main.rs` 以使用 `HyperliquidMonitor` 并通过 `.env` 环境变量加载钱包与测试网选项。

### Removed
- 移除初始化模版中未使用的占位模块 (`ModuleA`, `ModuleB`) 及其对应测试文件。

## [0.1.0] - 2026-05-30
### Added
- 初始化 Rust 工程项目结构与核心配置
- 实现 `ModuleA` 与 `ModuleB` 业务类及接口设计
- 配置 `tokio` 异步运行时、`thiserror` 自定义错误与 `anyhow` 二进制入口错误处理
- 集成 `tracing` 结构化日志与 `dotenvy` 环境配置读取
- 新增集成测试与示例代码
