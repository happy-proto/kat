# 架构与构建

本文档只记录当前稳定的架构结论、构建策略和已知边界，不再保留按日期展开的实验过程。

## 项目方向

`kat` 的目标是做一个尽可能为更多高价值文件类型提供语法高亮的 `cat` 风格终端查看工具。

当前技术方向已经收敛为：

- 基于 Tree-sitter 做语法识别和高亮；
- 在仓库内维护最小 grammar 资产，而不是依赖各语言的 Rust grammar crate；
- 用统一 runtime 和 injection 基础设施承接嵌套高亮、方言分发和宿主感知子语言；
- 让 grammar、query、detector 和 build 行为都由仓库内约定直接描述。

## 当前架构结论

### Grammar 资产模型

- `grammars/<name>/` 只保留项目集成必需的最小源码资产：
  `grammar.js`、`queries/*.scm`，以及可选的 `scanner.*` 或必要 support 文件。
- 不提交生成出来的 parser 产物，例如 `parser.c`、`grammar.json`、`node-types.json`。
- `grammars/registry.toml` 是 grammar 注册、构建参数和运行时识别规则的单一事实来源。

### 构建模型

- `build.rs` 在构建期通过 `tree-sitter-generate` 生成 parser C 源码。
- 生成出来的 `parser.c` 与 vendored `scanner.c` / `scanner.cc` / `scanner.cpp` 一起参与本地编译并静态链接进最终二进制。
- 仓库本地 JavaScript 构建依赖统一在根目录管理，构建前先执行 `pnpm install`。
- Tree-sitter 中间产物缓存到仓库级 `target/tree-sitter-cache/`，以便在不同 Cargo 命令之间复用。

### 运行时模型

- 高亮运行时基于共享 capture 注册和统一 `HighlightConfiguration` 组装。
- 文档检测不再只返回“基础语言名”，而是返回 `document kind`：把底层 grammar/runtime 与文档 profile 分开建模。当前 profile 至少已覆盖普通 YAML、GitHub Actions workflow YAML、`action.yml` 这类 GitHub Action metadata YAML。
- 嵌套高亮拆成两层：通用的 Tree-sitter query 注入，以及按宿主 / profile 注册的 host resolver。前者继续承接通用 injection 规则，后者负责 `Dockerfile` shell dispatch、GitHub Actions `run` + `shell` 分发这类仅靠 query 不够稳定的场景。
- 对 shell、Regex、SQL、JSDoc 以及 GitHub Actions expression 这类仅靠 highlights query 难以长期稳定表达局部结构语义的语言 / profile，允许在基础 capture 之后叠加一层轻量 semantic overlay；这层仍建立在 AST 或局部语法扫描之上，而不是把特判塞进 renderer。
- 共享 runtime 只承接真正共享 AST / 语义模型的语言；像 Protocol Buffers schema (`.proto`) 与 Protocol Buffers text format (`.textproto` / `.pbtxt`) 这种虽然同属一个生态、但语法角色不同的文件类型，应拆成独立 runtime，而不是在同一 grammar 上叠加 profile 特判。
- 主题系统按 capture 语义落色，不依赖“当前来自哪一层语言”这种渲染期上下文。
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
- 大型 grammar 的首次 parser 生成成本仍然偏高，冷构建优化仍是后续工作。
- `scanner.cc` 路线已经在当前环境验证过，但仍需要补 Linux 构建确认。
