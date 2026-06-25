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

我之前主要用 [bat](https://github.com/sharkdp/bat)，但它在嵌套高亮这类场景上，尤其是 `Justfile` 支持，一直没有特别顺手。`bat` 底层使用的是 `syntect`，读取的是 Sublime Text 的 `.sublime-syntax`；这套模型可以处理一部分嵌套高亮，但对我想要的那种更深、更多依赖结构语义的嵌套高亮，似乎较难继续做深，所以我就开始自己写这个。

这个项目直接选了基于 Tree-sitter 的路线。原因也很直接：我更想把这类依赖结构语义、宿主感知和嵌套分发的高亮场景继续做深，而不是停留在较浅的规则拼接上。

这个项目实现过程中也大量参考了 [Zed](https://github.com/zed-industries/zed) 的代码。它支持了很多语言的高亮，而且效果很好，所以在 query、嵌套语言处理和语言支持范围这些问题上，它一直是一个很有价值的参考对象。

这个项目本质上就是个人玩具：完全 vibe Coding，基本不 Review。

## 安装

推荐优先使用 [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) 基于当前仓库安装，它会直接下载对应平台的预编译二进制，避免本地编译 `kat`：

```bash
cargo binstall --git https://github.com/happy-proto/kat kat
```

如果你想基于同一个仓库源码直接本地编译安装，也可以用：

```bash
cargo install --git https://github.com/happy-proto/kat kat
```

## 快速开始

日常使用：

```bash
kat path/to/file
```

直接在支持 inline image 的终端里展示图片：

```bash
kat path/to/image.png
```

当前会自动选择 iTerm2 inline image、Kitty graphics 或 Sixel；必要时可用 `KAT_IMAGE_PROTOCOL=iterm2|kitty|sixel|auto|none` 覆盖。
当前图片识别覆盖 PNG、JPEG、GIF、WebP、TIFF、BMP、ICO、QOI、PNM、Farbfeld、HDR 和 DDS。
图片默认会按当前终端宽度和约 80% 终端高度做等比缩放；可用 `--image-width`、`--image-height` 或 `--image-fit=original` 覆盖。
透明图片默认保留透明通道；如果终端协议对透明支持不理想，可用 `--image-background=black|white|checker` 先合成背景。
如果 stdout 不是 TTY，或者当前终端没有可用图片协议，`kat` 会输出图片尺寸、格式和原因说明；可用 `--debug-image` 查看图片检测、EXIF orientation、目标尺寸和协议选择的 JSON。

查看当前文件在 `kat` 内部各阶段的耗时：

```bash
kat --debug-timing path/to/file >/dev/null
```

TTY 下的普通渲染输出统一进入内建 alternate-screen viewer，从文件开头开始显示，并支持 `j/k`、方向键、PageUp/PageDown、Home/End 和 `q` 退出；`kat` 不再启动外部 `PAGER`，也不再提供 `--paging` 开关。stdout 不是 TTY 时仍直接输出完整内容，适合 pipe/redirect。

在支持 OSC 8 的终端里，Markdown URL 这类有明确 URI 语义的片段默认会输出可点击超链接。可用 `--hyperlinks=auto|always|never` 或 `KAT_HYPERLINKS=auto|always|never` 覆盖；检测不到语言而走纯文本透传时不会额外注入超链接。

查看当前构建的版本与构建元信息：

```bash
kat --version
```

安装 shell completion 到用户目录：

```bash
kat --install-completion bash
kat --install-completion fish
kat --install-completion zsh
```

默认安装位置（若设置了 XDG 目录变量，会优先使用对应 XDG 路径）：

- `bash`：`~/.local/share/bash-completion/completions/kat`
- `fish`：`~/.config/fish/completions/kat.fish`
- `zsh`：`~/.local/share/zsh/site-functions/_kat`

如果你更想手动启用，也可以继续直接 source 动态补全脚本：

```bash
echo 'source <(COMPLETE=bash kat)' >> ~/.bashrc
echo 'source <(COMPLETE=zsh kat)' >> ~/.zshrc
echo 'COMPLETE=fish kat | source' >> ~/.config/fish/completions/kat.fish
```

## 文档入口

- 当前支持现状：[`docs/language-coverage.md`](docs/language-coverage.md)
- 架构说明：[`docs/architecture.md`](docs/architecture.md)
- 路线图：[`docs/roadmap.md`](docs/roadmap.md)
- 仓库维护约定：[`docs/maintenance.md`](docs/maintenance.md)
- 主仓库 vendored parser 现状：[`docs/vendor-grammar-exceptions.md`](docs/vendor-grammar-exceptions.md)
- 测试样例约定：[`docs/test-assets.md`](docs/test-assets.md)
- 面向 Agent 的开发调试入口：[SKILL.md](.agents/skills/kat-dev/SKILL.md)

如果你最关心的是“现在到底支持哪些语言、做到什么程度”，先读 [`docs/language-coverage.md`](docs/language-coverage.md)。如果你想看这个仓库为什么会长成现在这样，再看 [`docs/architecture.md`](docs/architecture.md)。
