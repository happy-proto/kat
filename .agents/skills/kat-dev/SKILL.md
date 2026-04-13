---
name: kat-dev
description: 介绍 kat 当前架构分层，并按层整理调试命令与其他常用开发命令；当需要修改 kat、排查高亮链路或理解 analysis/visual/render_ops/terminal 关系时使用。
---

先按 `analysis -> visual -> render_ops -> terminal` 这条链理解 `kat`。更完整的稳定结论见 [architecture.md](../../../docs/architecture.md)。

## 架构分层

- `analysis`：负责 document kind 检测、基础高亮、semantic overlay 和 injection region 收集。
- `visual`：负责把 analysis 结果整理成稳定视觉模型，包括 styled spans 和 nested region 的视觉分段。
- `render_ops`：负责把视觉模型编译成终端无关的渲染状态流。
- `terminal`：负责终端能力探测、分页接入和最终 ANSI 编码输出。

## 按层调试命令

- `analysis`：`kat --debug-analysis path/to/file`。用来看 document kind、nested regions、runtime identity，以及 analysis 层稳定 JSON。
- `semantic overlay`：`kat --debug-semantics --language sql_postgres path/to/file`。用来看 query 之外补上的结构语义；`--debug-shell-semantics` 是兼容别名。
- `visual`：`kat --debug-visual path/to/file`。用来看 visual regions、block/tight-block/transparent 等视觉层结果。
- `render_ops`：`kat --debug-render-ops path/to/file`。用来看终端无关的 render plan / state flow，适合做稳定 diff。
- `terminal`：`kat --debug-terminal path/to/file`。用来看 terminal capability 和最终编码后的输出。
- `AST`：`kat --debug-ast --language fish path/to/file`。用来先确认 grammar 实际产出的语法树，再决定问题该落在 query、semantic overlay 还是更后面的层。
- `timing`：`kat --debug-timing --paging=never path/to/file >/dev/null`。用来看 detect/highlight/semantic/injection/render 等阶段耗时。

## 其他命令

- 跑测试：`just test`
- 提交前检查：`prek run --all-files`
- 校验 grammar 注册与布局：`cargo run --quiet --locked -p validate-grammar-registry`
- 跑仓库性能基线：`just perf`
- 跑单文件性能基线：`just perf-file path/to/file`
- 查看版本：`kat --version`
- 控制长输出分页：`kat --paging=auto|always|never path/to/file`

## 输出约定

- `--debug-analysis`、`--debug-visual`、`--debug-render-ops`、`--debug-terminal` 默认输出稳定 JSON，优先用于 snapshot、回归和跨环境 diff。
- 长输出默认支持 `--paging=auto|always|never`；`auto` 会在 TTY 中按屏高决定是否接入 pager。

## 资源

- [architecture.md](../../../docs/architecture.md)
- [language-coverage.md](../../../docs/language-coverage.md)
- [test-assets.md](../../../docs/test-assets.md)
- [justfile](../../../justfile)
