# Kat

`kat` 是一个高亮版的 `cat`。

我之前主要用 `bat`，但它在嵌套高亮这类场景上，尤其是 `Justfile` 支持，一直没有特别顺手。`bat` 底层使用的是 `syntect`，读取的是 Sublime Text 的 `.sublime-syntax`；这套模型可以处理一部分嵌套高亮，但对我想要的那种更深、更多依赖结构语义的嵌套高亮，似乎较难继续做深，所以我就开始自己写这个。

这个项目直接选了基于 Tree-sitter 的路线，并把 grammar、query、scanner、detector 和 nested runtime 都尽量放在仓库里。这样做不是为了追求某种“更标准”的架构，而是为了把更多中间层暴露出来，方便继续实验这类更依赖结构语义和嵌套分发的高亮场景，比如 Rust 文档注释里的 Markdown、`Justfile` 里的 Bash/Python/Zsh/Fish recipe、宿主语言里的 SQL/GraphQL/Regex，以及特殊文件名、shebang 和无扩展名内容识别。

这个项目实现过程中也大量参考了 `Zed` 的代码。它支持了很多语言的高亮，而且效果很好，所以在 query、嵌套语言处理和语言支持范围这些问题上，它一直是一个很有价值的参考对象。

这个项目本质上仍然只是个人玩具：纯 vibe coding，完全没有 review。

重要文档：

- 当前支持现状：[docs/language-coverage.md](docs/language-coverage.md)
- 架构说明：[docs/architecture.md](docs/architecture.md)
- 路线图：[docs/roadmap.md](docs/roadmap.md)
- 仓库维护约定：[docs/maintenance.md](docs/maintenance.md)
- 测试样例约定：[docs/test-assets.md](docs/test-assets.md)

开发调试：

- 查看某门语言的 AST：`kat --debug-ast --language fish path/to/file`
- 查看 semantic overlay 命中的结构语义：`kat --debug-semantics --language sql_postgres path/to/file`
- `--debug-shell-semantics` 仍保留为兼容别名，但现在输出的是通用 semantic overlay 结果
- 长输出默认支持外部分页：`--paging=auto|always|never`，`auto` 会在 TTY 中按屏高判断是否接入 pager；pager 命令优先读 `PAGER`，未设置时默认回退到 `less -R -F -X`
- Tree-sitter 构建中间产物在本地仍会落到仓库级 `.build-cache/tree-sitter-cache/`，用于复用 `build.rs` 生成出来的 grammar 资产
- CI 当前只保留 pnpm store 与 Cargo `registry` / `index` 缓存，不再额外维护 `sccache` 或 tree-sitter build cache 的 GitHub Actions 逻辑
- CI 的 release 构建会额外上传 Cargo timings HTML、linker timing 日志和 tree-sitter build profile，用于判断瓶颈是否落在最终链接阶段还是 `build.rs`
