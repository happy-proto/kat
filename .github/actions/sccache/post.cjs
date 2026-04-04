const fs = require("node:fs/promises");

const {
  appendFileFromEnv,
  escapeCommandValue,
  getSccacheStats,
  logError,
  sumCounter,
} = require("./shared.cjs");

async function main() {
  const disableAnnotations = (process.env.STATE_disable_annotations || "false").toLowerCase() === "true";
  const sccachePath = process.env.STATE_sccache_path || process.env.SCCACHE_PATH;
  const cleanupRoot = process.env.STATE_cleanup_root;

  try {
    if (!disableAnnotations && sccachePath) {
      const stats = await getSccacheStats(sccachePath);
      const hitCount = sumCounter(stats.json.stats.cache_hits);
      const missCount = sumCounter(stats.json.stats.cache_misses);
      const errorCount = sumCounter(stats.json.stats.cache_errors);
      const total = hitCount + missCount + errorCount;
      const ratio = total === 0 ? 0 : Math.round((hitCount / total) * 100);
      const notice = `${ratio}% - ${hitCount} hits, ${missCount} misses, ${errorCount} errors`;

      process.stdout.write(`::notice title=sccache stats::${escapeCommandValue(notice)}\n`);

      const summary = [
        "## sccache stats",
        "",
        `- Cache hit %: \`${ratio}%\``,
        `- Cache hits: \`${hitCount}\``,
        `- Cache misses: \`${missCount}\``,
        `- Cache errors: \`${errorCount}\``,
        `- Compile requests: \`${stats.json.stats.compile_requests}\``,
        `- Requests executed: \`${stats.json.stats.requests_executed}\``,
        "",
        "<details><summary>Full human-readable stats</summary>",
        "",
        "```text",
        stats.human.trimEnd(),
        "```",
        "",
        "</details>",
        "",
      ].join("\n");

      await appendFileFromEnv("GITHUB_STEP_SUMMARY", summary);
    }
  } finally {
    if (cleanupRoot) {
      await fs.rm(cleanupRoot, { recursive: true, force: true });
    }
  }
}

main().catch(error => {
  logError(error.stack || error.message);
  process.exitCode = 1;
});
