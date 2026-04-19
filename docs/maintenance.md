# 维护约定

本文档只记录仓库级维护约定，不再混入测试样例说明或阶段性过程记录。

## 文档分工

- 根 [README.md](../README.md) 只做项目简介和少量导航，不承担完整状态清单。
- 当前支持现状统一维护在 [language-coverage.md](language-coverage.md)。
- 当前稳定架构、构建策略和已知边界统一维护在 [architecture.md](architecture.md)。
- 当前未完成事项统一维护在 [roadmap.md](roadmap.md)。
- 测试样例组织和补样例约定统一维护在 [test-assets.md](test-assets.md)。

## 第三方来源与归属

- 仓库内引入的第三方 grammar 源文件及其归属说明见 [THIRD_PARTY.md](../THIRD_PARTY.md)。
- 引入或改编第三方 grammar、query、scanner 时，要同步更新来源、许可证和仓库级归属说明。
- 不要把导入的源码表述成项目原创代码。

## Grammar 与 Query 维护

- `grammar.js`、`scanner.*` 和 grammar 构建必需的 support 源文件，默认按上游 revision 快照管理。
- 对 vendored `scanner.c` / `scanner.cc` / `scanner.cpp` 的本地改动，优先保持跨编译器可移植；不要在函数声明里直接写裸 `__attribute__(...)` 之类的 GNU 扩展，确实需要编译器特定属性时要通过可移植宏封装。
- `queries/*.scm` 默认按仓库内集成资产管理，可以独立于 grammar revision 演进。
- 如果某个 grammar 改为 crate-backed parser，要在 `grammars/registry.toml` 明确标记 parser 来源，并同步更新相关文档与第三方归属说明；此时仓库内默认不再保留会误导维护者的本地 parser 源文件。
- `grammars/registry.toml` 中的每个 grammar 都必须对应一个 `grammars/<name>/` 目录；vendored grammar 必须保留 `grammar.js`，crate-backed grammar 也必须至少保留一个本地资产文件（通常是 `queries/*.scm`），这样仓库级校验、构建缓存和维护流程才能稳定工作。
- 仓库级 grammar 布局校验默认通过 workspace 内的独立工具包 `validate-grammar-registry` 执行；不要再把这类轻量校验重新挂回主 crate，否则 CI 会为了跑校验而额外触发 `kat` 的 `build.rs`。
- 升级 grammar 时，优先按单个 upstream revision 同步最小必需源码资产。
- 调整 query 时，优先服务当前仓库的 capture 语义、nested runtime 和 detector 设计，而不是机械贴近某个上游仓库。
- 如果只是为了高亮效果或 injection 行为调整 query，默认不要顺手升级 grammar；只有当 query 目标明确受 upstream AST 或 grammar 变化驱动时，才一起升级 grammar。

## 变更同步

- 添加或更新某种语言时，要同步更新 `grammars/registry.toml`、运行时识别、测试和第三方归属信息。
- 升级已有 grammar 或 query 时，要同步更新 [THIRD_PARTY.md](../THIRD_PARTY.md) 中的来源、revision 与本地改编说明；如果升级带来了行为层面的新结论或边界变化，也要同步更新相关文档。
- 添加或更新语言支持、文件识别规则或相关路线图时，要同步更新 [language-coverage.md](language-coverage.md)。
- 有意义的进展和已经确认的决策要记录到对应文档中，不要只留在聊天历史里。
- 如果仓库工作流发生变化，要同步更新 [README.md](../README.md)、`prek.toml` 与相关任务运行器或配置文件；只有确实涉及 agent 专属行为时，再同步更新 `AGENTS.md`。

## 提交前检查

- 本地提交前检查统一收敛到根目录的 `prek.toml`。
- 安装 Git hooks 使用 `prek install`；默认使用 `pre-commit` 的标准入口。
- 当前提交前检查统一通过默认 `prek run` 执行；具体检查项只在 `prek.toml` 中维护。
- 如果调整了提交前检查范围，优先修改 `prek.toml`，再同步 README、相关任务入口与 CI 说明，避免把配置内容重复散落到其它文档里。

## Release 与分发

- CI、发布与缓存的具体行为以仓库里的 workflow 和配置文件为准；文档只保留长期约定，不重复维护实现级细节。
- 如果调整了发布资产或 `cargo binstall` 下载约定，要同步更新相关 workflow、[Cargo.toml](../Cargo.toml) 与 [README.md](../README.md)。
