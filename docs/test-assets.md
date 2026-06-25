# 测试资产

本文档只记录仓库内测试样例和展示样例的组织约定。

## 目录结构

- 统一样例目录在 [testdata](../testdata/README.md)。
- `fixtures/` 放自动化测试使用的小而稳定的输入。
- `perf/` 放仓库内性能基线清单与相关输入组织约定。
- `showcase/` 放便于人工查看渲染效果的较完整样例。

## 维护约定

- 每个新的语言支持、检测规则或嵌套高亮场景，都应先补最小 `fixture`。
- 默认优先用端到端测试锁住用户可见的 CLI / 终端行为；只有行为难以通过真实终端稳定观察，或目标本身就是 `analysis` / `visual` / `layout` / `render_ops` 这类 debug contract 时，才直接断言对应稳定 IR。
- 当测试目标是“是否复用了正确的 nested runtime / profile”，优先在 analysis snapshot 上断言递归子区域的 `document kind`，不要再通过颜色片段间接猜测。
- 当问题只在某个渲染层稳定复现时，优先补能锁定该层语义的最小 `fixture`，再决定是否需要额外的 ANSI / PTY 回放样例。
- 涉及 block 对齐、右侧补齐、显示列宽或 ANSI 剥离的回归时，优先补带宽字符的最小 `fixture`（至少覆盖 CJK，必要时再补 emoji / tab），并让断言复用共享的 `display_geometry` 语义以及 `ByteOffset` / `DisplayColumn` 约定，而不是在测试里重新手写一套宽度规则。
- 需要长期观察性能回归时，把基线输入纳入 `testdata/perf/` 管理，并通过 `just perf` 复用同一套入口。
- 当某个嵌套场景已经进入效果打磨阶段时，再补至少一个可人工检查的 `showcase` 文件。
- 优先使用真实代码风格的样例，而不是只堆 token。
- 当一个语言、主题规则或 injection 场景难以通过微型样例判断效果时，再补更完整的 `showcase`。
- 对终端兼容性、wrapped layout、内建 viewer 或 ANSI 编码问题，优先先固定最小 `fixture`，再通过 Ghostty E2E 锁住用户可见行为；只有目标是中间层 contract 时才退回 `--debug-layout` / `--debug-render-ops` / `--debug-terminal`。
- 需要验证真实终端边界时，优先使用 `ghostty-e2e` feature 下的 PTY + `libghostty-vt` 测试：测试先在固定尺寸 PTY 里运行真实 `kat` 二进制，再把输出字节交给 Ghostty 的无头终端模型断言 screen/cell 结果；交互式场景可以在进程退出前发送输入或 resize PTY。该测试依赖 `libghostty-vt-sys` 的 Ghostty native build，需要匹配 Ghostty 要求的 Zig 工具链；默认 `just test` 不启用它，CI 通过独立 `ghostty-e2e` job 显式运行。
- 手工运行 Ghostty 端到端测试使用 `just ghostty-e2e`。
- 一旦仓库存在 `justfile`，就使用 `just test` 作为标准测试入口；需要手工查看效果时，使用 `just showcase`。
