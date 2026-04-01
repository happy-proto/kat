# Third-Party Sources

This repository includes selected upstream Tree-sitter grammar source files.

Included sources:

- `grammars/json/grammar.js`
- `grammars/json/queries/highlights.scm`
- `grammars/ignore/grammar.js`
- `grammars/ignore/queries/highlights.scm`
- `grammars/dockerfile/grammar.js`
- `grammars/dockerfile/queries/highlights.scm`
- `grammars/dockerfile/queries/injections.scm`
- `grammars/dockerfile/scanner.c`
- `grammars/bash/grammar.js`
- `grammars/bash/queries/highlights.scm`
- `grammars/bash/queries/injections.scm`
- `grammars/bash/scanner.c`
- `grammars/fish/grammar.js`
- `grammars/fish/queries/highlights.scm`
- `grammars/fish/scanner.c`
- `grammars/zsh/grammar.js`
- `grammars/zsh/queries/highlights.scm`
- `grammars/zsh/queries/injections.scm`
- `grammars/zsh/scanner.c`
- `grammars/toml/grammar.js`
- `grammars/toml/queries/highlights.scm`
- `grammars/toml/scanner.c`
- `grammars/yaml/grammar.js`
- `grammars/yaml/queries/highlights.scm`
- `grammars/yaml/scanner.c`
- `grammars/yaml/schema.core.c`
- `grammars/hcl/grammar.js`
- `grammars/hcl/queries/highlights.scm`
- `grammars/hcl/scanner.c`
- `grammars/rust/grammar.js`
- `grammars/rust/queries/highlights.scm`
- `grammars/rust/queries/injections.scm`
- `grammars/rust/scanner.c`
- `grammars/python/grammar.js`
- `grammars/python/queries/highlights.scm`
- `grammars/python/queries/injections.scm`
- `grammars/python/scanner.cc`
- `grammars/go/grammar.js`
- `grammars/go/queries/highlights.scm`
- `grammars/go/queries/injections.scm`
- `grammars/gomod/grammar.js`
- `grammars/gomod/queries/highlights.scm`
- `grammars/gowork/grammar.js`
- `grammars/gowork/queries/highlights.scm`
- `grammars/gosum/grammar.js`
- `grammars/gosum/queries/highlights.scm`
- `grammars/graphql/grammar.js`
- `grammars/graphql/queries/highlights.scm`
- `grammars/sql/grammar.js`
- `grammars/sql/queries/highlights.scm`
- `grammars/sql/queries/highlights-postgres.scm`
- `grammars/sql/queries/highlights-mysql.scm`
- `grammars/sql/queries/highlights-sqlite.scm`
- `grammars/sql/scanner.c`
- `grammars/html/grammar.js`
- `grammars/html/queries/highlights.scm`
- `grammars/html/queries/injections.scm`
- `grammars/html/scanner.c`
- `grammars/html/tag.h`
- `grammars/css/grammar.js`
- `grammars/css/queries/highlights.scm`
- `grammars/css/scanner.c`
- `grammars/javascript/grammar.js`
- `grammars/javascript/queries/highlights.scm`
- `grammars/javascript/queries/injections.scm`
- `grammars/javascript/queries/locals.scm`
- `grammars/javascript/scanner.c`
- `grammars/regex/grammar.js`
- `grammars/regex/queries/highlights.scm`
- `grammars/regex/queries/highlights-javascript.scm`
- `grammars/regex/queries/highlights-python.scm`
- `grammars/regex/queries/highlights-rust.scm`
- `grammars/regex/queries/highlights-go.scm`
- `grammars/regex/queries/highlights-posix.scm`
- `grammars/jsdoc/grammar.js`
- `grammars/jsdoc/queries/highlights.scm`
- `grammars/jsdoc/scanner.c`
- `grammars/markdown/grammar.js`
- `grammars/markdown/common.js`
- `grammars/markdown/html_entities.json`
- `grammars/markdown/queries/highlights.scm`
- `grammars/markdown/queries/injections.scm`
- `grammars/markdown/scanner.c`
- `grammars/markdown_inline/grammar.js`
- `grammars/markdown_inline/queries/highlights.scm`
- `grammars/markdown_inline/queries/injections.scm`
- `grammars/markdown_inline/scanner.c`
- `grammars/just/grammar.js`
- `grammars/just/queries/highlights.scm`
- `grammars/just/queries/injections.scm`
- `grammars/just/scanner.c`

