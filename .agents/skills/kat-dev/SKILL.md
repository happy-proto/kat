---
name: kat-dev
description: 介绍 kat 当前架构分层，并按层整理调试命令与其他常用开发命令；当需要修改 kat、排查高亮链路或理解 analysis/visual/render_ops/terminal 关系时使用。
---

先按 `analysis -> visual -> layout -> render_ops -> terminal` 这条链理解 `kat`。更完整的稳定结论见 [architecture.md](../../../docs/architecture.md)。

## 架构分层

- `analysis`：负责 document kind 检测、基础高亮、semantic overlay 和 injection region 收集。
- `visual`：负责把 analysis 结果整理成稳定视觉模型，包括 styled spans、nested region 的视觉分段和 block 成员关系；不在这一层固化 viewport 相关宽高。
- `layout`：先按当前 viewport 生成 wrapped screen rows，再计算 block bbox 和 display-space 背景范围。`RectBlock` 的宽度取成员视觉行最大 display width，高度覆盖全部成员视觉行；短行只补到 block 右边界，不无条件铺满 viewport。
- `render_ops`：负责把视觉模型编译成终端无关的渲染状态流。
- `terminal`：负责终端能力探测、分页接入和最终 ANSI 编码输出。

## 按层调试命令

- `analysis`：`kat --debug-analysis path/to/file`。用来看 document kind、nested regions、runtime identity，以及 analysis 层稳定 JSON。
- `semantic overlay`：`kat --debug-semantics --language sql_postgres path/to/file`。用来看 query 之外补上的结构语义；`--debug-shell-semantics` 是兼容别名。
- `visual`：`kat --debug-visual path/to/file`。用来看 visual regions、block/tight-block/transparent 和成员关系，不要从这里判断最终 block 宽高。
- `layout`：`kat --debug-layout path/to/file`。用来看当前 viewport 下的 wrapped rows、display columns、block bbox 和 background runs；终端 resize / block 几何问题应优先在这一层定位。
- `render_ops`：`kat --debug-render-ops path/to/file`。用来看终端无关的 render plan / state flow，适合做稳定 diff。
- `terminal`：`kat --debug-terminal path/to/file`。用来看 terminal capability 和最终编码后的输出。
- `AST`：`kat --debug-ast --language fish path/to/file`。用来先确认 grammar 实际产出的语法树，再决定问题该落在 query、semantic overlay 还是更后面的层。
- `timing`：`kat --debug-timing path/to/file >/dev/null`。用来看 detect/highlight/semantic/injection/render 等阶段耗时。

## 其他命令

- 跑测试：`just test`
- 提交前检查：`prek run --all-files`
- 校验 grammar 注册与布局：`cargo run --quiet --locked -p validate-grammar-registry`
- 跑仓库性能基线：`just perf`
- 跑单文件性能基线：`just perf-file path/to/file`
- 采集真实 PTY / terminal pane 几何：`./.agents/skills/kat-dev/scripts/terminal_geometry.py`；需要原样回传或机器处理时使用 `./.agents/skills/kat-dev/scripts/terminal_geometry.py --json`。应在需要复现问题的同一个 pane 尺寸下运行。
- 查看版本：`kat --version`
- 普通 TTY 渲染会进入内建 alternate-screen viewer；调试或脚本场景可通过重定向 stdout 直接拿完整输出。

## 输出约定

- `--debug-analysis`、`--debug-visual`、`--debug-render-ops`、`--debug-terminal` 默认输出稳定 JSON，优先用于 snapshot、回归和跨环境 diff。
- 普通 TTY 渲染默认进入内建 viewer；stdout 不是 TTY 时直接输出完整 ANSI 文本。

## 资源

- [architecture.md](../../../docs/architecture.md)
- [language-coverage.md](../../../docs/language-coverage.md)
- [test-assets.md](../../../docs/test-assets.md)
- [justfile](../../../justfile)
