# Vendored Grammar 现状

当前 `kat` 主仓库已经不再保留需要在本仓库内生成 parser 的 vendored grammar。

现在的默认策略是：

- 优先直接使用 crates.io 上与当前 `tree-sitter` 主版本兼容的 grammar crate；
- 如果上游没有合适 crate、许可证不合适，或者仍需要保留仓库级特化 grammar 源，就把 parser 源外置到 [`kat-parsers`](https://github.com/happy-proto/kat-parsers)，由那个仓库预生成并提交 `parser.c` 等产物；
- 当前 `kat` 主仓库只继续保留 queries 等 kat 侧集成资产，不再重复保留会误导维护者的本地 parser 源文件。

## 当前状态

- 当前 `grammars/registry.toml` 中所有语言都已经走 crate-backed parser 路线。
- 主仓库的 `grammars/<name>/` 目录现在只保留 kat 集成仍然需要的本地资产，通常是 `queries/*.scm`。
- 历史上为什么某些 grammar 长期留在主仓库里 vendored，以及这些理由在迁到外部 parser bundle 后如何继续维护，统一见 [`kat-parsers` 的 `docs/vendor-grammar-reasons.md`](https://github.com/happy-proto/kat-parsers/blob/master/docs/vendor-grammar-reasons.md)。

## 后续原则

- 如果某个语言已经可以直接走稳定的 crates.io crate，优先继续回切到公开 crate，而不是长期依赖外部 parser bundle。
- 如果某个语言仍然需要本地 grammar 包装层、特殊 scanner 或许可证隔离，优先把 parser 源继续维护在 `kat-parsers`，不要重新把生成链路搬回 `kat` 主仓库。
- 任何 parser 来源变化都要同步更新 `grammars/registry.toml`、`THIRD_PARTY.md`、以及相关维护文档。
