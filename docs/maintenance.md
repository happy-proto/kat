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
- `queries/*.scm` 默认按仓库内集成资产管理，可以独立于 grammar revision 演进。
- 如果某个 grammar 改为 crate-backed parser，要在 `grammars/registry.toml` 明确标记 parser 来源，并同步更新相关文档与第三方归属说明；此时仓库内默认不再保留会误导维护者的本地 parser 源文件。
- 升级 grammar 时，优先按单个 upstream revision 同步最小必需源码资产。
- 调整 query 时，优先服务当前仓库的 capture 语义、nested runtime 和 detector 设计，而不是机械贴近某个上游仓库。
- 如果只是为了高亮效果或 injection 行为调整 query，默认不要顺手升级 grammar；只有当 query 目标明确受 upstream AST 或 grammar 变化驱动时，才一起升级 grammar。

## 变更同步

- 添加或更新某种语言时，要同步更新 `grammars/registry.toml`、运行时识别、测试和第三方归属信息。
- 升级已有 grammar 或 query 时，要同步更新 [THIRD_PARTY.md](../THIRD_PARTY.md) 中的来源、revision 与本地改编说明；如果升级带来了行为层面的新结论或边界变化，也要同步更新相关文档。
- 添加或更新语言支持、文件识别规则或相关路线图时，要同步更新 [language-coverage.md](language-coverage.md)。
- 有意义的进展和已经确认的决策要记录到对应文档中，不要只留在聊天历史里。
- 如果仓库工作流发生变化，要同步更新 `AGENTS.md`、[README.md](../README.md) 与相关任务运行器或配置文件。

## Release 与分发

- `master` 分支的 CI 在 `fmt` / `clippy` / 测试与 release build matrix 全部通过后，会覆盖更新 GitHub Releases 的 `latest` prerelease channel。
- `latest` channel 绑定一个同名 tag，并始终指向当前最新一次成功发布的 `master` commit；不要把它当作稳定版本 tag 使用。
- 供 `cargo binstall --git` 使用的 release 资产命名与包内目录约定保持固定：资产名使用 `kat-<target>.(tgz|zip)`，包内目录使用 `kat-<target>/`，其中包含最终可执行文件 `kat` 或 `kat.exe`。
- 如果调整了 `latest` channel 的资产命名、包结构或发布标签，要同步更新 [Cargo.toml](../Cargo.toml) 里的 `package.metadata.binstall` 与 [README.md](../README.md) 中的安装说明。
