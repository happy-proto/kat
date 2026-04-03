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
- 仓库本地 JavaScript 构建依赖统一在根目录管理，构建前先执行 `pnpm install`。
- Tree-sitter 中间产物在本地会缓存到仓库级 `.build-cache/tree-sitter-cache/`，与 Cargo 的 `target/` 产物目录解耦，以便在不同 Cargo 命令之间复用。
- CI 只保留 pnpm store 与 Cargo `registry` / `index` 缓存，不再额外维护 `sccache` 或 tree-sitter build cache；`target/` 仍不是跨 job 缓存对象。

### 运行时模型

- 高亮运行时基于共享 capture 注册和统一 `HighlightConfiguration` 组装。
- 文档检测不再只返回“基础语言名”，而是返回 `document kind`：把底层 grammar/runtime 与文档 profile 分开建模。当前 profile 至少已覆盖普通 YAML、GitHub Actions workflow YAML、`action.yml` 这类 GitHub Action metadata YAML。
- 嵌套高亮拆成两层：通用的 Tree-sitter query 注入，以及按宿主 / profile 注册的 host resolver。前者继续承接通用 injection 规则，后者负责 `Dockerfile` shell dispatch、GitHub Actions `run` + `shell` / `defaults.run.shell` 分发这类仅靠 query 不够稳定的场景。
- 对 `Justfile` recipe、Markdown fenced code、GitHub Actions `run` block 这类明显是“块级运行时区域”的注入，renderer 现在会基于注入 range 和共享缩进推导矩形 block range，而不是只给每一行已有文本上色。
- 这套 block region renderer 会在较短行尾部补带背景色的空格，把同一个嵌套区域渲染成视觉上连续的矩形块；这是当前设计的一部分，不再假设输出一定逐字节保留原始行尾。
- 对 shell、Regex、SQL、JSDoc 以及 GitHub Actions expression 这类仅靠 highlights query 难以长期稳定表达局部结构语义的语言 / profile，允许在基础 capture 之后叠加一层轻量 semantic overlay；这层仍建立在 AST 或局部语法扫描之上，而不是把特判塞进 renderer。GitHub Actions 这层 overlay 现在既作用于 YAML 宿主上的 expression，也可叠加到 `run` block 注入出来的 shell / Python 子语言上。
- 共享 runtime 只承接真正共享 AST / 语义模型的语言；像 Protocol Buffers schema (`.proto`) 与 Protocol Buffers text format (`.textproto` / `.pbtxt`) 这种虽然同属一个生态、但语法角色不同的文件类型，应拆成独立 runtime，而不是在同一 grammar 上叠加 profile 特判。
- 主题系统按 capture 语义落色，不依赖“当前来自哪一层语言”这种渲染期上下文。
- 终端背景色查询被收敛在 [terminal_background.rs](../src/terminal_background.rs) 这一层；当前用 `terminal-colorsaurus` 作为临时 OSC 11 后端，只负责给 nested region tint 提供基础背景色输入，未来应迁移到仓库自己的统一 terminal API 层。
- SQL 方言、Regex host-aware runtime、Justfile shell dispatch、Dockerfile shell dispatch、GitHub Actions `run`/`shell` dispatch 等能力都建立在这套共享 runtime + document profile + host resolver 模型之上。

## 维护约定

- grammar 与 query 默认分开治理：grammar 源文件按上游 revision 快照管理，query 按仓库内集成资产独立演进。
- 具体同步和归属约定见 [maintenance.md](maintenance.md)。

### 文档分工

- 对外概览放在根 [README.md](../README.md)。
- 当前支持现状统一放在 [language-coverage.md](language-coverage.md)。
- 未完成事项统一放在 [roadmap.md](roadmap.md)。
- 仓库维护约定见 [maintenance.md](maintenance.md)。
- 测试样例约定见 [test-assets.md](test-assets.md)。

## 已知边界

- 只有仓库里已经注册并构建的 runtime 才能作为可高亮的注入目标。
- 无扩展名内容检测目前仍是有限启发式，不是完整内容识别系统。
- 部分共享 grammar 的表达能力仍然限制了 query 可细化的上限，例如 SQL、Regex、JSDoc 一类场景。
- vendored 大型 grammar 的首次 parser 生成成本仍然偏高；需要继续权衡是保持 vendored 还是转成 crate-backed。
- `scanner.cc` 路线已经在当前环境验证过，但仍需要补 Linux 构建确认。
