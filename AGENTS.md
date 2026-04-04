# 仓库入口

在这个仓库里开始任何实现决策前，先按下面顺序建立上下文：

1. 读 [README.md](README.md)
2. 按任务类型跳到对应文档
3. 只按需继续下钻，不要默认通读所有文档

## 必读规则

### 第三方来源

当把第三方 grammar、query、scanner 或类似源码复制或改编进这个仓库时，遵循 [docs/maintenance.md](docs/maintenance.md) 里的归属与同步约定。

### Tree-sitter 集成约束

在这个仓库里集成 Tree-sitter grammar 时：

- 只在 `grammars/<name>/` 下保存项目所需的最小源码资产：
  `grammar.js`、`queries/*.scm`，以及可选的 `scanner.c` / `scanner.cc` / `scanner.cpp`
- 不要提交生成出来的 parser 产物，例如 `parser.c`、`grammar.json`、`node-types.json` 或生成出来的头文件；
- `tree-sitter.json` 是可选的，除非项目确实需要其中的元数据，否则不要加入；
- grammar 注册、文件识别规则和构建期选项都要和 `grammars/registry.toml` 保持一致；
- 如果某个 grammar 源码需要额外的构建期 JavaScript 依赖，应通过合适的包管理器安装和跟踪，不要把临时辅助代码随意复制到无关位置；
- 对仓库本地 JavaScript 构建依赖，优先使用 `pnpm`。
- grammar 与 query 的维护方式、归属说明和同步要求统一见 [docs/maintenance.md](docs/maintenance.md)。

### 实现决策默认策略

- 对高收益语言支持、嵌入语言、高亮基础设施或 detector 做实现决策时，默认优先追求最终效果、长期可维护性与架构清晰度，而不是优先压低开发成本；
- 不要为了短期把功能“接上”而引入难以扩展的 hack、脆弱特判或会把复杂度持续下沉到底层的临时设计；
- 如果现有架构明显妨碍需求落地，应优先考虑重构抽象层、runtime 组织方式、检测链路或 grammar 集成模型，让需求通过结构自然成立，而不是继续堆上层例外规则；
- 对已经确认值得投入的一组相关需求，默认应尽量在同一轮内推进到当前架构下难以继续推进为止，而不是只做浅层占位；
- 优先选择小而明确的仓库级约定，而不是依赖隐式的本机环境配置。
- 为了参考其它项目的实现、query 或行为，可以把外部仓库 clone 到 `.external/` 目录下做本地验证；该目录只用于临时参考，不应纳入版本控制。

## 变更同步要求

- 添加或更新某种语言时，要同步更新 `grammars/registry.toml`、运行时识别、测试和第三方归属信息；
- 升级已有 grammar 或 query 时，要同步更新 [THIRD_PARTY.md](THIRD_PARTY.md) 中的来源、revision 与本地改编说明；如果升级带来了行为层面的新结论或边界变化，也要同步更新对应文档；
- 添加或更新语言支持、文件识别规则或相关路线图时，要同步更新 [docs/language-coverage.md](docs/language-coverage.md)；
- 有意义的进展和已经确认的决策要记录到对应文档中，不要只留在聊天历史里；
- 文档分流默认遵循下面约定：
  - 当前稳定架构、构建策略和长期约定写入 [docs/architecture.md](docs/architecture.md)；
  - 当前支持现状、成熟度和检测边界写入 [docs/language-coverage.md](docs/language-coverage.md)；
  - 当前仍未完成的事项、优先级和后续方向写入 [docs/roadmap.md](docs/roadmap.md)；
  - 仓库维护约定写入 [docs/maintenance.md](docs/maintenance.md)；
  - 测试样例约定写入 [docs/test-assets.md](docs/test-assets.md)；
- 仓库只维护根 [README.md](README.md) 这一份中文入口文档，不再维护平行语言版本的 README；
- 默认不要新增按日期组织的阶段性开发过程文档；如果只是过程记录而非当前有效约定，默认不保留；
- 如果仓库工作流发生变化，要同步更新 `AGENTS.md`、README 与相关任务运行器或配置文件；
- 当前 GitHub Actions workflow 默认应先执行测试，再继续 release build matrix；
- 当前 CI 的缓存默认模型是：保留 pnpm store 与 Cargo `registry` / `index` 缓存，不要再把 `target/` 目录当作跨 job 主缓存；
- 对 `build.rs` 内部生成流程（例如 tree-sitter grammar 产物）优先复用本地 `.build-cache/tree-sitter-cache/`；CI 默认不再额外维护 tree-sitter build cache，并继续保留相关 profiling 日志，避免只观察 Rust 编译层；
- 仓库内需要复用的 GitHub Actions 优先放在 `.github/actions/` 下本地维护；若外部 action 的 runtime 或维护状态不理想，优先内建最小可维护实现；
- 需要分析 CI 构建瓶颈时，优先保留并利用 Cargo timings 与 linker timing 这类直接观测数据，而不是依赖额外缓存命中率做推断；
- 保持 `node_modules/` 被忽略，但应提交受版本控制的包元数据，例如 `package.json` 和锁文件；
- 一旦仓库存在 `justfile`，就使用 `just test` 作为标准测试入口。
