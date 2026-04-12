# 架构与构建

本文档只记录当前稳定的架构结论、构建策略和已知边界，不再保留按日期展开的实验过程。

## 项目方向

`kat` 的目标是做一个尽可能为更多高价值文件类型提供语法高亮的 `cat` 风格终端查看工具。

当前技术方向已经收敛为：

- 基于 Tree-sitter 做语法识别和高亮；
- 默认在仓库内维护最小 grammar 资产；只有当某个 grammar 的冷构建生成成本明显高到影响整体构建体验时，才允许切换到预生成 parser 的 Rust grammar crate；
- 用统一 runtime 和 injection 基础设施承接嵌套高亮、方言分发和宿主感知子语言；
- 让 grammar、query、detector 和 build 行为都由仓库内约定直接描述。

## 当前架构结论

### Grammar 资产模型

- `grammars/<name>/` 默认只保留项目集成必需的最小源码资产：
  `grammar.js`、`queries/*.scm`，以及可选的 `scanner.*` 或必要 support 文件。
- 不提交生成出来的 parser 产物，例如 `parser.c`、`grammar.json`、`node-types.json`。
- `grammars/registry.toml` 是 grammar 注册、parser 来源、构建参数和运行时识别规则的单一事实来源。
- 当某个 grammar 标记为 crate-backed 时，仓库内只保留 queries 等集成资产；底层 parser AST 由对应 Rust crate 提供。

### 构建模型

- `build.rs` 对 vendored grammar 在构建期通过 `tree-sitter-generate` 生成 parser C 源码。
- vendored grammar 的 `parser.c` 会与仓库内 `scanner.c` / `scanner.cc` / `scanner.cpp` 一起参与本地编译并静态链接进最终二进制。
- 对 crate-backed grammar，`kat` 不再在自己的 `build.rs` 中重新生成 parser，而是直接链接对应 grammar crate 提供的预生成 parser。
- 构建缓存与 CI cache 的具体策略以 workflow 和相关配置为准；这里不重复展开实现级细节。

### 运行时模型

- 高亮运行时基于共享 capture 注册和统一 `HighlightConfiguration` 组装。
- 文档检测不再只返回“基础语言名”，而是返回 `document kind`：把底层 grammar/runtime 与文档 profile 分开建模。
- 嵌套高亮拆成两层：通用的 Tree-sitter query 注入，以及按宿主 / profile 注册的 host resolver。前者继续承接通用 injection 规则，后者负责 `Dockerfile` shell dispatch、GitHub Actions `run` + `shell` / `defaults.run.shell` 分发这类仅靠 query 不够稳定的场景。
- 对 shell、Regex、SQL、JSDoc 以及 GitHub Actions expression 这类仅靠 highlights query 难以长期稳定表达局部结构语义的语言 / profile，允许在基础 capture 之后叠加轻量 semantic overlay。
- 共享 runtime 只承接真正共享 AST / 语义模型的语言；像 Protocol Buffers schema (`.proto`) 与 Protocol Buffers text format (`.textproto` / `.pbtxt`) 这种虽然同属一个生态、但语法角色不同的文件类型，应拆成独立 runtime，而不是在同一 grammar 上叠加 profile 特判。
- SQL 方言、Regex host-aware runtime、Justfile shell dispatch、Dockerfile shell dispatch、GitHub Actions `run`/`shell` dispatch 等能力都建立在这套共享 runtime + document profile + host resolver 模型之上。

### 渲染分层

当前渲染链路明确拆成 4 层：

1. `analysis`
   - 负责 document kind 检测、基础 highlight、semantic overlay、injection region 收集。
   - 这一层内部仍保留 detect / highlight / semantic / injections 的细分，不会因为外层抽象而把 parser 与高亮逻辑重新揉平。
2. `visual`
   - 负责把 analysis 层产物整理成稳定的视觉模型：styled spans、visual regions、block/tight-block/transparent 的区域结果。
   - 对块级嵌套区域，会按共享缩进和注入 range 推导统一的视觉区域，而不是只给已有文本逐行上色。
