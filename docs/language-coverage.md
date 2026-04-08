# `zed` 参考基线下的 `kat` 语法支持现状

## 文档定位

本文档记录 `kat` 当前语言支持的真实成熟度、主要差距和特殊文件检测现状。

## 什么时候读

- 需要判断某门语言当前是不是样板语言时；
- 需要确认某类文件或嵌入场景是否已支持时；
- 需要评估新增工作更接近“补 query 细节”还是“补 runtime / detector 基础设施”时。

## 相关文档

- 项目说明见 [../README.md](../README.md)
- 文档总入口见 [README.md](README.md)
- 架构说明见 [architecture.md](architecture.md)
- 未完成事项见 [roadmap.md](roadmap.md)

本文档的目标，不是简单回答“某个语言有没有高亮”，而是记录：

1. `kat` 当前到底已经接入了什么；
2. 这些语言的 query / injection / 展示效果当前做到什么层级；
3. 以本地 [../zed](../zed) 仓库为参考时，我们和它在“语法高亮成熟度”上的差距主要在哪。

这里把 `zed` 当作语法支持成熟度的参考实现，而不是主题标准。
最终配色和语义解释仍然以 `kat` 自己的 Dracula 目标为准。

当前主题对齐原则补充：

- 语法覆盖成熟度可以参考 `zed`。
- 配色语义不以 `bat` 或 `zed` 的具体实现为准，而以 Dracula 官方 spec 为准。
- 如果 `bat` / `zed` 与 Dracula 官方语义冲突，`kat` 应优先遵循官方规范。

## 评估口径

为了避免把 IDE 能力和终端高亮混在一起，本文档把比较口径拆成两层：

- `语法层`：Tree-sitter grammar、highlights query、injections query、文件识别、嵌套解析。
- `编辑器层`：LSP、semantic tokens、toolchain、outline、textobjects、runnables、imports、brackets 等。

对 `kat` 来说，真正要对齐的是前一层。
后一层不是终端渲染器的直接目标，但可以作为 `zed` 语言成熟度的旁证。

## `kat` 层级定义

- `🟥 占位`：语言已经注册，能出基础颜色，但 capture 很粗，几乎还没做过针对性打磨。
- `🟨 基础`：常见 token 已可读，存在最小测试/样例，但 query 细节和嵌套场景还明显落后于成熟实现。
- `🟦 中等`：核心 token 和主要嵌套场景已经打通，日常阅读基本够用，但仍有一批语言特有细节未覆盖。
- `🟩 精细`：当前仓库里已经做过较系统的 query / injection 打磨，语法层效果可作为后续语言细化的参照。

### Emoji 速览

- `🟥`：刚接入，后续工作还很多。
- `🟨`：已经可用，但还有明显细化空间。
- `🟦`：结构基本到位，剩下主要是补细节。
- `🟩`：当前可作为样板语言，后续以维护和增量打磨为主。

## 当前结论

- `kat` 当前真正达到 `精细` 的，已经扩展为 `JSON`、`Bash`、`TOML`、`YAML`、`Git Config`、`Rust`、`Go`、`Markdown`、`Justfile`。
- 前五门语言已经不只是“亮起来”，而是开始按 Dracula 官方 spec 的语言语义落色：
  `JSON` / `TOML` / `YAML` 的 key 走配置语言 key 语义，`YAML alias` 单独走绿色斜体下划线，`Rust` 则细分到 trait、attribute、macro、function definition、local variable 等层级。
