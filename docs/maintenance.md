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

主仓库所有 grammar 都使用 crate-backed parser，`grammars/<name>/` 只保存 query 等 kat 集成资产。parser 来源优先选择兼容且许可证合适的公开 crate；需要特化 grammar、scanner 或许可证隔离时，由 [kat-parsers](https://github.com/happy-proto/kat-parsers) 维护源码和预生成产物。来源选择依据由该仓库的 [grammar 维护理由](https://github.com/happy-proto/kat-parsers/blob/master/docs/vendor-grammar-reasons.md) 记录。来源变更通过 Cargo 依赖和 `THIRD_PARTY.md` 跟踪；registry 维护文件识别与 capture 声明，不保存不起作用的 C 编译参数或本地 parser 生成模式。

- `queries/*.scm` 默认按仓库内集成资产管理，可以独立于 grammar revision 演进。
- `grammars/registry.toml` 中的每个 grammar 都必须对应一个 `grammars/<name>/` 目录，并保留自己的 `queries/highlights.scm` 集成资产，供仓库级校验和构建流程使用。
- 仓库级 grammar 布局校验默认通过 workspace 内的独立 test crate `validate-grammar-registry` 覆盖；不要再把这类轻量校验重新挂回主 crate，否则 CI 会为了跑校验而额外触发 `kat` 的 `build.rs`。
- 升级 parser 时，通过 Cargo 更新对应依赖；需要改造 grammar 或 scanner 的语言先在 `kat-parsers` 完成源码与生成产物的同步，再更新主仓库依赖。
- 调整 query 时，优先服务当前仓库的 capture 语义、nested runtime 和 detector 设计，而不是机械贴近某个上游仓库。
- 如果只是为了高亮效果或 injection 行为调整 query，默认不要顺手升级 grammar；只有当 query 目标明确受 upstream AST 或 grammar 变化驱动时，才一起升级 grammar。

## 变更同步

- 添加或更新某种语言时，要同步更新 `grammars/registry.toml`、运行时识别、测试和第三方归属信息。
- 升级已有 grammar 或 query 时，要同步更新 [THIRD_PARTY.md](../THIRD_PARTY.md) 中的来源、revision 与本地改编说明；如果升级带来了行为层面的新结论或边界变化，也要同步更新相关文档。
- 添加或更新语言支持、文件识别规则或相关路线图时，要同步更新 [language-coverage.md](language-coverage.md)。
- 有意义的进展和已经确认的决策要记录到对应文档中，不要只留在聊天历史里。
- 如果仓库工作流发生变化，要同步更新 [README.md](../README.md)、[`hk.pkl`](../hk.pkl) 与相关任务运行器或配置文件；只有确实涉及 agent 专属行为时，再同步更新 `AGENTS.md`。

## 开发环境

仓库通过 [mise](https://mise.jdx.dev/) 管理开发工具；Rust 使用 stable toolchain。首次进入仓库后运行：

```bash
mise install
mise exec -- hk install --mise
```

普通测试使用 `mise run test`；真实 PTY 与 Ghostty 无头终端模型的端到端测试使用 `mise run ghostty-e2e`。Ghostty 构建遇到明确的 Zig 依赖下载网络错误时会有限退避重试；本地 E2E 与 CI 使用相同入口，编译错误直接返回。重试行为的回归检查包含在 `mise run test` 中。

开发任务集中在 [mise.toml](../mise.toml)，使用 `mise tasks` 查看列表，`mise run <任务> --help` 查看带参数任务的用法。

本地复现 CI 的紧凑 parser 静态发布构建使用 `mise run compact-release`，需预先安装 `uv` 和 C 编译器。该命令在独立临时 Cargo home 中提取并压缩 parser 表，生成的二进制在首次使用某语言时按需解压其表。

从仓库源码安装：

```bash
cargo install --git https://github.com/happy-proto/kat --locked kat
```

在本地 checkout 中使用 `mise run install`。源码安装统一使用仓库的 `Cargo.lock`。

## 提交前检查

- 本地提交前检查统一收敛到根目录的 [hk.pkl](../hk.pkl)，手动完整检查使用 `mise exec -- hk check --all`。
- 提交时保存未暂存改动，只检查暂存版本，不自动修复或暂存；完成后恢复未暂存改动。
- TOML / YAML 语法检查由 [check-config.py](../scripts/check-config.py) 承载，使用 `uv` 管理脚本依赖。`check <文件...>` 校验文件，`self-test` 运行内建回归测试，并接入标准测试入口；命令行契约由 Typer 声明，语法测试直接调用文件解析函数。其它检查优先复用 hk 工具和现有命令。
- 全项目 Rust 检查始终执行，顺序为格式、Clippy、`mise run test`，避免 Cargo 构建争用；测试入口同时覆盖 Zig 下载重试回归。
- 调整检查范围时先修改 [hk.pkl](../hk.pkl)，再按需同步文档、任务入口和 CI，检查项不在文档里重复维护。

## GitHub Actions 依赖

Workflow 的外部 action 使用可读的版本标签，实际执行的 commit 与来源仓库身份由 [actions.lock](../.github/workflows/actions.lock) 固定。Rust action 使用 `@v1`，通过 `toolchain: stable` 选择编译器；锁文件只固定 action 代码，不固定它下载的 Rust、工具或系统软件包。同仓库 action 使用 `$/…`，始终解析到当前运行的 commit。

[官方锁文件工具](https://github.com/github/gh-actions-lock) 作为 GitHub CLI 扩展按需安装，直接使用官方命令生成和校验。生成与在线校验需要 GitHub CLI 的登录凭据，或通过 `GH_TOKEN` / `GITHUB_TOKEN` 提供令牌：

```bash
gh extension install github/gh-actions-lock
gh actions-lock
gh actions-lock --verify
```

添加、删除或修改 workflow / action 的 `uses` 后，运行生成命令并一起提交 workflow 与锁文件；不要手工编辑锁文件。已有 GitHub Actions Dependabot 配置会在版本更新 PR 中同步维护锁文件，但不会自动接入新 workflow。

普通生成保留已有锁定。需要主动更新 `@v1` 等可移动引用时，先审查上游变化，再运行 `gh actions-lock --relock`。审查锁文件时关注 commit 变化；来源仓库的 `owner_id` / `repo_id` 变化需要确认仓库身份。

修改 workflow / action 或锁文件时，hk 提交前检查通过 `gh actions-lock --verify-local` 离线验证依赖覆盖，需要先安装扩展；CI 通过 `gh actions-lock --verify` 在线验证锁定 commit、引用与仓库身份。此功能仍处于 GitHub 公开预览；仓库级 Require lockfile 策略是额外的平台设置，当前接入依赖已登记 workflow 的原生执行约束与 CI 校验。

## Release 与分发

- CI、发布与缓存的具体行为以仓库里的 workflow 和配置文件为准；文档只保留长期约定，不重复维护实现级细节。
- CI 中的构建观测数据（例如 cargo timings、link timing）默认优先写入 step summary 与 artifact；不要把原始明细长期直接刷到 stdout 里。
- 如果调整了发布资产或 `cargo binstall` 下载约定，要同步更新相关 workflow、[Cargo.toml](../Cargo.toml) 与 [README.md](../README.md)。
