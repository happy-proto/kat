# Kat

[![build](https://github.com/happy-proto/kat/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/happy-proto/kat/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/happy-proto/kat/status.svg)](https://deps.rs/repo/github/happy-proto/kat)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/happy-proto/kat/blob/master/LICENSE)

`kat` 是一个基于 Tree-sitter 的高亮版 `cat`。

它的重点不是把更多文件“简单点亮”，而是尽量把那些依赖结构语义、宿主感知和嵌套分发的场景做深，比如：

- Rust 文档注释里的 Markdown
- `Justfile` 里的 Bash / Python / Zsh / Fish recipe
- 宿主语言里的 SQL / GraphQL / Regex
- 特殊文件名、shebang 和无扩展名内容识别

## 为什么有这个项目

我之前主要用 `bat`，但它在嵌套高亮这类场景上，尤其是 `Justfile` 支持，一直没有特别顺手。`bat` 底层使用的是 `syntect`，读取的是 Sublime Text 的 `.sublime-syntax`；这套模型可以处理一部分嵌套高亮，但对我想要的那种更深、更多依赖结构语义的嵌套高亮，似乎较难继续做深，所以我就开始自己写这个。

这个项目直接选了基于 Tree-sitter 的路线。现在默认优先使用 crate-backed parser，只在确有必要时保留最小 vendored grammar 资产；与此同时，query、detector 和 nested runtime 仍尽量放在仓库里。这样做不是为了追求某种“更标准”的架构，而是为了把更多中间层暴露出来，方便继续实验这类更依赖结构语义和嵌套分发的高亮场景。

这个项目实现过程中也大量参考了 `Zed` 的代码。它支持了很多语言的高亮，而且效果很好，所以在 query、嵌套语言处理和语言支持范围这些问题上，它一直是一个很有价值的参考对象。

## 当前项目侧重点

- 基于 Tree-sitter 做语法识别和高亮
- 统一处理 grammar、query、detector 和 nested runtime
- 优先把高收益语言和嵌套场景做深，而不是只追求“支持数量”
- 让仓库内约定直接描述语言注册、文件识别和构建行为

当前这个项目本质上仍然只是个人玩具：纯 vibe coding，完全没有 review。

## 快速开始

先安装仓库依赖并把本地二进制装起来：

```bash
just install
```

日常使用：

```bash
kat path/to/file
```

查看当前文件在 `kat` 内部各阶段的耗时：

```bash
kat --debug-timing --paging=never path/to/file >/dev/null
```

查看当前构建的版本与构建元信息：

```bash
kat --version
```

启用基于 `clap` 动态补全：

```bash
echo 'source <(COMPLETE=bash kat)' >> ~/.bashrc
echo 'source <(COMPLETE=zsh kat)' >> ~/.zshrc
echo 'COMPLETE=fish kat | source' >> ~/.config/fish/completions/kat.fish
```

如果只想本地编译调试：

```bash
cargo build
```

如果想直接安装当前仓库配置对应的预编译包，可以用：

```bash
cargo binstall --git https://github.com/happy-proto/kat --force kat
```

这条命令会读取仓库里的 `cargo-binstall` 元数据，并从 GitHub Releases 下载对应平台的预编译资产。

## 开发调试

- 提交前检查：`prek run --all-files`
- 跑测试：`just test`
- 跑仓库内性能基线：`just perf`，单文件性能基线可用 `just perf-file path/to/file`
- CI 与发布流程以仓库里的工作流配置为准
- 查看某门语言的 AST：`kat --debug-ast --language fish path/to/file`
- 查看 analysis 层输出的检测结果、styled spans 和 nested regions：`kat --debug-analysis path/to/file`
- 查看 semantic overlay 命中的结构语义：`kat --debug-semantics --language sql_postgres path/to/file`
- 查看 visual 层输出的 region / block / tint 分段：`kat --debug-visual path/to/file`
- 查看终端无关的渲染状态流：`kat --debug-render-ops path/to/file`
- 查看 terminal 层能力与最终 ANSI 编码：`kat --debug-terminal path/to/file`
- 查看渲染分段耗时：`kat --debug-timing --paging=never path/to/file >/dev/null`
- `--debug-shell-semantics` 仍保留为兼容别名，但现在输出的是通用 semantic overlay 结果
- `--debug-analysis`、`--debug-visual`、`--debug-render-ops`、`--debug-terminal` 默认输出稳定 JSON，适合做 snapshot、回归和跨环境 diff
- 仓库内自动化测试默认优先断言 `analysis` / `visual` / `render_ops` 这几层的稳定 IR；最终 ANSI / terminal 编码只保留最小必要的兼容与回归覆盖
- 长输出默认支持外部分页：`--paging=auto|always|never`，`auto` 会在 TTY 中按屏高判断是否接入 pager；pager 命令优先读 `PAGER`，未设置时默认回退到 `less -R -F -X`

## 文档入口

- 当前支持现状：[`docs/language-coverage.md`](docs/language-coverage.md)
- 架构说明：[`docs/architecture.md`](docs/architecture.md)
- 路线图：[`docs/roadmap.md`](docs/roadmap.md)
- 仓库维护约定：[`docs/maintenance.md`](docs/maintenance.md)
- vendored grammar 例外清单：[`docs/vendor-grammar-exceptions.md`](docs/vendor-grammar-exceptions.md)
- 测试样例约定：[`docs/test-assets.md`](docs/test-assets.md)

如果你最关心的是“现在到底支持哪些语言、做到什么程度”，先读 [`docs/language-coverage.md`](docs/language-coverage.md)。如果你想看这个仓库为什么会长成现在这样，再看 [`docs/architecture.md`](docs/architecture.md)。
