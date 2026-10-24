# 使用指南

## 目录浏览

终端中直接执行 `kat` 等同于 `kat .`，从当前目录开始浏览，也可以用 `kat path/to/directory` 指定起点。界面只有左右两栏：左侧为当前目录列表，右侧为选中项的预览。目录优先，其余按文件名排序，包含隐藏文件；选中目录时预览它的直接子项，选中文本时复用 kat 的语法高亮和嵌套语言渲染。

左栏按当前目录文件名的实际显示宽度自适应，包含图标和间距；通常至少 16 列，最多 40 列，并且不超过终端宽度的三分之一。短文件名时自动收窄，长文件名不会持续挤占预览区；切换目录、刷新目录和窗口缩放后更新布局。

浏览器使用与源码高亮共享的 Dracula 色板：路径栏为紫色，状态栏为青色，分隔线与提示为 Comment 色，目录名与目录图标为紫色加粗；选中行、路径栏和状态栏使用 Current Line 背景。正文区域沿用终端背景。颜色能力与高亮共用同一次探测，支持 `KAT_COLOR_MODE=ansi`、`KAT_COLOR_MODE=none` 和 `NO_COLOR`；无彩色时仍保留选中标记和反色反馈。

左侧列表和右侧目录预览默认显示 Nerd Font 图标，按文件名、扩展名区分文件类型；Cargo 文件使用 Rust 图标，未知文件使用通用文件图标。图标映射由 `devicons` 依赖维护，其颜色统一映射到 Dracula 色板。终端需内置 Nerd Font 支持，或配置 `Symbols Nerd Font` fallback；字体显示方框时可以先使用 `kat --icons=never`，恢复纯文件名列表。`--icons=always` 为默认值；图标仅作用于目录列表，不改变文件或管道的内容输出。

| 按键 | 操作 |
| --- | --- |
| `j/k`、`↓/↑` | 选择下一项 / 上一项 |
| `l`、`→`、`Enter` | 在 kat 内进入目录，或全屏查看文件 |
| `h`、`←` | 返回上级目录，并选中刚离开的目录 |
| `o` | 文件进入全文查看；目录退出 kat 并切换 shell 目录（需 shell 集成） |
| `g/G`、Home/End | 选择第一项 / 最后一项 |
| PageUp/PageDown | 按列表可见高度翻页 |
| `R` | 重新读取目录和当前预览，尽量保留选中项 |
| `q`、Esc | 退出浏览；全文阅读中则返回目录 |
| `Ctrl+C` | 退出整个会话 |

全文阅读与目录浏览共用 Dracula 路径栏、状态栏和提示样式。顶部路径过长时省略中间部分，优先保留末尾的文件名。阅读界面保留下面的滚动、搜索和刷新操作；从目录进入后，普通模式下 `h`、`←`、`q` 或 Esc 返回目录，并保留选择和列表位置；直接打开文件时 `q` 退出程序。搜索输入中的 `h` 仍是搜索文字。图片在预览区内等比缩放，`l`、`→`、`o` 或 `Enter` 可全屏查看，`h`、`←` 或 `q` 返回；终端没有可用图片协议时显示图片信息。浏览模式下图片尺寸受当前区域约束，使用 contain 缩放；直接打开图片时仍使用原有图片选项。

空目录、读取失败和二进制文件会显示提示；设备、socket、FIFO 等特殊文件不会被读取。自动预览最多读取 8 MiB，超过限制时按 `Enter` 显式打开完整文件。窗口过小时显示提示，放大后恢复两栏。

目录浏览要求 stdin 和 stdout 都是终端，且一次只接受一个目录。`cat file | kat` 和 `kat -` 仍读取标准输入；文件的非交互输出与 debug 模式保持原有行为。

> **Breaking change** 交互终端中的无参数 `kat` 改为浏览当前目录；需要手工输入 stdin 时请使用 `kat -`。管道输入不受影响。

### 在退出后切换 shell 目录

fish 用户执行一次：

```bash
kat --install-shell-integration fish
```

这会安装自动加载的 `kat` 函数。之后直接运行 `kat`，选中目录按 `o`，退出后 shell 就位于该目录；普通 `q` 退出保留原来的工作目录。比如在项目根选中 `workers/` 后按 `o`，就回到该目录里的 shell 提示符。

Bash / Zsh 用户可执行 `kat --install-shell-integration bash` 或 `kat --install-shell-integration zsh`，再从 shell 配置中 source 命令输出的脚本路径。安装不会覆盖已有的自定义 `kat` 函数文件；`command kat` 可以绕过包装函数运行二进制。

切换目录由父 shell 的包装函数执行，终端已有的 shell 集成负责更新工作目录报告。OSC 7 本身不能替父 shell 执行 `cd`。三种 shell 都会保留 kat 的退出状态，并清理临时目录选择文件。

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

