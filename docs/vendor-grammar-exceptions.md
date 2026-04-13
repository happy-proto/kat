# Vendored Grammar 保留清单

本文档记录当前仍保留 vendored 模式的 grammar，以及为什么它们暂时不迁移到 crate-backed parser。

当前仓库默认策略已经改为：优先使用 crate-backed parser；只有存在明确阻碍时，才把某个 grammar 留在 vendored 模式。

## 没有可用的 crates.io grammar crate

- `git_config`
- `git_link`
- `git_log`
- `git_mailmap`
- `textproto`
- `userscript_metadata`
- `cabal`
- `crontab`
- `dotenv`
- `email`
- `ninja`
- `twig`
- `ssh_config`
- `git_commit`
- `git_rebase`
- `requirements`
- `apache`
- `sass`
- `todotxt`

## 已有 crate，但仍依赖旧版 `tree-sitter`，暂时无法和当前 0.26 系列共存

- `query`
- `fish`
- `markdown`
- `markdown_inline`
- `gitattributes`
- `dot`
- `svelte`
- `just`

## 当前继续 vendored，主要是为了沿用上游 grammar 源和仓库内 query 集成资产

- `actionscript`
- `applescript`
- `asciidoc`
- `asm`
- `awk`
- `bibtex`

## 仓库内有本地 grammar 或 scanner 改造，短期内不适合直接切回上游 crate

- `ignore`
  原因：本地重命名并调整成统一的 ignore-pattern runtime，供 `.gitignore`、`.dockerignore`、`.npmignore` 等文件共享。
- `authorized_keys`
  原因：当前直接在仓库内维护了一个面向 SSH 公钥/授权文件的小 grammar，并配合 `.pub` 内容检测做路径识别。
- `cmakecache`
  原因：仓库内维护了一个专门面向 `CMakeCache.txt` 的小 grammar，用来稳定承接 key/type/value 结构和 path-like value 高亮。
- `command_help`
  原因：当前直接在仓库内维护了一个针对命令帮助文本的小 grammar，用来识别标题、directive 和常见命令行片段。
- `cpuinfo`
  原因：仓库内维护了面向 `cpuinfo` 键值结构的本地 mini-runtime，当前没有值得迁移的现成 crate。
- `dockerfile`
  原因：本地维护了适配 `kat` 宿主模型的 injections query，且当前可用 crate 仍停留在旧版 `tree-sitter` 依赖链上。
- `debsources`
  原因：仓库内维护了 `sources.list` 专用 mini-runtime，用来稳定识别源类型、suite/component 和 URI。
- `fortran_namelist`
  原因：当前直接在仓库内维护了一份面向 namelist 结构的轻量 grammar，和主 `fortran` runtime 分开演进更简单。
- `fstab`
  原因：仓库内维护了 `fstab` / `crypttab` / `mtab` 这类系统表文件的小 grammar，便于按字段类型持续细化。
- `hcl`
  原因：本地 grammar/scanner 做过适配，用来承接 `.hcl` 和 `.nomad` 这类文件的独立 runtime。
- `graphql`
  原因：当前 highlights query 是按 vendored grammar 的节点结构维护的本地集成版本。
- `proto`
  原因：当前 runtime 依赖 vendored grammar 的节点结构和现有 query 约定。
- `gomod`
  原因：当前沿用 vendored grammar 资产和本地 query 集成。
- `gowork`
  原因：当前沿用 vendored grammar 资产和本地 query 集成。
- `gosum`
  原因：当前沿用 vendored grammar 资产和本地 query 集成。
- `less`
  原因：本地 grammar 改成复用仓库里的 `css/grammar.js`，避免引入第二套 CSS 依赖。
- `jinja`
  原因：`grammar.js` 是仓库内包装层，指向 vendored base grammar 和 kat 的 HTML 注入模型。
- `vue`
  原因：本地 scanner 改了 HTML scanner 符号名，确保 Vue runtime 和独立 HTML runtime 可同时链接。
- `scss`
  原因：本地 grammar 改成复用仓库里的 `css/grammar.js`，避免引入第二套 CSS 依赖。

## 许可证原因，继续使用 vendored 方案更合适

- `jq`
  原因：当前 crates.io 上常用的 `tree-sitter-jq` 走 GPL 许可证链路；仓库里改用 BSD 3-Clause 上游并保留 vendored parser 源。

## 例外：parser 已迁移，但仍保留 vendored support asset

- `css`
  原因：虽然 CSS runtime 已迁到 crate-backed，但 `grammars/less/grammar.js` 和 `grammars/scss/grammar.js` 仍会在生成期直接 `require("../css/grammar.js")`，所以仓库里继续保留 `grammars/css/grammar.js` 这份最小 support asset。

## 后续迁移原则

- 默认优先迁移那些没有本地改造、crate API 与当前 `tree-sitter` 主版本兼容、且不会引入额外许可证问题的 grammar。
- 一旦某个保留原因消失，例如上游 crate 升级到当前 `tree-sitter` 主版本，或仓库不再依赖某份 vendored support file，就应优先重新评估迁移。
