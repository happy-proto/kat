# 使用指南

## 文本与图片

日常使用：

```bash
kat path/to/file
```

直接在支持 inline image 的终端里展示图片：

```bash
kat path/to/image.png
```

当前会自动选择 iTerm2 inline image、Kitty graphics 或 Sixel；必要时可用 `KAT_IMAGE_PROTOCOL=iterm2|kitty|sixel|auto|none` 覆盖。
当前图片识别覆盖 PNG、JPEG、GIF、WebP、TIFF、BMP、ICO、QOI、PNM、Farbfeld、HDR、DDS、SVG 和 AVIF。
图片默认会按当前终端宽度和约 80% 终端高度做等比缩放；可用 `--image-width`、`--image-height` 或 `--image-fit=original` 覆盖。
透明图片默认保留透明通道；如果终端协议对透明支持不理想，可用 `--image-background=black|white|checker` 先合成背景。
如果 stdout 不是 TTY，或者当前终端没有可用图片协议，`kat` 会输出图片尺寸、格式和原因说明；可用 `--debug-image` 查看图片检测、EXIF orientation、目标尺寸和协议选择的 JSON。

## 阅读与搜索

TTY 下的普通渲染输出统一进入内建 alternate-screen viewer，从文件开头开始显示，并按终端宽度展开后的 display row 滚动；窗口大小变化时会按新宽度重新布局源码，同时尽量保持顶部源位置不变。viewer 支持 `j/k`、方向键、`Space`/`b` 与 PageUp/PageDown 整页滚动、`Ctrl+D`/`Ctrl+U` 半页滚动、Home/End、`/` 普通文本搜索、`n/N` 前后匹配和 `q` 退出。`kat` 不再启动外部 `PAGER`，也不再提供 `--paging` 开关。stdout 不是 TTY 时仍直接输出完整内容，适合 pipe/redirect。

viewer 普通模式下按 `R` 可重新读取所有打开的文本文件。状态栏会区分 `Reloaded: changed`（内容有变化）、`Reloaded: unchanged`（内容未变）和 `Reload failed`（失败）；提示显示约 3 秒后自动恢复正常状态栏，导航或搜索操作也可提前清除提示。刷新会尽量保持阅读位置并重建搜索匹配；任一文件读取失败时保留整份旧内容，可再次按 `R` 重试。stdin 内容保留首次读取的快照，不会重新读取。

在支持 OSC 8 的终端里，Markdown URL 这类有明确 URI 语义的片段默认会输出可点击超链接。可用 `--hyperlinks=auto|always|never` 或 `KAT_HYPERLINKS=auto|always|never` 覆盖；检测不到语言而走纯文本透传时不会额外注入超链接。

## 调试与版本

查看当前文件在 `kat` 内部各阶段的耗时：

```bash
kat --debug-timing path/to/file >/dev/null
```

查看当前构建的版本与构建元信息：

```bash
kat --version
```

## Shell 补全

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

