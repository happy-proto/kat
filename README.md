# Kat

[![build](https://github.com/happy-proto/kat/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/happy-proto/kat/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/happy-proto/kat/status.svg)](https://deps.rs/repo/github/happy-proto/kat)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/happy-proto/kat/blob/master/LICENSE)

`kat` 是一个基于 Tree-sitter 的高亮版 `cat`，让终端里的源码更容易读。

它最在意的是嵌套语言：Rust 文档注释里的 Markdown、Markdown 代码块里的 Mermaid、Justfile 和 mise / hk 配置里的脚本，都能按各自的语言高亮。也支持直接查看图片。

这个项目源于我使用 [bat](https://github.com/sharkdp/bat) 时对嵌套高亮的需求，实现中参考了 [Zed](https://github.com/zed-industries/zed) 的语言支持。它是一个个人玩具，完全 vibe Coding，基本不 Review。

## 安装

推荐使用 [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) 下载预编译二进制：

```bash
cargo binstall --git https://github.com/happy-proto/kat kat
```

也可以[从源码安装](docs/maintenance.md#开发环境)。macOS 仅支持 Apple Silicon。

源码构建与预编译包统一使用 kat-parsers 的预生成压缩语法表，无需额外启用压缩 feature 或预处理依赖源码。

## 使用

```bash
kat path/to/file
kat path/to/image.png
```

在终端中打开文本会进入内建 viewer，支持滚动、搜索和刷新；`q` 退出。通过管道或重定向则输出完整内容。

## 继续了解

- [GitHub Actions 依赖锁定](docs/maintenance.md#github-actions-依赖)
- [CI 合并门槛](docs/maintenance.md#ci-合并门槛)：PR 验证汇总为一个 required check，完整发布构建在默认分支执行，过期构建不覆盖 `latest`
- [本地提交检查](docs/maintenance.md#提交前检查)：格式、Clippy 与基础检查；完整测试使用 `mise run test`
- [使用指南](docs/usage.md)：viewer 操作、图片选项和 shell 补全
- [语言支持](docs/language-coverage.md)：支持范围与检测边界
- [开发与维护](docs/maintenance.md)：mise 工具与任务、检查和源码安装
- [架构](docs/architecture.md) · [路线图](docs/roadmap.md) · [测试样例](docs/test-assets.md)