3. `render_ops`
   - 负责把视觉模型编译成终端无关的渲染状态流，而不是直接拼 ANSI 字符串。
   - 这层输出的是稳定 IR，适合做 snapshot、回归和跨环境 diff。
4. `terminal`
   - 负责终端能力探测、背景查询开关和 ANSI 编码视图。
   - CLI 输出层仍以“terminal 层编码出完整 ANSI 文本”为边界；长输出优先交给外部分页器，而不是继续向内建 TUI 演进。

### 视觉与终端约定

- 注入区域的视觉策略默认由 runtime 统一推导；块级区域和行内片段走不同的默认视觉模型。
- 与终端显示列宽相关的逻辑统一收口到 `display_geometry` 模块；显示宽度、前缀列位置、ANSI 剥离以及后续 tab stop 策略都必须通过这层抽象处理，不允许在 visual / render / test 里直接用 `len()`、字节差值或 `chars().count()` 近似显示宽度。
- `display_geometry` 内部显式区分 `ByteOffset` 与 `DisplayColumn`：前者只代表源码 UTF-8 偏移，后者只代表终端显示列。新的几何逻辑应优先沿用这两个类型，而不是继续把裸 `usize` 当作双重语义容器。
- `visual` / `render_ops` 仍然保留源码 byte offset 作为文本切片边界，但像 `RectBlock` 这类需要补齐右边界的区域，必须先通过 `display_geometry` 把内容换算成显示列宽，再反推出需要补多少尾部空格；不要再把源码 byte offset 直接当成终端列宽。
- `display_geometry` 当前内建统一的 Unicode 宽度规则，并把 tab stop 作为仓库级策略集中定义；如果未来要支持不同 terminal profile 或 East Asian 宽度策略，也应继续在这层扩展，而不是把特殊逻辑散落回各个语言 / 渲染分支里。
- 主题系统按 capture 语义落色，不依赖“当前来自哪一层语言”这种渲染期上下文。
- 终端背景色查询不再由 `theme` 直接触发，而是通过 `terminal` 层能力探测统一接入；当前 OSC 11 后端仍落在 [terminal_background.rs](../src/terminal_background.rs)。
- `kat` 现在提供稳定 JSON debug 出口：`--debug-analysis`、`--debug-visual`、`--debug-render-ops`、`--debug-terminal`，分别覆盖 analysis、visual、render IR 和 terminal 编码层。

## 维护约定

- grammar 与 query 默认分开治理：grammar 源文件按上游 revision 快照管理，query 按仓库内集成资产独立演进。
- 具体同步和归属约定见 [maintenance.md](maintenance.md)。
- 仍保留 vendored 模式的 grammar 以及保留原因，统一记录在 [vendor-grammar-exceptions.md](vendor-grammar-exceptions.md)。

### 文档分工

- 对外概览放在根 [README.md](../README.md)。
- 当前支持现状统一放在 [language-coverage.md](language-coverage.md)。
- 未完成事项统一放在 [roadmap.md](roadmap.md)。
- 仓库维护约定见 [maintenance.md](maintenance.md)。
- vendored grammar 保留清单见 [vendor-grammar-exceptions.md](vendor-grammar-exceptions.md)。
- 测试样例约定见 [test-assets.md](test-assets.md)。

## 已知边界

- 只有仓库里已经注册并构建的 runtime 才能作为可高亮的注入目标。
- 无扩展名内容检测目前仍是有限启发式，不是完整内容识别系统。
- 部分共享 grammar 的表达能力仍然限制了 query 可细化的上限，例如 SQL、Regex、JSDoc 一类场景。
- vendored 大型 grammar 的首次 parser 生成成本仍然偏高；需要继续权衡是保持 vendored 还是转成 crate-backed。
- `scanner.cc` 路线已经在当前环境验证过，但仍需要补 Linux 构建确认。
