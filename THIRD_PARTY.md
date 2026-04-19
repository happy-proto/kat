# Third-Party Sources

This repository now primarily keeps Tree-sitter query and other kat-side integration assets. Parser source files that still need vendored maintenance are externalized to [kat-parsers](https://github.com/happy-proto/kat-parsers), while this repository only retains the local assets required for runtime integration.

Included sources:
- `grammars/actionscript/queries/highlights.scm`
- `grammars/ada/queries/highlights.scm`
- `grammars/ada/queries/locals.scm`
- `grammars/adp/queries/highlights.scm`
- `grammars/apache/queries/highlights.scm`
- `grammars/applescript/queries/highlights.scm`
- `grammars/asciidoc/queries/highlights.scm`
- `grammars/asm/queries/highlights.scm`
- `grammars/asp/queries/highlights.scm`
- `grammars/authorized_keys/queries/highlights.scm`
- `grammars/awk/queries/highlights.scm`
- `grammars/bash/queries/highlights.scm`
- `grammars/bash/queries/injections.scm`
- `grammars/batch/queries/highlights.scm`
- `grammars/bibtex/queries/highlights.scm`
- `grammars/bibtex/queries/locals.scm`
- `grammars/c/queries/highlights.scm`
- `grammars/cabal/queries/highlights.scm`
- `grammars/cfml/queries/highlights.scm`
- `grammars/cfml/queries/injections.scm`
- `grammars/clojure/queries/highlights.scm`
- `grammars/cmake/queries/highlights.scm`
- `grammars/cmakecache/queries/highlights.scm`
- `grammars/coffeescript/queries/highlights.scm`
- `grammars/coffeescript/queries/injections.scm`
- `grammars/command_help/queries/highlights.scm`
- `grammars/cpp/queries/highlights.scm`
- `grammars/cpp/queries/injections.scm`
- `grammars/cpuinfo/queries/highlights.scm`
- `grammars/crontab/queries/highlights.scm`
- `grammars/crystal/queries/highlights.scm`
- `grammars/crystal/queries/injections.scm`
- `grammars/csharp/queries/highlights.scm`
- `grammars/css/queries/highlights.scm`
- `grammars/csv/queries/highlights.scm`
- `grammars/d/queries/highlights.scm`
- `grammars/d/queries/injections.scm`
- `grammars/dart/queries/highlights.scm`
- `grammars/dart/queries/locals.scm`
- `grammars/debsources/queries/highlights.scm`
- `grammars/diff/queries/highlights.scm`
- `grammars/dockerfile/queries/highlights.scm`
- `grammars/dockerfile/queries/injections.scm`
- `grammars/dot/queries/highlights.scm`
- `grammars/dot/queries/injections.scm`
- `grammars/dotenv/queries/highlights.scm`
- `grammars/eex/queries/highlights.scm`
- `grammars/elixir/queries/highlights.scm`
- `grammars/elixir/queries/injections.scm`
- `grammars/elm/queries/highlights.scm`
- `grammars/elm/queries/injections.scm`
- `grammars/elm/queries/locals.scm`
- `grammars/email/queries/highlights.scm`
- `grammars/erb/queries/highlights.scm`
- `grammars/erlang/queries/highlights.scm`
- `grammars/fish/queries/highlights.scm`
- `grammars/fortran/queries/highlights.scm`
- `grammars/fortran_namelist/queries/highlights.scm`
- `grammars/fsharp/queries/highlights.scm`
- `grammars/fsharp/queries/injections.scm`
- `grammars/fsharp/queries/locals.scm`
- `grammars/fsharp_signature/queries/highlights.scm`
- `grammars/fsharp_signature/queries/injections.scm`
- `grammars/fsharp_signature/queries/locals.scm`
- `grammars/fstab/queries/highlights.scm`
- `grammars/git_commit/queries/highlights.scm`
- `grammars/git_config/queries/highlights.scm`
- `grammars/git_link/queries/highlights.scm`
- `grammars/git_log/queries/highlights.scm`
- `grammars/git_log/queries/injections.scm`
- `grammars/git_mailmap/queries/highlights.scm`
- `grammars/git_rebase/queries/highlights.scm`
- `grammars/gitattributes/queries/highlights.scm`
- `grammars/go/queries/highlights.scm`
- `grammars/go/queries/injections.scm`
- `grammars/gomod/queries/highlights.scm`
- `grammars/gosum/queries/highlights.scm`
- `grammars/gowork/queries/highlights.scm`
- `grammars/graphql/queries/highlights.scm`
- `grammars/groovy/queries/highlights.scm`
- `grammars/hcl/queries/highlights.scm`
- `grammars/html/queries/highlights.scm`
- `grammars/html/queries/injections.scm`
- `grammars/ignore/queries/highlights.scm`
- `grammars/ini/queries/highlights.scm`
- `grammars/java/queries/highlights.scm`
- `grammars/javascript/queries/highlights.scm`
- `grammars/javascript/queries/injections.scm`
- `grammars/javascript/queries/locals.scm`
- `grammars/jinja/queries/highlights.scm`
- `grammars/jq/queries/highlights.scm`
- `grammars/jsdoc/queries/highlights.scm`
- `grammars/json/queries/highlights.scm`
- `grammars/jsp/queries/highlights.scm`
- `grammars/just/queries/highlights.scm`
- `grammars/just/queries/injections.scm`
- `grammars/kotlin/queries/highlights.scm`
- `grammars/latex/queries/highlights.scm`
- `grammars/latex/queries/injections.scm`
- `grammars/less/queries/highlights.scm`
- `grammars/lua/queries/highlights.scm`
- `grammars/lua/queries/injections.scm`
- `grammars/lua/queries/locals.scm`
- `grammars/make/queries/highlights.scm`
- `grammars/markdown/queries/highlights.scm`
- `grammars/markdown/queries/injections.scm`
- `grammars/markdown_inline/queries/highlights.scm`
- `grammars/markdown_inline/queries/injections.scm`
- `grammars/nasm/queries/highlights.scm`
- `grammars/nginx/queries/highlights.scm`
- `grammars/nginx/queries/injections.scm`
- `grammars/ninja/queries/highlights.scm`
- `grammars/nix/queries/highlights.scm`
- `grammars/nix/queries/injections.scm`
- `grammars/php/queries/highlights.scm`
- `grammars/php/queries/injections-text.scm`
- `grammars/php/queries/injections.scm`
- `grammars/powershell/queries/highlights.scm`
- `grammars/properties/queries/highlights.scm`
- `grammars/proto/queries/highlights.scm`
- `grammars/python/queries/highlights.scm`
- `grammars/python/queries/injections.scm`
- `grammars/query/queries/highlights.scm`
- `grammars/regex/queries/highlights-go.scm`
- `grammars/regex/queries/highlights-javascript.scm`
- `grammars/regex/queries/highlights-posix.scm`
- `grammars/regex/queries/highlights-python.scm`
- `grammars/regex/queries/highlights-rust.scm`
- `grammars/regex/queries/highlights.scm`
- `grammars/registry.toml`
- `grammars/requirements/queries/highlights.scm`
- `grammars/ruby/queries/highlights.scm`
- `grammars/ruby/queries/locals.scm`
- `grammars/rust/queries/highlights.scm`
- `grammars/rust/queries/injections.scm`
- `grammars/sass/queries/highlights.scm`
- `grammars/scala/queries/highlights.scm`
- `grammars/scala/queries/locals.scm`
- `grammars/scss/queries/highlights.scm`
- `grammars/sml/queries/highlights.scm`
- `grammars/solidity/queries/highlights.scm`
- `grammars/sql/queries/highlights-mysql.scm`
- `grammars/sql/queries/highlights-postgres.scm`
- `grammars/sql/queries/highlights-sqlite.scm`
- `grammars/sql/queries/highlights.scm`
- `grammars/ssh_config/queries/highlights.scm`
- `grammars/ssh_config/queries/injections.scm`
- `grammars/strace/queries/highlights.scm`
- `grammars/stylus/queries/highlights.scm`
- `grammars/svelte/queries/highlights.scm`
- `grammars/svelte/queries/injections.scm`
- `grammars/swift/queries/highlights.scm`
- `grammars/swift/queries/injections.scm`
- `grammars/swift/queries/locals.scm`
- `grammars/syslog/queries/highlights.scm`
- `grammars/systemverilog/queries/highlights.scm`
- `grammars/tcl/queries/highlights.scm`
- `grammars/textile/queries/highlights.scm`
- `grammars/textproto/queries/highlights.scm`
- `grammars/todotxt/queries/highlights.scm`
- `grammars/toml/queries/highlights.scm`
- `grammars/tsv/queries/highlights.scm`
- `grammars/tsx/queries/highlights.scm`
- `grammars/tsx/queries/injections.scm`
- `grammars/tsx/queries/locals.scm`
- `grammars/twig/queries/highlights.scm`
- `grammars/typescript/queries/highlights.scm`
- `grammars/typescript/queries/injections.scm`
- `grammars/typescript/queries/locals.scm`
- `grammars/typst/queries/highlights.scm`
- `grammars/typst/queries/injections.scm`
- `grammars/userscript_metadata/queries/highlights.scm`
- `grammars/varlink/queries/highlights.scm`
- `grammars/verilog/queries/highlights.scm`
- `grammars/vhdl/queries/highlights.scm`
- `grammars/vim/queries/highlights.scm`
- `grammars/vim/queries/injections.scm`
- `grammars/vimhelp/queries/highlights.scm`
- `grammars/vimhelp/queries/injections.scm`
- `grammars/vue/queries/highlights.scm`
- `grammars/vue/queries/injections.scm`
- `grammars/vyper/queries/highlights.scm`
- `grammars/wgsl/queries/highlights.scm`
- `grammars/xml/queries/highlights.scm`
- `grammars/yaml/queries/highlights.scm`
- `grammars/yaml/queries/injections.scm`
- `grammars/zig/queries/highlights.scm`
- `grammars/zig/queries/injections.scm`
- `grammars/zig/queries/locals.scm`
- `grammars/zsh/queries/highlights.scm`
- `grammars/zsh/queries/injections.scm`

- `grammars/json/queries/highlights.scm`
- `grammars/query/queries/highlights.scm`
- `grammars/ignore/queries/highlights.scm`
- `grammars/git_config/queries/highlights.scm`
- `grammars/dockerfile/queries/highlights.scm`
- `grammars/dockerfile/queries/injections.scm`
- `grammars/bash/queries/highlights.scm`
- `grammars/bash/queries/injections.scm`
- `grammars/fish/queries/highlights.scm`
- `grammars/zsh/queries/highlights.scm`
- `grammars/zsh/queries/injections.scm`
- `grammars/powershell/queries/highlights.scm`
- `grammars/batch/queries/highlights.scm`
- `grammars/toml/queries/highlights.scm`
- `grammars/yaml/queries/highlights.scm`
- `grammars/hcl/queries/highlights.scm`
- `grammars/rust/queries/highlights.scm`
- `grammars/rust/queries/injections.scm`
- `grammars/python/queries/highlights.scm`
- `grammars/python/queries/injections.scm`
- `grammars/go/queries/highlights.scm`
- `grammars/go/queries/injections.scm`
- `grammars/gomod/queries/highlights.scm`
- `grammars/gowork/queries/highlights.scm`
- `grammars/gosum/queries/highlights.scm`
- `grammars/graphql/queries/highlights.scm`
- `grammars/proto/queries/highlights.scm`
- `grammars/textproto/queries/highlights.scm`
- `grammars/latex/queries/highlights.scm`
- `grammars/latex/queries/injections.scm`
- `grammars/tcl/queries/highlights.scm`
- `grammars/textile/queries/highlights.scm`
- `grammars/tsv/queries/highlights.scm`
- `grammars/typst/queries/highlights.scm`
- `grammars/typst/queries/injections.scm`
- `grammars/sql/queries/highlights.scm`
- `grammars/sql/queries/highlights-postgres.scm`
- `grammars/sql/queries/highlights-mysql.scm`
- `grammars/sql/queries/highlights-sqlite.scm`
- `grammars/html/queries/highlights.scm`
- `grammars/html/queries/injections.scm`
- `grammars/css/queries/highlights.scm`
- `grammars/javascript/queries/highlights.scm`
- `grammars/javascript/queries/injections.scm`
- `grammars/javascript/queries/locals.scm`
- `grammars/regex/queries/highlights.scm`
- `grammars/regex/queries/highlights-javascript.scm`
- `grammars/regex/queries/highlights-python.scm`
- `grammars/regex/queries/highlights-rust.scm`
- `grammars/regex/queries/highlights-go.scm`
- `grammars/regex/queries/highlights-posix.scm`
- `grammars/jsdoc/queries/highlights.scm`
- `grammars/markdown/queries/highlights.scm`
- `grammars/markdown/queries/injections.scm`
- `grammars/markdown_inline/queries/highlights.scm`
- `grammars/markdown_inline/queries/injections.scm`
- `grammars/just/queries/highlights.scm`
- `grammars/just/queries/injections.scm`
- `grammars/typescript/queries/highlights.scm`
- `grammars/typescript/queries/injections.scm`
- `grammars/typescript/queries/locals.scm`
- `grammars/tsx/queries/highlights.scm`
- `grammars/tsx/queries/injections.scm`
- `grammars/tsx/queries/locals.scm`
- `grammars/vue/queries/highlights.scm`
- `grammars/vue/queries/injections.scm`
- `grammars/svelte/queries/highlights.scm`
- `grammars/svelte/queries/injections.scm`
- `grammars/dotenv/queries/highlights.scm`
- `grammars/ini/queries/highlights.scm`
- `grammars/xml/queries/highlights.scm`
- `grammars/make/queries/highlights.scm`
- `grammars/cmake/queries/highlights.scm`
- `grammars/ninja/queries/highlights.scm`
- `grammars/jinja/queries/highlights.scm`
- `grammars/eex/queries/highlights.scm`
- `grammars/twig/queries/highlights.scm`
- `grammars/erb/queries/highlights.scm`
- `grammars/jsp/queries/highlights.scm`
- `grammars/asp/queries/highlights.scm`
- `grammars/adp/queries/highlights.scm`
- `grammars/php/queries/highlights.scm`
- `grammars/php/queries/injections.scm`
- `grammars/php/queries/injections-text.scm`
- `grammars/scala/queries/highlights.scm`
- `grammars/scala/queries/locals.scm`
- `grammars/swift/queries/highlights.scm`
- `grammars/swift/queries/injections.scm`
- `grammars/swift/queries/locals.scm`
- `grammars/dart/queries/highlights.scm`
- `grammars/dart/queries/locals.scm`
- `grammars/elixir/queries/highlights.scm`
- `grammars/elixir/queries/injections.scm`
- `grammars/zig/queries/highlights.scm`
- `grammars/zig/queries/injections.scm`
- `grammars/zig/queries/locals.scm`
- `grammars/ssh_config/queries/highlights.scm`
- `grammars/ssh_config/queries/injections.scm`
- `grammars/gitattributes/queries/highlights.scm`
- `grammars/git_commit/queries/highlights.scm`
- `grammars/git_rebase/queries/highlights.scm`
- `grammars/requirements/queries/highlights.scm`
- `grammars/apache/queries/highlights.scm`
- `grammars/actionscript/queries/highlights.scm`
- `grammars/ada/queries/highlights.scm`
- `grammars/ada/queries/locals.scm`
- `grammars/applescript/queries/highlights.scm`
- `grammars/asm/queries/highlights.scm`
- `grammars/nasm/queries/highlights.scm`
- `grammars/asciidoc/queries/highlights.scm`
- `grammars/authorized_keys/queries/highlights.scm`
- `grammars/awk/queries/highlights.scm`
- `grammars/bibtex/queries/highlights.scm`
- `grammars/bibtex/queries/locals.scm`
- `grammars/cabal/queries/highlights.scm`
- `grammars/cfml/queries/highlights.scm`
- `grammars/cfml/queries/injections.scm`
- `grammars/clojure/queries/highlights.scm`
- `grammars/cmakecache/queries/highlights.scm`
- `grammars/coffeescript/queries/highlights.scm`
- `grammars/coffeescript/queries/injections.scm`
- `grammars/command_help/queries/highlights.scm`
- `grammars/cpuinfo/queries/highlights.scm`
- `grammars/crontab/queries/highlights.scm`
- `grammars/crystal/queries/highlights.scm`
- `grammars/crystal/queries/injections.scm`
- `grammars/d/queries/highlights.scm`
- `grammars/d/queries/injections.scm`
- `grammars/debsources/queries/highlights.scm`
- `grammars/elm/queries/highlights.scm`
- `grammars/elm/queries/injections.scm`
- `grammars/elm/queries/locals.scm`
- `grammars/email/queries/highlights.scm`
- `grammars/erlang/queries/highlights.scm`
- `grammars/fortran/queries/highlights.scm`
- `grammars/fortran_namelist/queries/highlights.scm`
- `grammars/fsharp/queries/highlights.scm`
- `grammars/fsharp/queries/injections.scm`
- `grammars/fsharp/queries/locals.scm`
- `grammars/fsharp_signature/queries/highlights.scm`
- `grammars/fsharp_signature/queries/injections.scm`
- `grammars/fsharp_signature/queries/locals.scm`
- `grammars/fstab/queries/highlights.scm`
- `grammars/scss/queries/highlights.scm`
- `grammars/sass/queries/highlights.scm`
- `grammars/todotxt/queries/highlights.scm`
- `grammars/vhdl/queries/highlights.scm`
- `grammars/vim/queries/highlights.scm`
- `grammars/vim/queries/injections.scm`
- `grammars/csharp/queries/highlights.scm`
- `grammars/groovy/queries/highlights.scm`
- `grammars/diff/queries/highlights.scm`
- `grammars/properties/queries/highlights.scm`
- `grammars/jq/queries/highlights.scm`
- `grammars/less/queries/highlights.scm`
- `grammars/dot/queries/highlights.scm`
- `grammars/dot/queries/injections.scm`
- `grammars/nginx/queries/highlights.scm`
- `grammars/nginx/queries/injections.scm`

Upstream projects:

- `grammars/json/queries/highlights.scm`
  Sources:
  - [tree-sitter/tree-sitter-json](https://github.com/tree-sitter/tree-sitter-json)
  - [zed-industries/zed](https://github.com/zed-industries/zed)
  Revisions: not recorded during the initial import
  Licenses: MIT and Apache-2.0
  Notes: repository-local adapted highlights query for kat's terminal renderer and Dracula-oriented capture model. JSON parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-json](https://crates.io/crates/tree-sitter-json).

- `grammars/query/queries/highlights.scm`
  Source: local integration query built around [nvim-treesitter/tree-sitter-query](https://github.com/nvim-treesitter/tree-sitter-query)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/query/` directory now only keeps kat-side integration assets such as queries.

- `grammars/ignore/*`
  Source: [shunsambongi/tree-sitter-gitignore](https://github.com/shunsambongi/tree-sitter-gitignore)
  Revision: `f4685bf11ac466dd278449bcfe5fd014e94aa504`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/ignore/` directory now only keeps kat-side integration assets such as queries.

- `grammars/git_config/*`
  Source: [the-mikedavis/tree-sitter-git-config](https://github.com/the-mikedavis/tree-sitter-git-config)
  Revision: `0fbc9f99d5a28865f9de8427fb0672d66f9d83a5`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/git_config/` directory now only keeps kat-side integration assets such as queries.

- `grammars/dockerfile/queries/highlights.scm`
  Source: [camdencheek/tree-sitter-dockerfile](https://github.com/camdencheek/tree-sitter-dockerfile)
  Revision: `971acdd908568b4531b0ba28a445bf0bb720aba5`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/dockerfile/` directory now only keeps kat-side integration assets such as queries.

- `grammars/dockerfile/queries/injections.scm`
  Sources:
  - [camdencheek/tree-sitter-dockerfile](https://github.com/camdencheek/tree-sitter-dockerfile)
  - [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  Revisions: not recorded during the initial import
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/dockerfile/` directory now only keeps kat-side integration assets such as queries.

- `grammars/bash/queries/highlights.scm`
  Sources:
  - [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  - [zed-industries/zed](https://github.com/zed-industries/zed)
  Revisions: not recorded during the initial import
  Licenses: MIT and Apache-2.0
  Notes: repository-local adapted highlights query for kat's terminal renderer and capture model. Bash parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-bash](https://crates.io/crates/tree-sitter-bash).

- `grammars/bash/queries/injections.scm`
  Source: local integration logic initially built around [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local injection query for Bash comment blocks and interpreter-aware heredoc dispatch. This file has substantial local integration logic and should not be treated as a direct upstream copy.

- `grammars/fish/*`
  Source: [ram02z/tree-sitter-fish](https://github.com/ram02z/tree-sitter-fish)
  Revision: `fa2143f5d66a9eb6c007ba9173525ea7aaafe788`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/fish/` directory now only keeps kat-side integration assets such as queries.

- `grammars/zsh/queries/highlights.scm`
  Source: [georgeharker/tree-sitter-zsh](https://github.com/georgeharker/tree-sitter-zsh)
  Revision: `bd344c23a7683e293d077c6648e88f209782fedb`
  License: MIT
  Notes: repository-local adapted highlights query. Zsh parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-zsh](https://crates.io/crates/tree-sitter-zsh).

- `grammars/zsh/queries/injections.scm`
  Sources:
  - [georgeharker/tree-sitter-zsh](https://github.com/georgeharker/tree-sitter-zsh)
  - [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  Revisions: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted query that extends kat's existing shell heredoc injection model to zsh. The exact upstream commits used as the starting point still need a later audit.

- `grammars/powershell/queries/highlights.scm`
  Source: [airbus-cert/tree-sitter-powershell](https://github.com/airbus-cert/tree-sitter-powershell)
  Revision: `da65ba3acc93777255781b447f5e7448245df4bf`
  License: MIT
  Notes: repository-local copy of the upstream PowerShell highlights query. Parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-powershell](https://crates.io/crates/tree-sitter-powershell).

- `grammars/batch/queries/highlights.scm`
  Source: [wharflab/tree-sitter-batch](https://github.com/wharflab/tree-sitter-batch)
  Revision: `8694fbc701ff6e35e3711bf39225860d13079906`
  License: MIT
  Notes: repository-local copy of the upstream Windows Batch / CMD highlights query. Parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-batch](https://crates.io/crates/tree-sitter-batch).

- `grammars/toml/queries/highlights.scm`
  Sources:
  - [helix-editor/helix](https://github.com/helix-editor/helix/blob/035450a2de62142b4117c01b89fff3d4f1b4d51f/runtime/queries/toml/highlights.scm)
  - [tree-sitter-grammars/tree-sitter-toml](https://github.com/tree-sitter-grammars/tree-sitter-toml)
  Revisions: query starting point not fully audited
  Licenses: MPL-2.0 and MIT
  Notes: repository-local adapted highlights query for kat's terminal renderer and capture model. TOML parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-toml-ng](https://crates.io/crates/tree-sitter-toml-ng).

- `grammars/yaml/queries/highlights.scm`
  Source: [tree-sitter-grammars/tree-sitter-yaml](https://github.com/tree-sitter-grammars/tree-sitter-yaml)
  Revision: `4463985dfccc640f3d6991e3396a2047610cf5f8`
  License: MIT
  Notes: repository-local copy of the upstream YAML highlights query. Parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-yaml](https://crates.io/crates/tree-sitter-yaml).

- `grammars/hcl/queries/highlights.scm`
  Sources:
  - [helix-editor/helix](https://github.com/helix-editor/helix/blob/035450a2de62142b4117c01b89fff3d4f1b4d51f/runtime/queries/hcl/highlights.scm)
  - [tree-sitter-grammars/tree-sitter-hcl](https://github.com/tree-sitter-grammars/tree-sitter-hcl)
  License: MPL-2.0 and Apache-2.0
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/hcl/` directory now only keeps kat-side integration assets such as queries.

- `grammars/rust/queries/highlights.scm`
  Sources:
  - [tree-sitter/tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)
  - [zed-industries/zed](https://github.com/zed-industries/zed)
  Revisions: not recorded during the initial import
  Licenses: MIT and Apache-2.0
  Notes: repository-local adapted highlights query for kat's terminal renderer and Dracula-oriented capture model. Rust parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-rust](https://crates.io/crates/tree-sitter-rust).

- `grammars/python/queries/highlights.scm`
  Source: [tree-sitter/tree-sitter-python](https://github.com/tree-sitter/tree-sitter-python)
  Revision: `62827156d01c74dc1538266344e788da74536b8a`
  License: MIT
  Notes: repository-local copy of the upstream Python highlights query. Parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-python](https://crates.io/crates/tree-sitter-python).

- `grammars/go/queries/highlights.scm`
- `grammars/go/queries/injections.scm`
  Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted copies of Zed's Go query assets, further tuned for kat's runtime reuse and terminal rendering. Go parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-go](https://crates.io/crates/tree-sitter-go).

- `grammars/gomod/queries/highlights.scm`
  Source: [camdencheek/tree-sitter-go-mod](https://github.com/camdencheek/tree-sitter-go-mod)
  Revision: not recorded during the initial import
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/gomod/` directory now only keeps kat-side integration assets such as queries.

- `grammars/gowork/*`
  Sources:
  - [omertuc/tree-sitter-go-work](https://github.com/omertuc/tree-sitter-go-work)
  - [Go Modules Reference](https://go.dev/ref/mod#workspaces)
  Revisions: not recorded during the initial import
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/gowork/` directory now only keeps kat-side integration assets such as queries.

- `grammars/gosum/queries/highlights.scm`
  Source: [amaanq/tree-sitter-go-sum](https://github.com/amaanq/tree-sitter-go-sum)
  Revision: not recorded during the initial import
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/gosum/` directory now only keeps kat-side integration assets such as queries.

- `grammars/graphql/queries/highlights.scm`
  Source: repository-local query maintained against the node structure of [joowani/tree-sitter-graphql](https://github.com/joowani/tree-sitter-graphql)
  Revision: not recorded during the initial import
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/graphql/` directory now only keeps kat-side integration assets such as queries.

- `grammars/proto/queries/highlights.scm`
  Source: [mitchellh/tree-sitter-proto](https://github.com/mitchellh/tree-sitter-proto)
  Revision: `42d82fa18f8afe59b5fc0b16c207ee4f84cb185f`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/proto/` directory now only keeps kat-side integration assets such as queries.

- `grammars/textproto/queries/highlights.scm`
  Source: [PorterAtGoogle/tree-sitter-textproto](https://github.com/PorterAtGoogle/tree-sitter-textproto)
  Revision: `568471b80fd8793d37ed01865d8c2208a9fefd1b`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/textproto/` directory now only keeps kat-side integration assets such as queries.

- `grammars/latex/queries/highlights.scm`
- `grammars/latex/queries/injections.scm`
  Source: [nvim-treesitter/nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter)
  Revision: `cf12346a3414fa1b06af75c79faebe7f76df080a`
  License: Apache-2.0
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/latex/` directory now only keeps kat-side integration assets such as queries.

- `grammars/tcl/*`
  Source: [tree-sitter-grammars/tree-sitter-tcl](https://github.com/tree-sitter-grammars/tree-sitter-tcl)
  Revision: `8f11ac7206a54ed11210491cee1e0657e2962c47`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/tcl/` directory now only keeps kat-side integration assets such as queries.

- `grammars/textile/*`
  Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/textile/` directory now only keeps kat-side integration assets such as queries.

- `grammars/tsv/queries/highlights.scm`
  Source: [tree-sitter-grammars/tree-sitter-csv](https://github.com/tree-sitter-grammars/tree-sitter-csv)
  Revision: `f6bf6e35eb0b95fbadea4bb39cb9709507fcb181`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/tsv/` directory now only keeps kat-side integration assets such as queries.

- `grammars/typst/*`
  Source: [uben0/tree-sitter-typst](https://github.com/uben0/tree-sitter-typst)
  Revision: `46cf4ded12ee974a70bf8457263b67ad7ee0379d`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/typst/` directory now only keeps kat-side integration assets such as queries.

- `grammars/sml/*`
  Source: [matthew-fluet/tree-sitter-sml](https://github.com/matthew-fluet/tree-sitter-sml)
  Revision: `fd4b4955bb998262840ab8119885b3edf20ea75a`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/sml/` directory now only keeps kat-side integration assets such as queries.

- `grammars/solidity/queries/highlights.scm`
  Source: [JoranHonig/tree-sitter-solidity](https://github.com/JoranHonig/tree-sitter-solidity)
  Revision: `048fe686cb1fde267243739b8bdbec8fc3a55272`
  License: MIT
  Notes: repository-local adapted highlights query. The Solidity parser is provided by the Rust crate [tree-sitter-solidity](https://crates.io/crates/tree-sitter-solidity).

- `grammars/strace/*`
  Source: [sigmaSd/tree-sitter-strace](https://github.com/sigmaSd/tree-sitter-strace)
  Revision: `ac874ddfcc08d689fee1f4533789e06d88388f29`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/strace/` directory now only keeps kat-side integration assets such as queries.

- `grammars/systemverilog/queries/highlights.scm`
  Source: [gmlarumbe/tree-sitter-systemverilog](https://github.com/gmlarumbe/tree-sitter-systemverilog)
  Revision: `293928578cb27fbd0005fcc5f09c09a1e8628c89`
  License: MIT
  Notes: repository-local adapted highlights query. The SystemVerilog parser is provided by the Rust crate [tree-sitter-systemverilog](https://crates.io/crates/tree-sitter-systemverilog).

- `grammars/varlink/queries/highlights.scm`
  Source: [M0ppers/tree-sitter-varlink](https://github.com/M0ppers/tree-sitter-varlink)
  Revision: `52976e66d3f4529045a14201841e4dc289de8107`
  License: MIT
  Notes: repository-local adapted highlights query. The Varlink parser is provided by the Rust crate [tree-sitter-varlink](https://crates.io/crates/tree-sitter-varlink).

- `grammars/vimhelp/*`
  Source: [neovim/tree-sitter-vimdoc](https://github.com/neovim/tree-sitter-vimdoc)
  Revision: `f061895a0eff1d5b90e4fb60d21d87be3267031a`
  License: Apache-2.0
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/vimhelp/` directory now only keeps kat-side integration assets such as queries.

- `grammars/sql/queries/highlights.scm`
- `grammars/sql/queries/highlights-postgres.scm`
- `grammars/sql/queries/highlights-mysql.scm`
- `grammars/sql/queries/highlights-sqlite.scm`
  Source: [nervenes/tree-sitter-sql](https://github.com/nervenes/tree-sitter-sql)
  Revision: `6dfca8b6dcb196d943c10e9cabab25e60232d332`
  License: MIT
  Notes: repository-local adapted highlight queries. SQL parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-sequel](https://crates.io/crates/tree-sitter-sequel).

- `grammars/html/queries/highlights.scm`
- `grammars/html/queries/injections.scm`
  Source: [tree-sitter/tree-sitter-html](https://github.com/tree-sitter/tree-sitter-html)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted query assets for kat's terminal renderer and nested-runtime model. HTML parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-html](https://crates.io/crates/tree-sitter-html).

- `grammars/css/queries/highlights.scm`
  Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted CSS highlights query. Parser code is linked from the Rust crate [tree-sitter-css](https://crates.io/crates/tree-sitter-css).

- `grammars/javascript/queries/locals.scm`
  Source: [tree-sitter/tree-sitter-javascript](https://github.com/tree-sitter/tree-sitter-javascript)
  Revision: `9802cc5812a19cd28168076af36e88b463dd3a18`
  License: MIT
  Notes: repository-local copy of the upstream locals query.

- `grammars/javascript/queries/highlights.scm`
  Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted copy of Zed's JavaScript highlights query, further adjusted for kat's terminal renderer. JavaScript parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-javascript](https://crates.io/crates/tree-sitter-javascript).

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
  Notes: repository-local highlight queries and host-aware overlay queries, adapted for kat's shared parser plus runtime-family model. These files include local integration changes and invalid-construct overlays for different regex hosts. Regex parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-regex](https://crates.io/crates/tree-sitter-regex). The exact upstream commits used as the starting point still need a later audit.

- `grammars/jsdoc/queries/highlights.scm`
  Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted copy of Zed's JSDoc highlights query. JSDoc parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-jsdoc](https://crates.io/crates/tree-sitter-jsdoc).

- `grammars/markdown/*`, `grammars/markdown_inline/*`
  Source: [tree-sitter-grammars/tree-sitter-markdown](https://github.com/tree-sitter-grammars/tree-sitter-markdown)
  Revision: not recorded during the initial import
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/markdown/` directory now only keeps kat-side integration assets such as queries.

- `grammars/just/queries/highlights.scm`
  Source: [IndianBoy42/tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just)
  Revision: `7333f8c150aaac5bb46decc2d225a2d4cde8c51e`
  License: Apache-2.0
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/just/` directory now only keeps kat-side integration assets such as queries.

- `grammars/just/queries/injections.scm`
  Source: [IndianBoy42/tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/just/` directory now only keeps kat-side integration assets such as queries.

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

- `grammars/typescript/queries/highlights.scm`
- `grammars/typescript/queries/injections.scm`
- `grammars/tsx/queries/highlights.scm`
- `grammars/tsx/queries/injections.scm`
  Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: local working copy reference under `/Users/jun.fan/Code/zed`; exact upstream commit not yet recorded in this repository
  License: Apache-2.0
  Notes: repository-local adapted copies of Zed's TypeScript / TSX query files, reused to keep `kat`'s TypeScript-family captures aligned with the existing JavaScript / JSX integration model.

- `grammars/typescript/queries/locals.scm`
- `grammars/tsx/queries/locals.scm`
  Source: [tree-sitter/tree-sitter-typescript](https://github.com/tree-sitter/tree-sitter-typescript)
  Revision: `75b3874edb2dc714fb1fd77a32013d0f8699989f`
  License: MIT
  Notes: repository-local copies of the upstream locals query shared by the TypeScript and TSX runtimes. Parser code is linked from the Rust crate `tree-sitter-typescript`.

- `grammars/vue/*`
  Source: [ikatyang/tree-sitter-vue](https://github.com/ikatyang/tree-sitter-vue)
  Revision: `91fe2754796cd8fba5f229505a23fa08f3546c06`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/vue/` directory now only keeps kat-side integration assets such as queries.

- `grammars/svelte/*`
  Source: [Himujjal/tree-sitter-svelte](https://github.com/Himujjal/tree-sitter-svelte)
  Revision: `60ea1d673a1a3eeeb597e098d9ada9ed0c79ef4b`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/svelte/` directory now only keeps kat-side integration assets such as queries.

- `grammars/dotenv/*`
  Source: [pnx/tree-sitter-dotenv](https://github.com/pnx/tree-sitter-dotenv)
  Revision: `f3b1f1f20d255082f2fd4761f6961ab5cf01d4f4`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/dotenv/` directory now only keeps kat-side integration assets such as queries.

- `grammars/ini/queries/highlights.scm`
  Source: [justinmk/tree-sitter-ini](https://github.com/justinmk/tree-sitter-ini)
  Revision: `e4018b5176132b4f3c5d6e61cea383f42288d0f5`
  License: Apache-2.0
  Notes: repository-local copy of the upstream highlights query. Parser code is linked from the Rust crate `tree-sitter-ini`.

- `grammars/xml/queries/highlights.scm`
  Source: [tree-sitter-grammars/tree-sitter-xml](https://github.com/tree-sitter-grammars/tree-sitter-xml)
  Revision: `5000ae8f22d11fbe93939b05c1e37cf21117162d`
  License: MIT
  Notes: repository-local copy of the upstream XML highlights query. Parser code is linked from the Rust crate `tree-sitter-xml`.

- `grammars/make/queries/highlights.scm`
  Source: [tree-sitter-grammars/tree-sitter-make](https://github.com/tree-sitter-grammars/tree-sitter-make)
  Revision: `70613f3d812cbabbd7f38d104d60a409c4008b43`
  License: MIT
  Notes: repository-local copy of the upstream Makefile highlights query. Parser code is linked from the Rust crate `tree-sitter-make`.

- `grammars/cmake/queries/highlights.scm`
  Source: [uyha/tree-sitter-cmake](https://github.com/uyha/tree-sitter-cmake)
  Revision: `c7b2a71e7f8ecb167fad4c97227c838439280175`
  License: MIT
  Notes: repository-local copy of the upstream CMake highlights query. Parser code is linked from the Rust crate `tree-sitter-cmake`.

- `grammars/ninja/*`
  Source: [alemuller/tree-sitter-ninja](https://github.com/alemuller/tree-sitter-ninja)
  Revision: `0a95cfdc0745b6ae82f60d3a339b37f19b7b9267`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/ninja/` directory now only keeps kat-side integration assets such as queries.

- `grammars/jinja/*`
  Source: [cathaysia/tree-sitter-jinja](https://github.com/cathaysia/tree-sitter-jinja)
  Revision: `7e254abb76618227806f6881525980231faa1610`
  License: Apache-2.0
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/jinja/` directory now only keeps kat-side integration assets such as queries.

- `grammars/twig/*`
  Source: [kaermorchen/tree-sitter-twig](https://github.com/kaermorchen/tree-sitter-twig)
  Revision: `dac11024e40536d05c958d920139c310cbe86625`
  License: MPL-2.0
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/twig/` directory now only keeps kat-side integration assets such as queries.

- `grammars/eex/queries/highlights.scm`
- `grammars/erb/queries/highlights.scm`
- `grammars/jsp/queries/highlights.scm`
- `grammars/asp/queries/highlights.scm`
- `grammars/adp/queries/highlights.scm`
  Source: [tree-sitter/tree-sitter-embedded-template](https://github.com/tree-sitter/tree-sitter-embedded-template)
  Revision: `3499d85f0a0d937c507a4a65368f2f63772786e1`
  License: MIT
  Notes: repository-local adapted highlights queries for the shared embedded-template parser used by `ERB` / `EEx` / `JSP` / `ASP` / `ADP`. Parser code is linked from the Rust crate `tree-sitter-embedded-template`; host/content dispatch is handled by kat's document-profile + host-resolver layer.

- `grammars/php/queries/highlights.scm`
- `grammars/php/queries/injections.scm`
- `grammars/php/queries/injections-text.scm`
  Source: [tree-sitter/tree-sitter-php](https://github.com/tree-sitter/tree-sitter-php)
  Revision: `3f2465c217d0a966d41e584b42d75522f2a3149e`
  License: MIT
  Notes: repository-local copies of the upstream PHP query assets. Parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-php](https://crates.io/crates/tree-sitter-php).

- `grammars/scala/queries/highlights.scm`
- `grammars/scala/queries/locals.scm`
  Source: [tree-sitter/tree-sitter-scala](https://github.com/tree-sitter/tree-sitter-scala)
  Revision: `22af0ac923c90cef50a31085b27049d50c94c70f`
  License: MIT
  Notes: repository-local copies of the upstream Scala query assets. Parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-scala](https://crates.io/crates/tree-sitter-scala).

- `grammars/swift/queries/highlights.scm`
- `grammars/swift/queries/injections.scm`
- `grammars/swift/queries/locals.scm`
  Source: [alex-pinkus/tree-sitter-swift](https://github.com/alex-pinkus/tree-sitter-swift)
  Revision: `da7f9370b70ba31357122c211734db98eb6f6a35`
  License: MIT
  Notes: repository-local copies of the upstream Swift query assets. Parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-swift](https://crates.io/crates/tree-sitter-swift).

- `grammars/dart/queries/highlights.scm`
- `grammars/dart/queries/locals.scm`
  Source: [nielsenko/tree-sitter-dart](https://github.com/nielsenko/tree-sitter-dart)
  Revision: `316b9743b2d45b7e7b71fdbdb28e3e8971d64c13`
  License: MIT
  Notes: repository-local copies of the upstream Dart query assets. Parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-dart](https://crates.io/crates/tree-sitter-dart).

- `grammars/elixir/queries/highlights.scm`
- `grammars/elixir/queries/injections.scm`
  Source: [elixir-lang/tree-sitter-elixir](https://github.com/elixir-lang/tree-sitter-elixir)
  Revision: `7937d3b4d65fa574163cfa59394515d3c1cf16f4`
  License: Apache-2.0
  Notes: repository-local copies of the upstream Elixir query assets. Parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-elixir](https://crates.io/crates/tree-sitter-elixir).

- `grammars/zig/queries/highlights.scm`
- `grammars/zig/queries/injections.scm`
- `grammars/zig/queries/locals.scm`
  Source: [tree-sitter-grammars/tree-sitter-zig](https://github.com/tree-sitter-grammars/tree-sitter-zig)
  Revision: `6479aa13f32f701c383083d8b28360ebd682fb7d`
  License: MIT
  Notes: repository-local copies of the upstream Zig query assets. Parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-zig](https://crates.io/crates/tree-sitter-zig).

- `grammars/ssh_config/*`
  Source: [tree-sitter-grammars/tree-sitter-ssh-config](https://github.com/tree-sitter-grammars/tree-sitter-ssh-config)
  Revision: `71d2693deadaca8cdc09e38ba41d2f6042da1616`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/ssh_config/` directory now only keeps kat-side integration assets such as queries.

- `grammars/gitattributes/*`
  Source: [tree-sitter-grammars/tree-sitter-gitattributes](https://github.com/tree-sitter-grammars/tree-sitter-gitattributes)
  Revision: `1b7af09d45b579f9f288453b95ad555f1f431645`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/gitattributes/` directory now only keeps kat-side integration assets such as queries.

- `grammars/git_commit/*`
  Source: [the-mikedavis/tree-sitter-git-commit](https://github.com/the-mikedavis/tree-sitter-git-commit)
  Revision: `5a50da19b3841ac51e9d483cd9c856a85232233d`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/git_commit/` directory now only keeps kat-side integration assets such as queries.

- `grammars/git_rebase/*`
  Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/git_rebase/` directory now only keeps kat-side integration assets such as queries.

- `grammars/requirements/*`
  Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/requirements/` directory now only keeps kat-side integration assets such as queries.

- `grammars/apache/*`
  Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/apache/` directory now only keeps kat-side integration assets such as queries.

- `grammars/actionscript/*`
  Source: [jcs090218/tree-sitter-actionscript](https://github.com/jcs090218/tree-sitter-actionscript)
  Revision: `12fc0c4c822c6edd924c13b328a93fe69454b299`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/actionscript/` directory now only keeps kat-side integration assets such as queries.

- `grammars/ada/queries/highlights.scm`
- `grammars/ada/queries/locals.scm`
  Source: [briot/tree-sitter-ada](https://github.com/briot/tree-sitter-ada)
  Revision: `6b58259a08b1a22ba0247a7ce30be384db618da6`
  License: MIT
  Notes: repository-local copies of the upstream Ada query assets. Parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on `tree-sitter-ada`. `kat` intentionally stays on the git revision instead of crates.io `0.1.0`, because the pinned commit already includes newer Ada constructs such as `parallel_block_statement`, `procedural_iterator`, and `parallel for`-style iteration support that are not present in the published crate release.

- `grammars/applescript/*`
  Source: [waddie/tree-sitter-applescript](https://github.com/waddie/tree-sitter-applescript)
  Revision: `adff3f4de87033350050232c8dd23947c7b34850`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/applescript/` directory now only keeps kat-side integration assets such as queries.

- `grammars/asm/*`
  Source: [RubixDev/tree-sitter-asm](https://github.com/RubixDev/tree-sitter-asm)
  Revision: `839741fef4dab5128952334624905c82b40c7133`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/asm/` directory now only keeps kat-side integration assets such as queries.

- `grammars/nasm/*`
  Source: [naclsn/tree-sitter-nasm](https://github.com/naclsn/tree-sitter-nasm)
  Revision: `d1b3638d017f2a8585e26dcfc66fe1df94185e30`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/nasm/` directory now only keeps kat-side integration assets such as queries.

- `grammars/asciidoc/*`
  Source: [cpkio/tree-sitter-asciidoc](https://github.com/cpkio/tree-sitter-asciidoc)
  Revision: `a00a91dd44cd6c228f3bc10b3e548f651058e0db`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/asciidoc/` directory now only keeps kat-side integration assets such as queries.

- `grammars/authorized_keys/*`
  Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/authorized_keys/` directory now only keeps kat-side integration assets such as queries.

- `grammars/awk/*`
  Source: [Beaglefoot/tree-sitter-awk](https://github.com/Beaglefoot/tree-sitter-awk)
  Revision: `34bbdc7cce8e803096f47b625979e34c1be38127`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/awk/` directory now only keeps kat-side integration assets such as queries.

- `grammars/bibtex/*`
  Source: [latex-lsp/tree-sitter-bibtex](https://github.com/latex-lsp/tree-sitter-bibtex)
  Revision: `8d04ed27b3bc7929f14b7df9236797dab9f3fa66`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/bibtex/` directory now only keeps kat-side integration assets such as queries.

- `grammars/cabal/*`
  Source: [thomasvergne/tree-sitter-cabal](https://github.com/thomasvergne/tree-sitter-cabal)
  Revision: `1762ded13e5351c0bc662a2273d523b80d314b4e`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/cabal/` directory now only keeps kat-side integration assets such as queries.

- `grammars/cfml/queries/*`
  Source: [cfmleditor/tree-sitter-cfml](https://github.com/cfmleditor/tree-sitter-cfml)
  Revision: `4628d0be345c033330acc8e8b36d7c6eaf201c87`
  License: MIT
  Notes: parser now comes from the Rust crate [tree-sitter-cfml](https://crates.io/crates/tree-sitter-cfml); the repository only keeps the upstream highlights and injections queries as integration assets.

- `grammars/clojure/queries/highlights.scm`
  Source: [grammar-orchard/tree-sitter-clojure-orchard](https://codeberg.org/grammar-orchard/tree-sitter-clojure-orchard)
  Revision: `40dc14c61c46e48d39166b3cc60cdb16256e3384`
  License: MIT
  Notes: parser now comes from the Rust crate [tree-sitter-clojure-orchard](https://crates.io/crates/tree-sitter-clojure-orchard); the repository only keeps the bundled highlights query.

- `grammars/cmakecache/*`
  Source: repository-local
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/cmakecache/` directory now only keeps kat-side integration assets such as queries.

- `grammars/coffeescript/queries/highlights.scm`
- `grammars/coffeescript/queries/injections.scm`
  Source: [svkozak/tree-sitter-coffeescript](https://github.com/svkozak/tree-sitter-coffeescript)
  Revision: `3bb4dbd68ca926c76b3baadb529da4de3726ea37`
  License: MIT
  Notes: repository-local copies of the upstream CoffeeScript query assets. Parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers).

- `grammars/command_help/*`
  Source: repository-local
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/command_help/` directory now only keeps kat-side integration assets such as queries.

- `grammars/cpuinfo/*`
  Source: repository-local
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/cpuinfo/` directory now only keeps kat-side integration assets such as queries.

- `grammars/crontab/*`
  Source: [slqy123/tree-sitter-crontab](https://github.com/slqy123/tree-sitter-crontab)
  Revision: `70b5628278756c3dc429fac6545fe7b2e8c553a0`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/crontab/` directory now only keeps kat-side integration assets such as queries.

- `grammars/crystal/queries/highlights.scm`
- `grammars/crystal/queries/injections.scm`
  Source: [crystal-lang-tools/tree-sitter-crystal](https://github.com/crystal-lang-tools/tree-sitter-crystal)
  Revision: `50ca9e6fcfb16a2cbcad59203cfd8ad650e25c49`
  License: MIT
  Notes: repository-local copy of the upstream Crystal query assets, including the Neovim highlights query adapted for kat. Parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers).

- `grammars/d/queries/*`
  Source: [gdamore/tree-sitter-d](https://github.com/gdamore/tree-sitter-d)
  Revision: `fb028c8f14f4188286c2eef143f105def6fbf24f`
  License: MIT
  Notes: parser now comes from the Rust crate [tree-sitter-d](https://crates.io/crates/tree-sitter-d); the repository only keeps the upstream highlights and injections queries as integration assets.

- `grammars/debsources/*`
  Source: repository-local
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/debsources/` directory now only keeps kat-side integration assets such as queries.

- `grammars/elm/queries/*`
  Source: [elm-tooling/tree-sitter-elm](https://github.com/elm-tooling/tree-sitter-elm)
  Revision: `6d9511c28181db66daee4e883f811f6251220943`
  License: MIT
  Notes: parser now comes from the Rust crate [tree-sitter-elm](https://crates.io/crates/tree-sitter-elm); the repository only keeps the upstream highlights, injections and locals queries as integration assets.

- `grammars/email/*`
  Source: [stevenxxiu/tree-sitter-mail](https://github.com/stevenxxiu/tree-sitter-mail)
  Revision: `8d2905d06a15586652c3a73387b4170424201e1a`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/email/` directory now only keeps kat-side integration assets such as queries.

- `grammars/erlang/queries/highlights.scm`
  Source: [WhatsApp/tree-sitter-erlang](https://github.com/WhatsApp/tree-sitter-erlang)
  Revision: `1d78195c4fbb1fc027eb3e4220427f1eb8bfc89e`
  License: Apache-2.0
  Notes: parser now comes from the Rust crate [tree-sitter-erlang](https://crates.io/crates/tree-sitter-erlang); the repository only keeps the upstream highlights query as an integration asset.

- `grammars/fortran/queries/highlights.scm`
  Source: [stadelmanma/tree-sitter-fortran](https://github.com/stadelmanma/tree-sitter-fortran)
  Revision: `be30d90dc7dfa4080b9c4abed3f400c9163a88df`
  License: MIT
  Notes: parser now comes from the Rust crate [tree-sitter-fortran](https://crates.io/crates/tree-sitter-fortran); the repository only keeps the upstream highlights query as an integration asset.

- `grammars/fortran_namelist/*`
  Source: repository-local
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/fortran_namelist/` directory now only keeps kat-side integration assets such as queries.

- `grammars/fsharp/queries/*`
- `grammars/fsharp_signature/queries/*`
  Source: [ionide/tree-sitter-fsharp](https://github.com/ionide/tree-sitter-fsharp)
  Revision: `594c500ecace8618db32dd1144307897277db067`
  License: MIT
  Notes: parser now comes from the Rust crate [tree-sitter-fsharp](https://crates.io/crates/tree-sitter-fsharp); the repository only keeps the upstream highlights, injections and locals queries as integration assets for both source and signature runtimes.

- `grammars/fstab/*`
  Source: repository-local
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/fstab/` directory now only keeps kat-side integration assets such as queries.

- `grammars/scss/*`
  Source: [tree-sitter-grammars/tree-sitter-scss](https://github.com/tree-sitter-grammars/tree-sitter-scss)
  Revision: `2ef6d42e3ad7a8208900f9346f4529806ae0f9f9`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/scss/` directory now only keeps kat-side integration assets such as queries.

- `grammars/sass/*`
  Source: [bajrangCoder/tree-sitter-sass](https://github.com/bajrangCoder/tree-sitter-sass)
  Revision: `fb280c41b070657e4ff4d4e5e6eea6cb19efd9b8`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/sass/` directory now only keeps kat-side integration assets such as queries.

- `grammars/todotxt/*`
  Source: [arnarg/tree-sitter-todotxt](https://github.com/arnarg/tree-sitter-todotxt)
  Revision: `3937c5cd105ec4127448651a21aef45f52d19609`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/todotxt/` directory now only keeps kat-side integration assets such as queries.

- `grammars/vhdl/queries/highlights.scm`
  Source: [jpt13653903/tree-sitter-vhdl](https://github.com/jpt13653903/tree-sitter-vhdl)
  Revision: `9df4e7a9543699e10e08aa1232a9ec8076b948cf`
  License: MIT
  Notes: repository-local copy of the upstream Neovim-oriented VHDL highlights query. Parser code is linked from the Rust crate `tree-sitter-vhdl`.

- `grammars/vim/queries/highlights.scm`
- `grammars/vim/queries/injections.scm`
  Source: [tree-sitter-grammars/tree-sitter-vim](https://github.com/tree-sitter-grammars/tree-sitter-vim)
  Revision: crate release `0.4.0`
  License: MIT
  Notes: repository-local copies of the upstream Vim highlights and injections queries. Parser code is linked from the Rust crate `tree-sitter-vim`.

- `grammars/csharp/queries/highlights.scm`
  Source: [tree-sitter/tree-sitter-c-sharp](https://github.com/tree-sitter/tree-sitter-c-sharp)
  Revision: `88366631d598ce6595ec655ce1591b315cffb14c`
  License: MIT
  Notes: repository-local copy of the upstream C# highlights query. Parser code is linked from the Rust crate `tree-sitter-c-sharp`.

- `grammars/groovy/queries/highlights.scm`
  Source: [Decodetalkers/tree-sitter-groovy](https://github.com/Decodetalkers/tree-sitter-groovy)
  Revision: `70efb0b9b50f95bcbd89dcfd42b275e0304e10cf`
  License: MIT
  Notes: repository-local Groovy highlights query derived from the upstream grammar's available node set and adapted for kat's theme model. Parser code is linked from the Rust crate `tree-sitter-groovy`.

- `grammars/diff/queries/highlights.scm`
  Source: [the-mikedavis/tree-sitter-diff](https://github.com/the-mikedavis/tree-sitter-diff)
  Revision: `2520c3f934b3179bb540d23e0ef45f75304b5fed`
  License: MIT
  Notes: repository-local copy of the upstream diff highlights query. Parser code is linked from the Rust crate `tree-sitter-diff`.

- `grammars/properties/queries/highlights.scm`
  Source: [ObserverOfTime/tree-sitter-properties](https://github.com/ObserverOfTime/tree-sitter-properties)
  Revision: `6310671b24d4e04b803577b1c675d765cbd5773b`
  License: MIT
  Notes: repository-local copy of the upstream Java properties highlights query. Parser code is linked from the Rust crate `tree-sitter-properties`.

- `grammars/jq/queries/highlights.scm`
  Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/jq/` directory now only keeps kat-side integration assets such as queries.

- `grammars/less/*`
  Source: [amaanq/tree-sitter-less](https://github.com/amaanq/tree-sitter-less)
  Revision: `e5ae6245f841b5778c79ac93b28fa4f56b679c5d`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/less/` directory now only keeps kat-side integration assets such as queries.

- `grammars/dot/*`
  Source: [rydesun/tree-sitter-dot](https://github.com/rydesun/tree-sitter-dot)
  Revision: `80327abbba6f47530edeb0df9f11bd5d5c93c14d`
  License: MIT
  Notes: parser sources are no longer vendored in this repository; the runtime parser now comes from the git-backed Rust crate dependency on [tree-sitter-kat-parsers](https://github.com/happy-proto/kat-parsers). The local `grammars/dot/` directory now only keeps kat-side integration assets such as queries.

- `grammars/nginx/queries/highlights.scm`
- `grammars/nginx/queries/injections.scm`
  Source: [ngalaiko/tree-sitter-nginx](https://github.com/ngalaiko/tree-sitter-nginx)
  Revision: `47ade644d754cce57974aac44d2c9450e823d4f4`
  License: MIT
  Notes: repository-local copies of the upstream nginx highlights / injections queries. Parser code is linked from the Rust crate `tree-sitter-nginx`.

- `grammars/c/queries/highlights.scm`
  Source: [tree-sitter/tree-sitter-c](https://github.com/tree-sitter/tree-sitter-c)
  Revision: crate release `0.24.1`
  License: MIT
  Notes: repository-local copy of the upstream C highlights query. Parser code is linked from the Rust crate `tree-sitter-c`.

- `grammars/cpp/queries/highlights.scm`
- `grammars/cpp/queries/injections.scm`
  Source: [tree-sitter/tree-sitter-cpp](https://github.com/tree-sitter/tree-sitter-cpp)
  Revision: crate release `0.23.4`
  License: MIT
  Notes: repository-local copies of the upstream C++-specific highlights and raw-string injection queries. The `cpp` runtime combines the vendored C query with these C++ additions while linking parser code from the Rust crate `tree-sitter-cpp`.

- `grammars/java/queries/highlights.scm`
  Source: [tree-sitter/tree-sitter-java](https://github.com/tree-sitter/tree-sitter-java)
  Revision: crate release `0.23.5`
  License: MIT
  Notes: repository-local copy of the upstream Java highlights query. Parser code is linked from the Rust crate `tree-sitter-java`.

- `grammars/kotlin/queries/highlights.scm`
  Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: repository-local Kotlin highlights query written for kat because the upstream grammar crate currently does not ship query assets. Parser code is linked from the Rust crate `tree-sitter-kotlin-ng`.

- `grammars/ruby/queries/highlights.scm`
- `grammars/ruby/queries/locals.scm`
  Source: [tree-sitter/tree-sitter-ruby](https://github.com/tree-sitter/tree-sitter-ruby)
  Revision: crate release `0.23.1`
  License: MIT
  Notes: repository-local copies of the upstream Ruby highlights and locals queries. Parser code is linked from the Rust crate `tree-sitter-ruby`.

- `grammars/lua/queries/highlights.scm`
- `grammars/lua/queries/injections.scm`
- `grammars/lua/queries/locals.scm`
  Source: [tree-sitter-grammars/tree-sitter-lua](https://github.com/tree-sitter-grammars/tree-sitter-lua)
  Revision: crate release `0.5.0`
  License: MIT
  Notes: repository-local copies of the upstream Lua highlights, injections and locals queries. Parser code is linked from the Rust crate `tree-sitter-lua`.

- `grammars/nix/queries/highlights.scm`
- `grammars/nix/queries/injections.scm`
  Source: [nix-community/tree-sitter-nix](https://github.com/nix-community/tree-sitter-nix)
  Revision: crate release `0.3.0`
  License: MIT
  Notes: repository-local copies of the upstream Nix highlights and injections queries. Parser code is linked from the Rust crate `tree-sitter-nix`.

These files may be modified locally as needed for integration into `kat`.
