# 路线图

本文档只记录当前仍未完成的事项、明确的后续方向和已知边界，不再混入已经完成的阶段过程。

## 当前优先级

1. 继续收敛 Dracula 主题与 capture 语义映射，观察是否需要更清晰的分层主题模型。
2. 继续优化大型 grammar 的冷构建成本，尤其是 parser generation 最重的语言。
3. 在 Linux 上补跑包含 `scanner.cc` 的完整构建验证。
4. 评估是否要把当前 block region renderer 继续扩展到更多非矩形或 inline 注入场景，同时保持 renderer 抽象统一，而不是回退到宿主特判。
5. 继续优化特殊文件名和无扩展名输入的语言识别策略。
6. 保持 [README.md](../README.md)、[language-coverage.md](language-coverage.md) 与实际仓库状态同步。

## 语言与 Runtime

### SQL

- 继续增强 SQL 方言 detector，补更多内容感知信号和宿主上下文信号。
- 继续评估“共享 grammar + 方言 semantic overlay”是否还能覆盖主要收益场景。
- 当共享 grammar 的表达上限明显成为瓶颈时，再评估拆成 per-dialect grammar。

### Regex

- 继续补 host-aware Regex runtime 的高收益 query 细节，并观察哪些结构更适合留在 semantic overlay。
- 对 AST 没有稳定表达的 token，优先接受 grammar 上限，不往 renderer 塞脆弱特判。

### Shell

- 继续沿共享 shell semantic layer 细化 `fish` / `zsh` / `bash` 的 builtin family、argument role、list access 与 expansion 结构语义。
- 继续细化 `zsh` query 与 semantic layer 的分工，尤其是 option、parameter expansion、glob qualifier、arrays 和 arithmetic 场景。
- 继续观察 `zsh` grammar 的首次构建成本与增量构建体验。

### GraphQL

- 当前主要收益来自 query 继续细化，而不是重做 runtime 分发路径。

### Python / JavaScript / HTML / CSS / JSDoc

- 这些语言已经脱离占位阶段，但仍有继续精细化空间。
- `JSDoc` 已开始用 semantic overlay 补 inline reference target；后续是否继续深挖，应先确认是 query 问题还是 upstream grammar 表达上限问题。

## 检测与特殊文件

- 继续优化无扩展名或特殊路径文件的内容感知识别。
- 评估是否需要引入额外文件类型探测信号，而不是只依赖扩展名、文件名和 shebang。
- 继续补 shell 生态高频特殊文件识别，但避免无边界扩规则。
- 继续细化 Dockerfile 参数子结构和 shell-form / JSON-form 的边界一致性。

## 文档类与轻量文本类需求

### License 文件

- `LICENSE`、`COPYING`、`NOTICE`、`PATENTS` 这类文件，优先走“特殊文件识别 + 轻量文本语义高亮”路线。
- 第一阶段可以先映射到 Markdown runtime，再根据真实效果决定是否补更轻量的 license-aware 增强。

## 延后但保留的方向

- 前端文件支持仍保留需求记录，后续再统一决定 Vue / React 的文件范围、宿主模型和 detector 策略。
- 当需要跨多个相邻字符串做语义级拼接时，再评估 combined injection 与 decode 是否需要进一步统一建模。
- 当前 nested region tint 的终端背景色查询仍通过 `terminal-colorsaurus` 临时承接；长期应迁移到 kat 自己的 terminal API 层，理想方向是接到支持颜色查询的 `termwiz` 方案上，避免双 terminal I/O 栈并存。
- 如果后续要继续尝试更弱侵入的边界装饰，例如 rail、侧边界或其它非背景式对比，也应等统一 terminal 抽象稳定后再评估，而不是继续在现有查询路径上叠加临时实现。
