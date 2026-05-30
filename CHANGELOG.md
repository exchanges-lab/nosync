# Changelog

本文件记录项目所有值得注意的变更。
格式遵循 Keep a Changelog，版本号遵循 SemVer。

## [Unreleased]
### Added
- 添加外部参考库作为 Git 子模块：`noc` (notion-client) 与 `hype` (hyperliquid-rust-sdk)

## [0.1.0] - 2026-05-30
### Added
- 初始化 Rust 工程项目结构与核心配置
- 实现 `ModuleA` 与 `ModuleB` 业务类及接口设计
- 配置 `tokio` 异步运行时、`thiserror` 自定义错误与 `anyhow` 二进制入口错误处理
- 集成 `tracing` 结构化日志与 `dotenvy` 环境配置读取
- 新增集成测试与示例代码