Upstream projects:

- `grammars/json/grammar.js`
  Source: [tree-sitter/tree-sitter-json](https://github.com/tree-sitter/tree-sitter-json)
  Revision: `b8d881b4699dbe2e8b88d88864e7b220b9631a55`
  License: MIT
  Notes: repository-local copy of the upstream grammar source; generated parser artifacts are not stored.

- `grammars/json/queries/highlights.scm`
  Sources:
  - [tree-sitter/tree-sitter-json](https://github.com/tree-sitter/tree-sitter-json)
  - [zed-industries/zed](https://github.com/zed-industries/zed)
  Revisions: not recorded during the initial import
  Licenses: MIT and Apache-2.0
  Notes: repository-local adapted highlights query for kat's terminal renderer and Dracula-oriented capture model.

- `grammars/ignore/*`
  Source: [shunsambongi/tree-sitter-gitignore](https://github.com/shunsambongi/tree-sitter-gitignore)
  Revision: `f4685bf11ac466dd278449bcfe5fd014e94aa504`
  License: MIT
  Notes: repository-local adapted copy of the upstream ignore-pattern grammar, renamed to `ignore` so `.gitignore`, `.dockerignore`, `.npmignore` and similar files can share one runtime.

- `grammars/dockerfile/grammar.js`
- `grammars/dockerfile/queries/highlights.scm`
- `grammars/dockerfile/scanner.c`
  Source: [camdencheek/tree-sitter-dockerfile](https://github.com/camdencheek/tree-sitter-dockerfile)
  Revision: `971acdd908568b4531b0ba28a445bf0bb720aba5`
  License: MIT
  Notes: repository-local adapted copy of the upstream Dockerfile grammar/scanner/highlights.

- `grammars/dockerfile/queries/injections.scm`
  Sources:
  - [camdencheek/tree-sitter-dockerfile](https://github.com/camdencheek/tree-sitter-dockerfile)
  - [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  Revisions: not recorded during the initial import
  License: MIT
  Notes: repository-local injection query that routes Dockerfile shell-form instruction bodies into kat's shared Bash runtime. The exact upstream commits used as the starting point still need a later audit.

- `grammars/bash/grammar.js`
  Source: [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  Revision: `5d8a33249511ed8bcf6cf135b7b2a815c7a02d9b`
  License: MIT
  Notes: repository-local copy of the upstream grammar source.

- `grammars/bash/scanner.c`
  Source: [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  Revision: `8509e3229b863c255ab6b61f3bf74ad0bf14e8bc`
  License: MIT
  Notes: repository-local copy of the upstream scanner source. The vendored Bash grammar and scanner are not currently pinned to the same upstream commit.

- `grammars/bash/queries/highlights.scm`
  Sources:
  - [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  - [zed-industries/zed](https://github.com/zed-industries/zed)
  Revisions: not recorded during the initial import
  Licenses: MIT and Apache-2.0
  Notes: repository-local adapted highlights query for kat's terminal renderer and capture model.

- `grammars/bash/queries/injections.scm`
  Source: local integration logic initially built around [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local injection query for Bash comment blocks and interpreter-aware heredoc dispatch. This file has substantial local integration logic and should not be treated as a direct upstream copy.

- `grammars/fish/*`
  Source: [ram02z/tree-sitter-fish](https://github.com/ram02z/tree-sitter-fish)
  Revision: `fa2143f5d66a9eb6c007ba9173525ea7aaafe788`
  License: MIT
  Notes: repository-local copy of selected grammar sources only; generated parser artifacts are not stored.

- `grammars/zsh/grammar.js`
- `grammars/zsh/queries/highlights.scm`
- `grammars/zsh/scanner.c`
  Source: [georgeharker/tree-sitter-zsh](https://github.com/georgeharker/tree-sitter-zsh)
  Revision: `bd344c23a7683e293d077c6648e88f209782fedb`
  License: MIT
  Notes: repository-local copy of selected grammar sources only; generated parser artifacts are not stored.

- `grammars/zsh/queries/injections.scm`
  Sources:
  - [georgeharker/tree-sitter-zsh](https://github.com/georgeharker/tree-sitter-zsh)
  - [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  Revisions: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted query that extends kat's existing shell heredoc injection model to zsh. The exact upstream commits used as the starting point still need a later audit.

- `grammars/toml/*`
  Source: [tree-sitter/tree-sitter-toml](https://github.com/tree-sitter/tree-sitter-toml)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local copy of selected grammar sources only; generated parser artifacts are not stored. The exact upstream commit for the initial import still needs a later audit.

- `grammars/yaml/*`
  Source: [tree-sitter-grammars/tree-sitter-yaml](https://github.com/tree-sitter-grammars/tree-sitter-yaml)
  Revision: `4463985dfccc640f3d6991e3396a2047610cf5f8`
  License: MIT
  Notes: repository-local copy of selected grammar sources only; generated parser artifacts are not stored. `grammars/yaml/schema.core.c` is retained because the upstream scanner includes it directly at compile time.

- `grammars/hcl/grammar.js`
- `grammars/hcl/scanner.c`
  Source: [tree-sitter-grammars/tree-sitter-hcl](https://github.com/tree-sitter-grammars/tree-sitter-hcl)
  Revision: `fad991865fee927dd1de5e172fb3f08ac674d914`
  License: Apache-2.0
  Notes: repository-local adapted copy of the upstream HCL grammar/scanner, kept as a dedicated runtime for `.hcl` and `.nomad` files.

- `grammars/hcl/queries/highlights.scm`
  Sources:
  - [helix-editor/helix](https://github.com/helix-editor/helix/blob/035450a2de62142b4117c01b89fff3d4f1b4d51f/runtime/queries/hcl/highlights.scm)
  - [tree-sitter-grammars/tree-sitter-hcl](https://github.com/tree-sitter-grammars/tree-sitter-hcl)
  License: MPL-2.0 and Apache-2.0
  Notes: repository-local adapted HCL highlights query, starting from Helix's query and tuned for kat's capture/theme model.

- `grammars/rust/*`
  Source: [tree-sitter/tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)
  Revision: `b3e615de069beb04ff44f65ac52f7f03cff04438`
  License: MIT
  Notes: repository-local copy of the upstream grammar source.

- `grammars/rust/scanner.c`
  Source: [tree-sitter/tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local copy of the upstream scanner source. The exact upstream commit for the initial import still needs a later audit.

- `grammars/rust/queries/highlights.scm`
  Sources:
  - [tree-sitter/tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)
  - [zed-industries/zed](https://github.com/zed-industries/zed)
  Revisions: not recorded during the initial import
  Licenses: MIT and Apache-2.0
  Notes: repository-local adapted highlights query for kat's terminal renderer and Dracula-oriented capture model.

- `grammars/python/*`
  Source: [tree-sitter/tree-sitter-python](https://github.com/tree-sitter/tree-sitter-python)
  Revision: `62827156d01c74dc1538266344e788da74536b8a`
  License: MIT
  Notes: repository-local copy of selected grammar sources only; generated parser artifacts are not stored. This revision is intentionally pinned because it still uses `scanner.cc`, which is needed to validate the repository's C++ scanner build path. Current upstream has since rewritten the scanner in C.

- `grammars/go/grammar.js`
  Source: [tree-sitter/tree-sitter-go](https://github.com/tree-sitter/tree-sitter-go)
  Revision: `179ca03b3ac7da8ed7466fc4a8b6b445c3c968da`
  License: MIT
  Notes: repository-local copy of the upstream grammar source.

- `grammars/go/queries/highlights.scm`
- `grammars/go/queries/injections.scm`
  Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted copies of Zed's Go query assets, further tuned for kat's runtime reuse and terminal rendering.

- `grammars/gomod/grammar.js`
  Source: [camdencheek/tree-sitter-go-mod](https://github.com/camdencheek/tree-sitter-go-mod)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted grammar source for `go.mod`. The exact upstream commit for the initial import still needs a later audit.

- `grammars/gomod/queries/highlights.scm`
  Source: [camdencheek/tree-sitter-go-mod](https://github.com/camdencheek/tree-sitter-go-mod)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted highlights query for `go.mod`, further refined for kat's terminal rendering.

- `grammars/gowork/*`
  Sources:
  - [omertuc/tree-sitter-go-work](https://github.com/omertuc/tree-sitter-go-work)
  - [Go Modules Reference](https://go.dev/ref/mod#workspaces)
  Revisions: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted copy of the upstream grammar/query, kept as a dedicated runtime for `go.work` instead of mixing workspace files into the Go source runtime. The exact upstream commit for the grammar/query starting point still needs a later audit.

- `grammars/gosum/grammar.js`
  Source: [amaanq/tree-sitter-go-sum](https://github.com/amaanq/tree-sitter-go-sum)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted grammar source for `go.sum`. The exact upstream commit for the initial import still needs a later audit.

- `grammars/gosum/queries/highlights.scm`
  Source: [amaanq/tree-sitter-go-sum](https://github.com/amaanq/tree-sitter-go-sum)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted highlights query for `go.sum`, further refined for kat's terminal rendering.

- `grammars/graphql/grammar.js`
  Source: [joowani/tree-sitter-graphql](https://github.com/joowani/tree-sitter-graphql)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted grammar source. The exact upstream commit for the initial import still needs a later audit.

- `grammars/graphql/queries/highlights.scm`
  Source: repository-local query maintained against the node structure of [joowani/tree-sitter-graphql](https://github.com/joowani/tree-sitter-graphql)
  Revision: not recorded during the initial import
  License: MIT
  Notes: highlights query maintained in-repo and tuned for kat's renderer/runtime model.

- `grammars/sql/*`
  Source: [nervenes/tree-sitter-sql](https://github.com/nervenes/tree-sitter-sql)
  Revision: `6dfca8b6dcb196d943c10e9cabab25e60232d332`
  License: MIT
  Notes: repository-local copy of selected grammar sources only; generated parser artifacts are not stored.

- `grammars/html/grammar.js`
  Source: [tree-sitter/tree-sitter-html](https://github.com/tree-sitter/tree-sitter-html)
  Revision: `266f2e4785f4a66dd22b46ad39f8b4d332a682bb`
  License: MIT
  Notes: repository-local copy of the upstream grammar source.

- `grammars/html/scanner.c`
  Source: [tree-sitter/tree-sitter-html](https://github.com/tree-sitter/tree-sitter-html)
  Revision: `eee29ca0ce287b699d9d52118f9c1cd9094c48f9`
  License: MIT
  Notes: repository-local copy of the upstream scanner source.

- `grammars/html/tag.h`
  Source: [tree-sitter/tree-sitter-html](https://github.com/tree-sitter/tree-sitter-html)
  Revision: `bfa075d83c6b97cd48440b3829ab8d24a2319809`
  License: MIT
  Notes: repository-local copy of the upstream scanner support header.

- `grammars/html/queries/highlights.scm`
- `grammars/html/queries/injections.scm`
  Source: [tree-sitter/tree-sitter-html](https://github.com/tree-sitter/tree-sitter-html)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted query assets for kat's terminal renderer and nested-runtime model.

- `grammars/css/grammar.js`
  Source: [tree-sitter/tree-sitter-css](https://github.com/tree-sitter/tree-sitter-css)
  Revision: `4a9aab1668bf13d024710420648ef9a9ee6ccc17`
  License: MIT
  Notes: repository-local copy of the upstream grammar source.

- `grammars/css/scanner.c`
  Source: [tree-sitter/tree-sitter-css](https://github.com/tree-sitter/tree-sitter-css)
  Revision: `a1ca8a4921d13130f3b15118e0112da882f835ea`
  License: MIT
  Notes: repository-local copy of the upstream scanner source.

- `grammars/css/queries/highlights.scm`
  Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted copy of Zed's CSS highlights query.

- `grammars/javascript/grammar.js`
  Source: [tree-sitter/tree-sitter-javascript](https://github.com/tree-sitter/tree-sitter-javascript)
  Revision: `39798e26b6d4dbcee8e522b8db83f8b2df33a5ea`
  License: MIT
  Notes: repository-local copy of the upstream grammar source.

- `grammars/javascript/scanner.c`
  Source: [tree-sitter/tree-sitter-javascript](https://github.com/tree-sitter/tree-sitter-javascript)
  Revision: `ee3bc5af628b6f899ff56a7fddcb95e0266dec2c`
  License: MIT
  Notes: repository-local copy of the upstream scanner source.

- `grammars/javascript/queries/locals.scm`
  Source: [tree-sitter/tree-sitter-javascript](https://github.com/tree-sitter/tree-sitter-javascript)
  Revision: `9802cc5812a19cd28168076af36e88b463dd3a18`
  License: MIT
  Notes: repository-local copy of the upstream locals query.

- `grammars/javascript/queries/highlights.scm`
  Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted copy of Zed's JavaScript highlights query, further adjusted for kat's vendored grammar revision and terminal renderer.

- `grammars/regex/grammar.js`
  Source: [tree-sitter/tree-sitter-regex](https://github.com/tree-sitter/tree-sitter-regex)
  Revision: `b2ac15e27fce703d2f37a79ccd94a5c0cbe9720b`
  License: MIT
  Notes: repository-local copy of selected grammar sources only; generated parser artifacts are not stored.

- `grammars/regex/queries/highlights.scm`
- `grammars/regex/queries/highlights-javascript.scm`
- `grammars/regex/queries/highlights-python.scm`
- `grammars/regex/queries/highlights-rust.scm`
- `grammars/regex/queries/highlights-go.scm`
- `grammars/regex/queries/highlights-posix.scm`
  Sources:
  - [tree-sitter/tree-sitter-regex](https://github.com/tree-sitter/tree-sitter-regex)
  - [zed-industries/zed](https://github.com/zed-industries/zed)
  Revisions: not recorded during the initial import
  Licenses: MIT and Apache-2.0
  Notes: repository-local highlight queries and host-aware overlay queries, adapted for kat's shared parser plus runtime-family model. These files include local integration changes and invalid-construct overlays for different regex hosts. The exact upstream commits used as the starting point still need a later audit.

- `grammars/jsdoc/grammar.js`
  Source: [tree-sitter/tree-sitter-jsdoc](https://github.com/tree-sitter/tree-sitter-jsdoc)
  Revision: `658d18dcdddb75c760363faa4963427a7c6b52db`
  License: MIT
  Notes: repository-local copy of the upstream grammar source.

- `grammars/jsdoc/scanner.c`
  Source: [tree-sitter/tree-sitter-jsdoc](https://github.com/tree-sitter/tree-sitter-jsdoc)
  Revision: `658d18dcdddb75c760363faa4963427a7c6b52db`
  License: MIT
  Notes: repository-local copy of the upstream scanner source.

- `grammars/jsdoc/queries/highlights.scm`
  Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted copy of Zed's JSDoc highlights query.

- `grammars/markdown/*`, `grammars/markdown_inline/*`
  Source: [tree-sitter-grammars/tree-sitter-markdown](https://github.com/tree-sitter-grammars/tree-sitter-markdown)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local copy of selected grammar sources only; generated parser artifacts are not stored. The exact upstream commit for the initial import still needs a later audit. `grammars/markdown/common.js` and `grammars/markdown/html_entities.json` are copied from the upstream shared support files because the block and inline Markdown grammars depend on them at build time.

- `grammars/just/grammar.js`
  Source: [IndianBoy42/tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just)
  Revision: `d9da862c156020c1a83d3c6ccdda32be6d8a5d4a`
  License: Apache-2.0
  Notes: repository-local copy of the upstream grammar source.

- `grammars/just/scanner.c`
  Source: [IndianBoy42/tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local copy of the upstream scanner source. The exact upstream commit for the initial import still needs a later audit.

- `grammars/just/queries/highlights.scm`
  Source: [IndianBoy42/tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just)
  Revision: `7333f8c150aaac5bb46decc2d225a2d4cde8c51e`
  License: Apache-2.0
  Notes: repository-local copy of the upstream generated highlights query.

- `grammars/just/queries/injections.scm`
  Source: [IndianBoy42/tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted injections query. The vendored file is not currently pinned to a verified upstream commit.

- `grammars/regex/queries/highlights.scm`
- `grammars/python/queries/injections.scm`
  Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted copies of selected query files. The exact upstream commit for the initial import still needs a later audit.

- `grammars/javascript/queries/injections.scm`
- `grammars/rust/queries/injections.scm`
  Sources:
  - [tree-sitter/tree-sitter-javascript](https://github.com/tree-sitter/tree-sitter-javascript)
  - [tree-sitter/tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)
  - [zed-industries/zed](https://github.com/zed-industries/zed)
  Revisions: not recorded during the initial import
  Licenses: MIT and Apache-2.0
  Notes: repository-local query files that include local integration changes plus adapted patterns from Zed. The exact upstream commits used as the starting point still need a later audit.

These files may be modified locally as needed for integration into `kat`.