- `Python`、`HTML`、`CSS`、`JavaScript` 这一轮已经补齐了一批此前明显缺失的语义 capture，并新增 fixture / showcase / 专门测试锁住回归。
- `Go` 这轮也已接入为独立 runtime，并把 `zed` 的 Go highlights/injections 里对终端渲染最有价值的部分对齐进来。
- `go.mod`、`go.work`、`go.sum` 现在也已作为 Go 生态元数据文件接入独立 runtime，而不是混入 `.go` source runtime。
- `HCL` 现已作为独立配置语言 runtime 接入，覆盖 `.hcl` 与 `.nomad`；当前 query 已补齐注释、block/type、attribute key、function call、string/template、operator、布尔/数字/null。现有 fixture / showcase 以 Nomad 风格样例为主，但 runtime 定位仍保持通用 HCL。
- `SQL`、`Regex`、`GraphQL` 已经都从“明确待补的高收益嵌入语言”推进到“独立 runtime 已接入”；`SQL` 不仅支持顶层 `.sql` 文件，还新增了 `Postgres` / `MySQL` / `SQLite` 的方言分发层与无扩展名内容检测；`Regex` 已进一步演进成 `JavaScript` / `Python` / `Rust` / `Go` / `POSIX` 这些 host-aware runtime 族，并开始服务于 `JavaScript`、`Python`、`Rust`、`Go`、`Bash` / `Justfile` 等宿主；`GraphQL` 也已复用到 `JavaScript` tagged template / comment-hosted string、member-tagged template 与 Markdown fenced code，并支持无扩展名内容检测。
- 宿主字符串解码这层基础设施已经落地：`JavaScript` / `Python` / `Rust` / `Go` 的 raw / escaped string 不再只是按源码字面量截取，而会先经过统一 decode，再把高亮映射回原源码位置。
- `JSDoc` 也已经不再是纯占位，但受当前 upstream grammar 表达能力限制，参数名等细节仍没有达到和宿主语言同级的细粒度程度。
- `Justfile` 仍是当前一个明确强项，但参考基线应改为社区扩展 [`zed-just`](https://github.com/jackTabsCode/zed-just)，而不是再写成“`zed` 没有同级支持”。
- 对 `Justfile recipe`、Markdown fenced code、GitHub Actions `run` 这类块级 nested runtime，renderer 现在已经提供统一的 block contrast：会按共享缩进推导矩形区域并补齐较短行尾部背景；但 inline 注入和更复杂的非矩形区域还没有完全纳入同一视觉模型。

## 语言总览

| 语言 | `kat` 当前层级 | `kat` 当前事实 | `zed` 参考信号 | 现阶段判断 |
| --- | --- | --- | --- | --- |
| JSON | 🟩 精细 | 已支持 `.json` / `.jsonc`；fixture / showcase 已覆盖 rich 场景；当前 query 已细化到 comment、string、escape、number、boolean、null、delimiter、bracket、config-style object key。Dracula 语义上，JSON key 现按配置语言 key 使用青色，而不是退回普通 string。 | 本地 `zed` 有独立 `json` / `jsonc` grammar 包，附带 `outline`、`indents`、`textobjects`、`runnables` 等资产；其 `highlights.scm` 也把 key 单独提升为 `property.json_key`。 | 在终端高亮目标内，JSON 这一层已经基本没有明显缺口；后续再做更多工作也主要会落到编辑器层能力，而不是 query 本身。 |
| Git Config | 🟨 基础 | 已支持 `.gitconfig`、`gitconfig`、`.gitmodules`、`.git/config`、`*/git/config` 与 `config.worktree`；当前独立 `git_config` runtime 已覆盖 section / subsection、key、comment、boolean、integer、string、path-like string，以及 `include` / `includeIf` 这类 Git 特有 section 名。fixture 已覆盖顶层 config、submodule metadata 与路径感知 detector。 | 生态里通常会把它作为独立 `git_config` grammar，而不是退回通用 INI。 | 结构入口已经独立出来；下一步若继续做深，收益主要来自按 profile 做 Git schema-aware key/value 语义，而不是重新拆基础 parser。 |
| Ignore Files | 🟨 基础 | 已支持 `.gitignore`、`.dockerignore`、`*.dockerignore`、`.npmignore`、`.ignore`；当前独立 `ignore` runtime 已覆盖 comment、negation、directory separator、wildcard、bracket expr / char class 等高收益结构。fixture / showcase 已覆盖基础场景。 | 编辑器生态里通常会把这类文件作为专门文件类型处理，至少保证模式语法可读。 | 这一层已经脱离“纯文本”阶段，但目前仍是共享 ignore-pattern runtime；后续是否需要继续区分 Git / Docker 的语义差异，应以真实高收益差异为准。 |
| Dockerfile | 🟨 基础 | 已支持 `Dockerfile`、`Containerfile`、`Dockerfile.*`、`Containerfile.*` 以及 `.dockerfile` 扩展；当前独立 `dockerfile` runtime 已覆盖 instruction keyword、comment、image ref、param、string / escape、variable expansion 等基础结构，并把 `RUN`、shell-form `CMD` / `ENTRYPOINT`、`HEALTHCHECK CMD` 的 `shell_command` 注入 shell runtime；默认走 Bash，同时也已支持 `SHELL ["zsh", ...]`、`SHELL ["fish", ...]` 驱动后续 shell-form 指令切到对应 runtime。`RUN <<'EOF'` 这类 heredoc 行内容也已复用当前 shell runtime。宿主层现在还额外补了 `ARG` / `ENV` / `LABEL` key、`EXPOSE` port、`WORKDIR` / `COPY` path、常见 `--param=value` / `--mount=...` 的 name/value 语义、value 内变量展开、`--mount` 内部 key/value 与 enum / bool / number / path 这类常见 value 分类、按 key 区分的 mount family 语义，以及 JSON-form 命令数组首项、option argv、path-like executable、env-style argv 与 expansion argv 的宿主语义。fixture / showcase 已覆盖基础场景与 advanced heredoc/healthcheck/shell-dispatch/semantics/params/exec-form 场景。 | 本地 `zed` 文档也把 Dockerfile 视作独立语言，并依赖社区扩展与独立 Tree-sitter grammar。 | 这一层已经打通了“宿主 grammar + shell 注入”的关键架构；后续更值得继续抠的是更细的 param 子结构，以及 `SHELL [...]` 与 heredoc / healthcheck / JSON-form 边界的一致性细节。 |
| Bash | 🟩 精细 | 已支持扩展名、`.bashrc`、`.bash_profile`、`.bash_login`、`.profile`、`.bash_logout`、shebang；有 fixture / showcase；Justfile recipe 默认注入 Bash；解释器驱动的 heredoc 现可递归注入 `python` / `javascript` / `bash`，且 Justfile recipe 会自动复用。当前 query 已补齐 shebang directive、regex、special variable、parameter、ansi-c string、更多 punctuation/operator；同时 shell semantic layer 已开始接管一部分 query 很难长期表达的结构语义，例如 builtin family、declaration / unset 这类命令里的变量角色，以及 subscript bracket 这类组合结构。 | 本地 `zed` 有独立 Bash grammar 包，并补了 `textobjects`、`runnables`、`redactions` 等配套资产；其 `highlights.scm` 也是当前 `kat` Bash 细化的直接参考之一。 | 对终端高亮来说，Bash 仍是仓库内样板语言；下一步更值得继续做深的是沿这条 shell semantic layer 继续补 command family 和 expansion / subscript 细节，而不是把更多规则重新塞回 highlights query。 |
| TOML | 🟩 精细 | 已支持 `.toml`、`Cargo.toml`、`Cargo.lock` 和 `uv.lock`；Markdown `+++` frontmatter 已注入 TOML；fixture / showcase 已覆盖 quoted key、escape、datetime、inline table、array table，以及 lockfile 检测。Dracula 语义上，TOML key 现按配置语言 key 使用青色，datetime 走橙色值语义。 | 本地 `zed` 文档里有 TOML 语言支持，但当前代码树里没有像其它 grammar 那样直接暴露出可比对的 query 资产。 | 对 `kat` 而言，TOML 这层已经非常完整；现阶段很难再找到必须补的语法层缺口。 |
| YAML | 🟩 精细 | 已支持 `.yaml` / `.yml`；Markdown `---` frontmatter 已注入 YAML；fixture / showcase 已覆盖 anchor / alias / tag / merge key / block scalar / GitHub Actions workflow。当前除 `actions/github-script` 的 JavaScript 注入外，GitHub Actions workflow / `action.yml` 这类 profile 也已接入宿主级 resolver：`run` block 会按同级 `shell` 分发到 `bash` / `zsh` / `fish` / `python` 等已注册 runtime，`${{ ... }}` expression 与 `uses: owner/repo@ref` 也已有专门语义高亮。Dracula 语义上，YAML alias / anchor 单独走绿色斜体下划线，key 走配置语言 key 的青色。 | 本地 `zed` 有独立 YAML grammar 包，并额外做了 GitHub Actions `actions/github-script` -> JavaScript 注入；这部分已被 `kat` 对齐进统一 runtime。 | YAML 现在已经从“基础接入”升级到“场景化精细支持”；后续剩余工作更多是继续扩充被注入子语言，而不是 YAML 宿主层本身。 |
| Protocol Buffers (`.proto`) | 🟨 基础 | 已支持 `.proto` / `.protobuf` / `.protodevel`；当前独立 `proto` runtime 已覆盖 `syntax` / `package` / `import` / `message` / `service` / `rpc`、标量类型、message/service 标识符、字段编号和基础字符串字面量。fixture / showcase 已覆盖 schema 场景。 | 成熟编辑器生态通常会把 protobuf schema 作为独立 IDL 语言处理，而不是混入通用文本或数据格式 runtime。 | 这一层已经具备基础可读性；后续主要值得继续细化的是 enum / option / oneof / reserved / map 等 protobuf schema 专有结构。 |
| Protocol Buffers Text Format (`textproto`) | 🟨 基础 | 已支持 `.textproto` / `.textpb` / `.pbtxt` / `.prototxt`；当前独立 `textproto` runtime 已覆盖 field name、`:` delimiter、string / boolean / number 等基础值语义。fixture / showcase 已覆盖常见文本数据场景。 | 成熟编辑器生态通常会把 protobuf text format 视作独立于 `.proto` schema 的数据格式，而不是复用同一套 schema runtime。 | 这一层最重要的架构决策已经落定：schema 与文本数据格式拆成两个 runtime，比在同一 grammar 里混合特判更长期可维护。 |
| HCL | 🟨 基础 | 已支持 `.hcl` / `.nomad`；当前独立 `hcl` runtime 已覆盖注释、block/type、attribute key、function call、数字 / 布尔 / null、operator、template interpolation / directive、heredoc delimiter 与基本 string/token 结构。fixture / showcase 当前以 Nomad 风格 HCL 为主，但实现仍保持通用 HCL runtime。 | Helix 等成熟编辑器生态通常会把 HCL 作为独立配置语言处理，并在 Terraform / Nomad 等 DSL 之间复用同一语法基础。 | 这一层已经脱离“纯文本配置”阶段，足够支撑泛 HCL 文件的日常阅读；后续主要值得继续细化的是更多 HCL 专有 capture，以及是否要在未来为 Terraform / Nomad 这类生态增加 detector / overlay。 |
| Rust | 🟩 精细 | 已支持 `.rs`；宏 token-tree 注入、Rustdoc Markdown 注入、Rustdoc 内 fenced Rust / Python、普通 macro / attribute / trait / function definition / function method / variable / lifetime / local binding 都已细化；这一轮还补上了常见 SQL 宏字符串与 regex 宏 / `Regex(Builder)::new` 的注入。fixture / showcase 已覆盖 rich 结构、rustdoc nested case 与 SQL/Regex 场景。Dracula 语义上，attribute 走绿色斜体，trait/interface 走青色，local variable 保持前景色，不再被误染成 literal 色。 | 本地 `zed` 既有更细的 Rust highlights / injections，也有 `semantic_token_rules`、context provider、imports、runnables 等更成熟的语言层实现；当前 `kat` 已对齐它在 highlights/injections 里最关键的终端可见部分。 | 对终端语法高亮本身，Rust 已经达到当前仓库里最精细的一档；剩余明显差距已更多集中在 `rstml`、更细的 SQL 宏识别，以及编辑器层能力。 |
| Go | 🟩 精细 | 已支持 `.go`；当前 query 已补齐 package namespace、type / builtin type、generic type parameter、function definition、method definition / call、builtin function、directive comment、数值 / 字符串 / rune / escape；并且支持基于 comment hint 的 `json` / `yaml` / `html` / `javascript` / `css` / `bash` / `sql` / `regex` 注入调度。现在 `sql` 这条线还进一步支持了 `sql:postgres` / `sql:mysql` / `sql:sqlite` 方言 hint。fixture / showcase 已覆盖独立文件与 Markdown fenced `go` / `golang` alias；这些 `sql` / `regex` hint 已真正落到共享 runtime。 | 本地 `zed` 有独立 Go grammar 包，并维护了 `highlights.scm` 与面向 regex / sql / json / yaml / html / css / js / bash 等子语言的 `injections.scm`。 | 对 `kat` 来说，Go source runtime 已经达到样板语言级别；当前剩余收益更多来自继续补嵌入语言本体细节，而不是 Go 宿主层重写。 |
| Go Module Files (`go.mod` / `go.work` / `go.sum`) | 🟩 精细 | 现已作为 `gomod` / `gowork` / `gosum` 三个独立 runtime 接入，而不是混入 `.go` source runtime。`go.mod` 已细化 directive、module path、workspace/file path、replace operator、version、toolchain、retract range；`go.work` 已细化 go/use/replace 与 workspace path；`go.sum` 已细化 module path、module version、pseudo-version 数字段、`go.mod` suffix、`h1:` hash version 与 checksum value。fixture / showcase 已覆盖三类文件。 | `zed` 一类成熟语言系统通常会把这些 Go 生态特殊文件纳入独立语言或至少独立检测路径，而不是复用 Go source 语义。 | 这一层最重要的架构决策已经落定：保持 Go source 与 Go 生态元数据文件分离，是更简化也更长期可维护的设计。后续工作更多会是继续抠 query 细节，而不是重新争论是否应该并回 `.go` runtime。 |
| Python | 🟩 精细 | 已支持 `.py`；这一轮补齐了 decorator / decorator call、builtin call、builtin type、constructor method、annotation string、`self` / `cls`、`isinstance` / `issubclass`、更完整 docstring 识别；同时补上了 `# sql` 注释块、`.execute(...)` / `.executemany(...)` / `.executescript(...)` 调用点，以及 `re.*` / `regex.*` 调用点的 SQL/Regex 注入。raw string 与 backslash-escaped string 现在都会先 decode 再进入共享 runtime。fixture / showcase 已覆盖 advanced 场景与新增嵌入场景，并且所有嵌套 Python 场景都会直接复用同一 runtime。 | 本地 `zed` 的 Python grammar query 仍然还会继续覆盖更多注入场景，例如 SQL string injection；编辑器层还有 semantic tokens、toolchain、context provider。 | 在当前终端高亮目标下，Python 已经达到仓库内精细语言的标准；继续推进时，重点应更多落在嵌入语言边界规则，而不是再继续堆宿主语义 capture。 |
| HTML | 🟩 精细 | 已支持 `.html` / `.htm`；已注入 `<script>` -> JavaScript、`<style>` -> CSS，以及 `style=` -> CSS、`on*=` -> JavaScript；当前 query 还补上了 entity、custom element、attribute 与 nested-region showcase，fixture / showcase 已覆盖 rich 场景。 | 本地 `zed` 的 HTML 扩展除了 script/style，也对 `style=` 属性和 `on*=` 事件属性做了注入。 | 对终端阅读来说，HTML 这层剩余收益已经大多来自被注入子语言本体继续升级，而不是 HTML 宿主层本身继续加复杂规则。 |
| CSS | 🟩 精细 | 已支持 `.css`；这一轮补齐了 id/class/pseudo selector、namespace selector、custom property、at-rule、color value、unit、`!important`、keyframes 等细节，并新增 rich fixture / showcase 锁住回归。 | 本地 `zed` 有独立 CSS grammar 包，并配套 `outline`、`indents`、`textobjects`、`overrides`。 | 对 `kat` 而言，CSS 已经从“基础可读”升级到“宿主和嵌套场景都可直接复用的精细 runtime”；后续主要是继续让更多宿主语言注入到它，而不是再重做 CSS 本体。 |
| JavaScript | 🟩 精细 | 已支持 `.js` / `.mjs` / `.cjs` / `.jsx` 与 shebang；已注入 JSDoc；tagged template 现已细化到 `css/json/html/sql/yaml/graphql` 调度；regex pattern 也开始走 host-aware `regex_javascript` runtime。当前不仅支持 regex literal，还支持 `RegExp("...")`、`RegExp(\`...\`)`、`RegExp(String.raw\`...\`)`、`/* sql */` / `/* sql:postgres|mysql|sqlite */`、`/* graphql */` 注释宿主普通字符串 / template string、`.query(...)` / `.execute(...)` / `.prepare(...)` 这类 SQL 调用点、sqlite 风格的 `.get(...)` / `.all(...)` / `.run(...)` 调用点，以及 `client.gql\`...\`` 这类 member-tagged GraphQL，并会先按宿主字面量规则 decode 再进入子语言 runtime。query 补齐了 class / heritage / constructor / private property / function definition / regex body+flags / decorator / JSX tag+attribute 等细节；userscript metadata 也已通过独立 mini-runtime 融入统一注入路径；fixture / showcase 已覆盖 rich / JSX / userscript / SQL / GraphQL injection 场景。 | 本地 `zed` 的 JavaScript injections 仍更丰富，特别是 `sql`、`regex`、`graphql` 等独立嵌入 runtime 已更成熟。 | `kat` 的 JavaScript 本体已经可以归入精细档。剩余差距主要收敛到更复杂的 template 边界语义，以及后续继续加深被注入子语言本体，而不是“GraphQL 还没接进来”这类基础缺口。 |
| GraphQL | 🟦 中等 | 已支持顶层 `.graphql` / `.gql` / `.graphqls` 文件；当前 query 已覆盖 operation / schema / type-system keyword、directive、variable、fragment definition / spread、field / argument / object field、builtin operation type，以及基础 string / number / boolean / null。`JavaScript` 的 `gql` / `graphql` tagged template、member-tagged template 与 `/* graphql */` comment-hosted string，Markdown fenced `graphql` / `gql`，以及无扩展名内容检测，都会复用同一 runtime。fixture / showcase 已覆盖顶层文件、JavaScript 宿主与 Markdown fenced alias。 | `zed` 一类成熟实现通常也会把 GraphQL 作为高收益注入目标，尤其是 JavaScript tagged template。 | 这一层已经从“预留入口”推进到“真正接通 runtime”。接下来若还要继续抠，重点应是继续细化 query，而不是重新搭接宿主分发。 |
| JSDoc | 🟦 中等 | 已作为 JavaScript comment injection 目标接入；当前 tags、types、optional 参数括号、默认值 `=`、member/path delimiter、inline tag 与 code fence marker / language 都已有独立 query，能在 JavaScript / HTML 宿主场景里稳定复用。这一轮又补上了 semantic overlay：在 grammar 只给出 `inline_tag -> description` 这类粗节点时，仍能把 `ThemePreview#render`、`module:theme/preview` 这类 inline reference target 拆成 identifier/path 片段和 delimiter。 | 本地 `zed` 也有独立 `jsdoc` grammar 资产。 | 这门语言已经不再只是“tag 能亮”；当前主要剩余边界来自 upstream grammar 对描述区节点表达仍然偏粗，而不是 runtime 分发路径本身。 |
| Userscript Metadata | 🟩 精细 | 已作为 JavaScript comment injection 目标接入，使用独立 mini-runtime 解析 `==UserScript==` block；当前已细化 marker、metadata key、localized key suffix、URL、match pattern、grant API、special enum value、resource alias 等语义；Markdown fenced JavaScript 也会自动复用。 | 未在当前本地 `zed` 树里看到现成同类 mini-language 支持。 | 这是一个很适合 `kat` 当前注入架构的 DSL：语义稳定、宿主单一、收益高，而且不会要求基础设施为了这个 case 再额外特判。 |
| Markdown | 🟩 精细 | 已支持 block + inline；已支持 fenced code、HTML block、inline、YAML/TOML frontmatter；当前这些场景已经统一走递归 injection runtime，Rustdoc 多段 Markdown 也能复用同一条路径。Markdown 主体已补齐 heading、setext heading、ordered/unordered/task list、blockquote、table、reference-style link、image、autolink、inline HTML、plain fenced code 等展示与回归样例；配色语义严格以 Dracula 官方 Markup spec 为准。 | 本地 `zed` 也有 block / inline grammar，并支持 fenced code、HTML、frontmatter 等基础 injection。 | `kat` 的 Markdown 现在已经不只是“架构通了”，而是连 Dracula 的 Markup 语义也开始明确落地，可以作为后续处理其它 markup-like 场景的样板。剩余短板主要已收敛到被注入子语言本体的 query 深度，而不是 Markdown 主体本身。 |
| Justfile / Just | 🟩 精细 | 已支持 `justfile`、`Justfile`、`JUSTFILE`、`.just`；默认 recipe 注入 Bash；shebang recipe 与 `set shell := [...]` 现已可复用共享 runtime 别名归一化，覆盖 `python3`/`uv -> python`、`node`/`nodejs`/`bun -> javascript`、`sh`/`zsh`/`fish -> bash`，以及 `pwsh` / `powershell -> powershell`、`cmd.exe` / `cmd` / `batch -> batch` 这类 Windows shell 场景；fixture / showcase 已覆盖。 | `zed` 官方语言文档收录了社区扩展 [`zed-just`](https://github.com/jackTabsCode/zed-just)。该扩展包含独立的 `highlights.scm` / `injections.scm` / `outline.scm` / `runnables.scm` / `tasks.json`，并集成 `just-lsp`。它的 injections 还覆盖默认 Bash、`set shell := [...]`、以及 `python3`/`uv`/`node`/`bun`/`ts`/`tsx`/`deno`/`sh`/`zsh`/`fish`/`pwsh` 等 shebang/shell 映射。 | `kat` 现在已经具备与 `zed-just` 同类的专门语言支持基础，并且把一部分 shebang / shell 别名收敛到了共享 runtime 归一化层，因此不仅 Justfile 受益，其他嵌套场景未来也可直接复用。当前剩余差距主要是 `typescript` 和 tasks/runnables/LSP 这类编辑器层能力，而不再是 Windows shell runtime 缺口。 |
| PowerShell | 🟨 基础 | 已接入独立 `powershell` runtime，支持 `.ps1` / `.psm1` / `.psd1` 与 `pwsh` / `powershell` shebang；GitHub Actions `shell: pwsh`、`powershell {0}` 模板与 Justfile 的 pwsh shebang recipe 现在都会真正进入 PowerShell runtime。当前 query 已覆盖 keyword、operator、command、function、type、member/property、variable、string、number 与 comment；这一轮还补上了轻量 semantic overlay，用于把常见 cmdlet（如 `Write-Host` / `Get-Item`）提升成 builtin 语义，并把 `$env:...` 这类特殊变量提升成 special variable。fixture 已覆盖顶层文件和 GitHub Actions / Justfile 嵌套场景。 | 成熟实现通常会把 PowerShell 当成独立 shell 语言，而不是简单退回 POSIX shell 语义。 | 这一层已经从“没有 runtime”推进到“独立语言可用”。后续收益主要来自 query 细化和更完整的 shell-specific semantic overlay，而不是接线问题本身。 |
| Batch / CMD | 🟨 基础 | 已接入独立 `batch` runtime，支持 `.bat` / `.cmd`；GitHub Actions `shell: cmd` 与 Justfile 的 `set shell := [\"cmd.exe\", ...]` 现在都会进入 Batch runtime。当前 query 已覆盖 `@echo off`、`set`、`if/for/goto/call`、comparison/redirect operator、command name、variable reference、label、number、string 与 redirect target；这一轮又补上了轻量 semantic overlay，把常见 builtin command（如 `echo`）提升成 builtin 语义，并把 label 定义与 `goto :eof` 这种高收益目标做了专门落色。fixture 已覆盖顶层文件和 GitHub Actions / Justfile 嵌套场景。 | 成熟实现通常会把 Windows Batch / CMD 视作独立脚本语言，而不是交给 Bash 类 runtime。 | 这一层的基础 runtime 已经落地；后续值得继续做的是变量展开、delayed expansion、更多 command family 与 block 结构细节。 |
| SQL | 🟦 中等 | 已支持顶层 `.sql`、`.psql`、`.pgsql`、`.postgresql`、`.mysql`、`.mariadb`、`.sqlite`、`.sqlite3` 文件；当前 query 已覆盖 comment、keyword、operator、builtin type、parameter、field、alias、dollar-quoted string，以及一批更完整的 DDL / DML / 约束关键字；底层 parser 现改为使用 `tree-sitter-sequel` crate 提供的预生成 SQL grammar，并在共享 SQL runtime 之上继续维护 `sql_postgres` / `sql_mysql` / `sql_sqlite` runtime。顶层文件、无扩展名内容检测、`JavaScript` / `Python` / `Go` / `Rust` 的 SQL callsite、`Bash` / `Justfile` heredoc，以及 Markdown fenced `postgres` / `mysql` / `sqlite` alias 现在都会进入统一 SQL 方言分发。宿主普通字符串与 raw string 也会先 decode 再进入 SQL runtime，因此 `JavaScript` comment-hosted plain string、`Python` triple-quoted string、`Go` interpreted string、`Rust` 普通字符串等场景都已真正复用同一条链路。detector 也已经从纯正分模型升级成“正分 + 负分”，并继续补上了 `UNLOGGED` / `GENERATED ALWAYS AS IDENTITY` / `CREATE EXTENSION`、`ZEROFILL` / `INSERT IGNORE` / `SHOW CREATE TABLE`、`BEGIN IMMEDIATE` / `ATTACH DATABASE` / `REINDEX` 等更强的方言信号。现在还额外有 AST 驱动的 semantic overlay，专门补 query 很难稳定表达的方言结构，例如 Postgres 的 `LANGUAGE plpgsql` / index `opclass`，以及 MySQL 的 `ENGINE=InnoDB` / `CHARSET=utf8mb4` 这类 option value。fixture / showcase 已覆盖 direct file、多宿主注入和三种方言样例。 | 本地 `zed` 有 SQL 语言文档，并在 `Python` / `Rust` / `JavaScript` 等宿主 query 里把 SQL 作为重要注入目标。 | 这一层已经从“明确待补”推进到“高收益基础设施已落地”。当前不再缺“有没有方言感知”，而是共享 SQL parser 对方言细节的上限还没到独立 grammar 那一档。接下来真正的分水岭会是：继续加深 runtime 覆层，还是转向 per-dialect grammar。 |
| Regex | 🟦 中等 | 已接入独立 runtime 族；当前 query 已覆盖 group、escape、assertion、quantifier、class、inline flag、unicode property、named backreference、POSIX character class name 等高收益结构；并已演进成 `regex_javascript`、`regex_python`、`regex_rust`、`regex_go`、`regex_posix` 这些 host-aware runtime。宿主层现在不仅支持 raw string，也支持 backslash-escaped string decode，因此 `RegExp("...")`、`RegExp(\`...\`)`、`RegExp(String.raw\`...\`)`、`re.compile("...")`、`Regex::new("...")`、`regexp.MustCompile("...")`、`regexp.MustCompilePOSIX("...")` 都会落到合适的 regex runtime。宿主不支持的结构会通过 `invalid.illegal.regex` 显式标红下划线；这一轮还把 quantifier、inline flag group、unicode property、character class range 这类结构同时沉到 semantic overlay，避免未来继续把 bracket/operator 细节硬塞进 query。fixture / showcase 已覆盖多宿主复用。 | `zed` 在 Rust / JavaScript 等语言里把它当成嵌入目标，并对 bracket/operator/escape/quantifier 等结构做专门 capture。 | 对终端观感来说，这一层已经不是占位，而是能明显改善复杂模式阅读的共享 runtime。当前剩余边界主要不在宿主 decode，而在 upstream regex grammar 对某些分隔符 token 的节点表达不够细；这里应优先尊重 AST 实际能力，而不是为了少数 token 往 renderer 里塞脆弱 hack。 |
| Fish | 🟦 中等 | 已接入独立 `fish` runtime，支持 `.fish`、`config.fish`、`fishfile` 与 fish shebang；当前 query 已覆盖 comment、shebang、string、escape、number、operator、function definition、command/builtin、flag/option、variable expansion、fish special variable、`for` loop 变量、command substitution、glob，以及 `case` pattern 的 fish-specific 语义；这一轮除了继续扩 builtin 覆盖面，也开始由共享 shell semantic layer 处理 `status` / `string` 这类 builtin family 的 subcommand、`function --argument-names` / `--on-variable` 这类函数元数据参数，以及 list access 的结构化语义。fixture / showcase 已覆盖独立文件场景；Justfile 的 fish shebang recipe 也已真正落到 fish runtime，而不再退回 bash。 | 本地生态里已有独立 grammar。 | 这一层已经不只是“能识别成 fish”，而是开始有 shell-specific 结构语义；与 Bash 样板语言相比，剩余空间主要收敛到更细的 expansion / list access / builtin-family 细节，而不是 runtime 是否独立。 |
| Zsh | 🟦 中等 | 已接入独立 `zsh` runtime，支持 `.zsh`、`.zsh-theme`、`.zshrc`、`.zprofile`、`.zlogin`、`.zlogout`、`.zshenv`、`.zsh_aliases`、`.zsh_functions` 及若干 `.zsh*.local` 文件与 zsh shebang；当前 query 已覆盖 comment、shebang、string、ansi-c string、expansion、regex、test operator、glob qualifier、declaration command、command name、常见 control keyword，并继续补上了 `setopt` / `unsetopt` 选项名、`autoload` / `source` 等高收益结构；这一轮还通过共享 shell semantic layer 补上了 builtin family、declaration / read 里的变量角色，以及 subscript bracket 这类组合结构语义。fixture / showcase 已覆盖独立文件场景；Justfile 的 `set shell := [\"zsh\", ...]` 也已真正进入 zsh runtime。 | `zed` 文档层有 shell 相关支持，生态上也通常作为常见 shell 处理。 | 这一层已经从“shell alias 退回 bash”推进到真正的独立语言支持，并且不再只靠 query 硬堆细节；后续主要工作会继续落在 shell semantic layer 与 Zsh expansion AST 的配合上。 |

## 按“离 `zed` 还差什么”分类

### 🟩 第一档：已经能长期当样板继续打磨

- `JSON`
- `Bash`
- `TOML`
- `YAML`
- `Rust`
- `Go`
- `Python`
- `HTML`
- `CSS`
- `JavaScript`
- `Markdown`
- `Justfile`

这些语言已经不只是“亮起来了”，而是有比较稳定的 query / injection / showcase 基础，适合当后续主题与基础设施调整时的回归样板。

### 🟨 / 🟥 第二档：已经接入，但还不能误写成“样板语言”

- `GraphQL`
- `JSDoc`
- `SQL`
- `Regex`
- `HCL`
- `Proto`
- `Textproto`

这类语言的问题不再是“完全没接入”，而是 grammar / query 表达能力本身还有限。
其中 `JSDoc` 当前最值得继续观察的是：是否要升级 grammar revision、补独立 injections，或者接受它维持在“tags / types 已经够用”的层级。`GraphQL` 则已经接通 runtime，但 query 还明显浅于样板语言；`SQL` / `Regex` 的剩余工作更多集中在共享 grammar 的表达上限。

## 特殊文件与检测场景

| 场景 | `kat` 当前状态 | `zed` 参考情况 | 现阶段判断 |
| --- | --- | --- | --- |
| `Cargo.toml` | 🟩 已支持 | `zed` 语言系统里也把 manifest 作为 Rust / TOML 生态的一部分使用。 | 已达标。 |
| `Cargo.lock` / `uv.lock` | 🟩 已支持 | `zed` 设置系统可按语言自定义文件名映射。 | TOML 侧这类高频 lockfile 的检测缺口已经补上。 |
| `go.mod` | 🟩 已支持 | 现已按文件名直接进入独立 `gomod` runtime，而不是复用 Go source runtime；`module` / `require` / `replace` / `exclude` / `retract` / `toolchain` 及 module path / version / local path 都有专门高亮。 | `zed` 一类成熟语言系统通常会把生态内关键文件名纳入识别范围。 | 这类 manifest 文件现在已经有了明确而稳定的专用处理路径。 |
| `go.work` | 🟩 已支持 | 现已按文件名直接进入独立 `gowork` runtime；workspace path、`use` / `replace` directive 与版本字段都按 workspace 语义高亮。 | 参考成熟语言系统时，这类 workspace 文件通常也会与语言生态一并处理。 | 这类文件不再需要在 Go source 与 manifest 风格之间妥协。 |
| `go.sum` | 🟩 已支持 | 现已按文件名直接进入独立 `gosum` runtime；module path、module version、pseudo-version、`go.mod` suffix、`h1:` 与 checksum value 都有专门 capture。 | `zed` 侧即便未必单独强调，也可视作 Go 生态常见文件名识别的一部分。 | `go.sum` 现在也不再只是“能认出来”，而是有自己合适的高亮语义。 |
| Bash shebang | 🟩 已支持 | `zed` 主要依赖语言 matcher / 编辑器打开语义。 | 对终端工具来说已经足够关键。 |
| Markdown fenced code | 🟩 已支持 | `zed` 也支持。 | 现在已经走统一的 fenced-language dispatch；后续收益应主要来自子语言本身继续细化。 |
| Markdown YAML/TOML frontmatter | 🟩 已支持 | `zed` 也支持。 | 这部分已经对齐到合理基线。 |
| HTML 内嵌 `<script>` / `<style>` / `style=` / `on*=` | 🟩 已支持 | `zed` 也支持。 | 这一层结构性注入已经补齐，后续主要继续细化 CSS / JS 本体 query。 |
| Justfile recipe 嵌套语言 | 🟩 已支持 | `zed-just` 也支持默认 Bash、全局 shell 与多种 shebang 映射。 | `kat` 这一层已经具备对照 `zed-just` 的基础，并且由于 Bash heredoc 注入已下沉到共享 runtime，Justfile 中的 `python/node/bash <<'EOF'` 也会自动受益；后续重点应转向补 runtime 缺口与语言本体 query。 |
| Markdown fenced `go` / `golang` | 🟩 已支持 | `zed` 也支持按 fenced language dispatch 到 Go runtime。 | `kat` 现在已把 `go` runtime 接入统一 fenced-language 分发，并补了 `golang -> go` 的 alias 归一化，因此顶层 `.go` 文件和 Markdown/Rustdoc 等嵌套 Go 场景天然复用同一套 query。 |
| `Dockerfile` / `.dockerignore` | 🟨 已支持 | `Dockerfile` 现已作为独立 runtime 接入，并支持 `RUN`、shell-form `CMD` / `ENTRYPOINT`、`HEALTHCHECK CMD` 和 `RUN` heredoc 复用当前 shell runtime；默认走 Bash，也已支持由 `SHELL [...]` 驱动切到 `zsh` / `fish` 等已注册 shell runtime。宿主层 query 也已补到常见 key / port / path、`--mount` key/value 与常见 value 分类、按 key 区分的 mount family、value 内变量展开，以及 JSON-form 首项命令、option、path-like executable、env-style argv 和 expansion argv 语义。`.dockerignore` 则继续复用共享 `ignore` runtime。 | 成熟实现通常会把 `Dockerfile` 当成独立宿主语言，并进一步处理其中的 shell 片段。 | 结构性缺口已经补上；下一阶段更值得继续做深的是 param 子结构与边界一致性，而不是重新讨论 runtime 归属。 |
| GitHub Actions workflow / `action.yml` | 🟦 中等 | 现已通过 YAML profile 检测支持 `.github/workflows/*.yml` / `*.yaml` 与 `action.yml` / `action.yaml`。`run` block 会按 step 级 `shell`、`defaults.run.shell` 以及默认 Bash 分发到现有 shell / Python / PowerShell / Batch runtime；`python {0}`、`bash -euo pipefail {0}` 这类 shell template 现也会先归一化到真实 runtime。`${{ ... }}` expression 已有独立语义补层，且 workflow 里裸写的 `if:` expression 与 `run` block 内嵌的 `${{ ... }}` 现在也会继续保留 GitHub Actions 语义高亮；`uses:` 现已覆盖 `owner/repo@ref`、`owner/repo/path@ref`、`docker://image` 与本地 `./path` 这些常见 action ref 形态。除此之外，`permissions` value、`runs.using`、`shell` value、`with.cache`、`with.if-no-files-found`、静态 `runs-on` label，以及 `matrix.include[*].runner` 这类常见 runner label 也开始走 profile-aware 的 schema 语义落色。当前这层仍建立在 YAML 宿主 profile + host resolver + semantic overlay 上，而不是独立 grammar。 | 成熟实现通常会把 workflow YAML 视作 YAML 宿主上的场景化 profile，并对 expression、`uses`、`run` 做额外高亮或 schema 感知。 | 这一层的关键架构已经落定：不复制第二套 YAML grammar，而是沿 document profile / host resolver / overlay 继续做深。后续收益主要来自 expression 子语言进一步细化，以及更多 schema-aware key/value 语义的一致性补齐。 |
| Vue 单文件组件 | 🟥 待定 | 先记录为明确想支持的一类前端文件，但当前还没有确定文件后缀、运行时归属与宿主/嵌入语言拆分策略。 | `zed` 这类成熟实现通常会把模板、脚本、样式分层处理。 | 现阶段先保留需求记录，后续再决定是否以 `.vue` 作为主入口，以及内部如何分发到 HTML / JavaScript / CSS runtime。 |
| React 组件文件 | 🟥 待定 | 先记录为明确想支持的一类前端文件；当前仓库已经支持 `.jsx`，但 React 相关文件命名与是否纳入 `.tsx` 等后缀策略，这里先不下结论。 | 成熟实现通常会把 JSX / TSX 作为 React 生态的主要文件形态。 | 现阶段先记录方向，后续再统一决定 React 文件范围、后缀集合与 detector 策略。 |
| Userscript metadata block | 🟩 已支持 | 暂未见本地 `zed` 中有同级内建支持信号。 | 当前通过 JavaScript comment injection + 独立 mini-runtime 支持，因此独立 `.js` 文件和所有嵌套 JavaScript 场景都能原生复用。 |
| 无扩展名配置文件 / stdin | 🟨 仍较弱 | 现已对明显像 `SQL` / `GraphQL` 的无扩展名内容补上启发式识别，但其它配置类文件仍较弱。 | `zed` 可以通过语言映射设置扩展。 | 这条线已经从“完全缺失”推进到“开始有内容感知”，但离系统化 detector 还差不少。 |

## 相比 `bat` 的文件类型缺口

下面这节只记录“文件类型识别 / runtime 入口”层面的差距，不评价 `bat` 与 `kat` 在 query 质量、嵌入语言或终端主题语义上的优劣。对照基线为本机 `bat --list-languages` 当前输出。

- `kat` 当前 registry 已注册 `31` 组 runtime。
- 相比 `bat`，当前还有 `156` 组文件类型完全没有对应 runtime / detector 入口。
- 另外还有 `15` 组文件类型已部分覆盖，但文件名、扩展名或特殊检测范围仍明显窄于 `bat`。

### 完全缺失：`kat` 还没有对应 runtime / detector 入口

- `ActionScript`：`as`
- `Ada`：`adb`、`ads`、`gpr`
- `Apache Conf`：`envvars`、`htaccess`、`HTACCESS`、`htgroups`、`HTGROUPS`、`htpasswd`、`HTPASSWD`、`.htaccess`、`.HTACCESS`、`.htgroups`、`.HTGROUPS`、`.htpasswd`、`.HTPASSWD`、`httpd.conf`、`/etc/apache2/**/*.conf`、`/etc/apache2/sites-*/**/*`、`/etc/httpd/conf/**/*.conf`
- `AppleScript`：`applescript`、`script editor`
- `ARM Assembly`：`s`、`S`
- `AsciiDoc (Asciidoctor)`：`adoc`、`ad`、`asciidoc`
- `ASP`：`asa`
- `Authorized Keys`：`authorized_keys`、`pub`、`authorized_keys2`
- `AWK`：`awk`
- `BibTeX`：`bib`
- `C`：`c`
- `C#`：`cs`、`csx`
- `C++`：`cpp`、`cc`、`cp`、`cxx`、`c++`、`h`、`hh`、`hpp`、`hxx`、`h++`、`inl`、`ipp`、`*.h`
- `Cabal`：`cabal`
- `CFML`：`cfml`、`cfm`、`cfc`
- `Clojure`：`clj`、`cljc`、`cljs`、`edn`
- `CMake`：`CMakeLists.txt`、`cmake`
- `CMake C Header`：`h.in`
- `CMake C++ Header`：`hh.in`、`hpp.in`、`hxx.in`、`h++.in`
- `CMakeCache`：`CMakeCache.txt`
- `CoffeeScript`：`coffee`、`Cakefile`、`coffee.erb`、`cson`
- `Command Help`：`cmd-help`、`help`
- `CpuInfo`：`cpuinfo`
- `Crontab`：`tab`、`crontab`、`cron.d`
- `Crystal`：`cr`
- `D`：`d`、`di`
- `Dart`：`dart`
- `debsources`：`sources.list`
- `Diff`：`diff`、`patch`、`*.debdiff`
- `Elixir`：`ex`、`exs`
- `Elm`：`elm`
- `Email`：`eml`、`msg`、`mbx`、`mboxz`、`/var/spool/mail/*`、`/var/mail/*`
- `Erlang`：`erl`、`hrl`、`Emakefile`、`emakefile`、`escript`
- `F#`：`fs`、`fsi`、`fsx`、`*.fs`
- `Fortran (Fixed Form)`：`f`、`F`、`f77`、`F77`、`for`、`FOR`、`fpp`、`FPP`
- `Fortran (Modern)`：`f90`、`F90`、`f95`、`F95`、`f03`、`F03`、`f08`、`F08`
- `Fortran Namelist`：`namelist`
- `fstab`：`fstab`、`crypttab`、`mtab`
- `GDScript (Godot Engine)`：`gd`
- `Git Attributes`：`attributes`、`gitattributes`、`.gitattributes`、`/home/example/.config/git/attributes`
- `Git Commit`：`COMMIT_EDITMSG`、`MERGE_MSG`、`TAG_EDITMSG`
- `Git Link`：`.git`
- `Git Log`：`gitlog`
- `Git Mailmap`：`.mailmap`、`mailmap`
- `Git Rebase Todo`：`git-rebase-todo`
- `GLSL`：`vs`、`gs`、`vsh`、`fsh`、`gsh`、`vshader`、`fshader`、`gshader`、`vert`、`frag`、`geom`、`tesc`、`tese`、`comp`、`glsl`、`mesh`、`task`、`rgen`、`rint`、`rahit`、`rchit`、`rmiss`、`rcall`
- `gnuplot`：`gp`、`gpl`、`gnuplot`、`gnu`、`plot`、`plt`
- `Graphviz (DOT)`：`dot`、`DOT`、`gv`
- `Groff/troff`：`groff`、`troff`、`1`、`2`、`3`、`4`、`5`、`6`、`7`、`8`、`9`
- `Groovy`：`groovy`、`gvy`、`gradle`、`Jenkinsfile`
- `group`：`group`
- `Haskell`：`hs`
- `Highlight non-printables`：`show-nonprintable`
- `Hosts File`：`hosts`
- `HTML (ASP)`：`asp`
- `HTML (EEx)`：`html.eex`、`html.leex`
- `HTML (Erlang)`：`yaws`
- `HTML (Jinja2)`：`htm.j2`、`html.j2`、`xhtml.j2`、`xml.j2`
- `HTML (Rails)`：`rails`、`rhtml`、`erb`、`html.erb`
- `HTML (Tcl)`：`adp`
- `HTML (Twig)`：`twig`、`html.twig`
- `HTTP Request and Response`：`http`
- `Idris`：`idr`
- `INI`：`ini`、`INI`、`inf`、`INF`、`reg`、`REG`、`lng`、`cfg`、`CFG`、`desktop`、`url`、`URL`、`.editorconfig`、`.coveragerc`、`.pylintrc`、`.gitlint`、`.hgrc`、`hgrc`、`**/.aws/credentials`、`**/.aws/config`、`/etc/letsencrypt/renewal/*.conf`、`/etc/wireguard/*.conf`
- `Java`：`java`、`bsh`
- `Java Properties`：`properties`
- `Java Server Page (JSP)`：`jsp`
- `JavaScript`：`htc`
- `JavaScript (Rails)`：`js.erb`
- `Jinja2`：`j2`、`jinja2`、`jinja`
- `JQ`：`jq`
- `jsonnet`：`jsonnet`、`libsonnet`、`libjsonnet`
- `Julia`：`jl`
- `Known Hosts`：`known_hosts`、`known_hosts.old`
- `Kotlin`：`kt`、`kts`
- `LaTeX`：`tex`、`ltx`
- `Lean 4`：`lean`
- `Less`：`less`、`css.less`
- `Lisp`：`lisp`、`cl`、`clisp`、`l`、`mud`、`el`、`scm`、`ss`、`lsp`、`fasl`、`sld`
- `Literate Haskell`：`lhs`
- `LiveScript`：`ls`、`Slakefile`、`ls.erb`
- `LLVM`：`ll`
- `log`：`log`
- `Lua`：`lua`、`*.nse`
- `Makefile`：`make`、`GNUmakefile`、`makefile`、`Makefile`、`makefile.am`、`Makefile.am`、`makefile.in`、`Makefile.in`、`OCamlMakefile`、`mak`、`mk`
- `Manpage`：`man`
- `MATLAB`：`matlab`
- `MediaWiki`：`mediawiki`、`wikipedia`、`wiki`
- `MemInfo`：`meminfo`
- `NAnt Build File`：`build`
- `nginx`：`conf.erb`、`nginx.conf`、`mime.types`、`fastcgi_params`、`scgi_params`、`uwsgi_params`、`/etc/nginx/**/*.conf`、`/etc/nginx/sites-*/**/*`
- `Nim`：`nim`、`nims`、`nimble`
- `Ninja`：`ninja`
- `Nix`：`nix`
- `NSIS`：`nsi`、`nsh`、`bnsi`、`bnsh`、`nsdinc`
- `Objective-C`：`m`
- `Objective-C++`：`mm`
- `OCaml`：`ml`、`mli`
- `OCamllex`：`mll`
- `OCamlyacc`：`mly`
- `Odin`：`odin`
- `orgmode`：`org`
- `Pascal`：`pas`、`p`、`dpr`
- `passwd`：`passwd`
- `Perl`：`pl`、`pc`、`pm`、`pmc`、`pod`、`t`
- `PHP`：`php`、`php3`、`php4`、`php5`、`php7`、`phps`、`phpt`、`phtml`
- `Plain Text`：`txt`
- `Puppet`：`pp`、`epp`
- `PureScript`：`purs`
- `QML`：`qml`、`qmlproject`
- `R`：`R`、`r`、`Rprofile`
- `Racket`：`rkt`
- `Rd (R Documentation)`：`rd`
- `Rego`：`rego`
- `Regular Expression`：`re`
- `Requirements.txt`：`requirements.txt`、`requirements.in`、`pip`
- `resolv`：`resolv.conf`
- `reStructuredText`：`rst`、`rest`
- `Robot Framework`：`robot`、`resource`
- `Ruby`：`rb`、`Appfile`、`Appraisals`、`Berksfile`、`Brewfile`、`capfile`、`cgi`、`Cheffile`、`config.ru`、`Deliverfile`、`Fastfile`、`fcgi`、`Gemfile`、`gemspec`、`Guardfile`、`irbrc`、`jbuilder`、`Podfile`、`podspec`、`prawn`、`rabl`、`rake`、`Rakefile`、`Rantfile`、`rbx`、`rjs`、`ruby.rail`、`Scanfile`、`simplecov`、`Snapfile`、`thor`、`Thorfile`、`Vagrantfile`
- `Ruby Haml`：`haml`
- `Ruby on Rails`：`rxml`、`builder`
- `Ruby Slim`：`slim`、`skim`
- `Salt State (SLS)`：`sls`
- `Sass`：`sass`
- `Scala`：`scala`、`sbt`、`sc`、`*.mill`
- `SCSS`：`scss`
- `Separated Values`：`csv`
- `SML`：`sml`、`cm`、`sig`
- `Solidity`：`sol`
- `SQL (Rails)`：`erbsql`、`sql.erb`
- `SSH Config`：`ssh_config`、`**/.ssh/config`
- `SSHD Config`：`sshd_config`
- `Strace`：`strace`
- `Stylus`：`styl`、`stylus`
- `Svelte`：`svlt`、`svelte`
- `Swift`：`swift`
- `syslog`：`syslog`
- `SystemVerilog`：`sv`、`svh`、`vh`
- `Tab Separated Values`：`tsv`
- `Tcl`：`tcl`
- `TeX`：`sty`、`cls`
- `Textile`：`textile`
- `Todo.txt`：`todo.txt`、`done.txt`
- `TypeScript`：`ts`、`mts`、`cts`
- `TypeScriptReact`：`tsx`
- `Typst`：`typ`
- `varlink`：`varlink`
- `Verilog`：`v`、`V`
- `VHDL`：`vhd`、`vhdl`、`vho`、`vht`
- `VimHelp`：`vimhelp`
- `VimL`：`vim`、`vimrc`、`gvimrc`、`.vimrc`、`.gvimrc`、`.exrc`、`.nvimrc`、`_vimrc`、`_gvimrc`、`_exrc`
- `Vue Component`：`vue`
- `Vyper`：`vy`
- `WGSL`：`wgsl`
- `x86_64 Assembly`：`yasm`、`nasm`、`asm`、`inc`、`mac`
- `XML`：`xml`、`xsd`、`xslt`、`tld`、`dtml`、`rng`、`rss`、`opml`、`svg`、`xaml`、`*.csproj`、`*.vbproj`、`*.props`、`*.targets`
- `Zig`：`zig`、`zon`

### 部分覆盖：已有相关 runtime，但文件识别范围仍窄于 `bat`

- `Bourne Again Shell (bash)`：`kat` 已覆盖 `sh`、`bash`、`zsh`、`.bash_aliases`、`.bash_functions`、`.bash_login`、`.bash_logout`、`.bash_profile`、`.bashrc`、`.profile`、`.zlogin`、`.zlogout`、`.zprofile`、`.zshenv`、`.zshrc`；仍缺 `ash`、`.bash_completions`、`.bash_variables`、`.textmate_init`、`PKGBUILD`、`ebuild`、`eclass`、`**/bat/config`、`*.ksh`、`*.kshrc`、`/etc/os-release`、`/var/run/os-release`、`/etc/profile`、`bashrc`、`*.bashrc`、`bash_profile`、`*.bash_profile`、`bash_login`、`*.bash_login`、`bash_logout`、`*.bash_logout`、`zshrc`、`*.zshrc`、`zprofile`、`*.zprofile`、`zlogin`、`*.zlogin`、`zlogout`、`*.zlogout`、`zshenv`、`*.zshenv`
- `CSS`：`kat` 已覆盖 `css`；仍缺 `css.erb`、`css.liquid`
- `Dockerfile`：`kat` 已覆盖 `Dockerfile`、`dockerfile`、`Containerfile`；仍缺 `.Dockerfile`
- `DotENV`：`kat` 已覆盖 `.envrc`；仍缺 `.env`、`.env.dist`、`.env.local`、`.env.sample`、`.env.example`、`.env.template`、`.env.test`、`.env.test.local`、`.env.testing`、`.env.dev`、`.env.development`、`.env.development.local`、`.env.prod`、`.env.production`、`.env.production.local`、`.env.dusk.local`、`.env.staging`、`.env.default`、`.env.defaults`、`.flaskenv`、`env`、`env.example`、`env.sample`、`env.template`
- `Git Ignore`：`kat` 已覆盖 `gitignore`、`.gitignore`；仍缺 `exclude`、`/home/example/.config/git/ignore`
- `HTML`：`kat` 已覆盖 `html`、`htm`；仍缺 `shtml`、`xhtml`
- `JavaScript (Babel)`：`kat` 已覆盖 `js`、`mjs`、`jsx`、`cjs`；仍缺 `babel`、`es6`、`*.pac`
- `JSON`：`kat` 已覆盖 `json`；仍缺 `sublime-settings`、`sublime-menu`、`sublime-keymap`、`sublime-mousemap`、`sublime-theme`、`sublime-build`、`sublime-project`、`sublime-completions`、`sublime-commands`、`sublime-macro`、`sublime-color-scheme`、`ipynb`、`Pipfile.lock`、`*.jsonl`、`*.jsonc`、`*.jsonld`、`*.geojson`、`*.ndjson`、`flake.lock`、`*.sarif`
- `Markdown`：`kat` 已覆盖 `md`、`markdown`；仍缺 `mdown`、`markdn`、`*.mkd`
- `Python`：`kat` 已覆盖 `py`；仍缺 `py3`、`pyw`、`pyi`、`pyx`、`pyx.in`、`pxd`、`pxd.in`、`pxi`、`pxi.in`、`rpy`、`cpy`、`SConstruct`、`Sconstruct`、`sconstruct`、`SConscript`、`gyp`、`gypi`、`Snakefile`、`vpy`、`wscript`、`bazel`、`bzl`、`*.xsh`、`*.xonshrc`
- `Rust`：`kat` 已覆盖 `rs`；仍缺 `*.ron`
- `SQL`：`kat` 已覆盖 `sql`；仍缺 `ddl`、`dml`
- `Terraform`：`kat` 已覆盖 `hcl`；仍缺 `tf`、`tfvars`
- `TOML`：`kat` 已覆盖 `toml`、`Cargo.lock`、`uv.lock`；仍缺 `tml`、`Gopkg.lock`、`Pipfile`、`pdm.lock`、`poetry.lock`
- `YAML`：`kat` 已覆盖 `yaml`、`yml`；仍缺 `sublime-syntax`、`CITATION.cff`、`.clang-format`、`fish_history`

## 后续细化优先级

如果目标是“先把语法支持水平对齐到 `zed` 的参考线，再继续扩语言”，当前建议顺序是：

1. `SQL`
   原因：方言分发已经落地，下一步的高收益问题变成继续做深 runtime 覆层，还是在共享 grammar 上限到来后切到 per-dialect grammar。
2. `Regex`
   原因：runtime 已接入，但不同宿主目前仍主要覆盖高收益边界；继续细化会直接提升 JS / Rust / Python / Go 等复杂模式可读性。
3. `GraphQL`
   原因：runtime 与宿主分发已经接通，后续每一轮 query 细化都会同时反馈到顶层文件、JavaScript template string 与 Markdown fenced code。
4. `JSDoc`
   原因：这是当前仍明显落后于样板语言的一门已接入 grammar；如果要继续追精细度，应该优先判断是 query 还能补，还是 upstream grammar 本身已成为瓶颈。

## 备注

- 本文档关注的是当前仓库真实状态，不应为了“看起来支持很多语言”而把 `基础` 写成 `精细`。
- 如果后续某门语言做了实质性 query / injection 细化，应同步更新这里的层级和差距描述，而不是只改 README。
