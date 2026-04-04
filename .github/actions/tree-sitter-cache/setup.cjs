const {
  buildManifest,
  getCacheKey,
  getCurrentManifestPath,
  getRestorePrefix,
  logError,
  logInfo,
  setOutput,
  writeJson,
} = require("./shared.cjs");

async function main() {
  const manifest = await buildManifest();
  const currentManifestPath = getCurrentManifestPath(
    manifest.cacheRoot,
    manifest.target,
    manifest.profile,
  );

  await writeJson(currentManifestPath, manifest);
  await setOutput("cache-key", getCacheKey(manifest));
  await setOutput("restore-prefix", getRestorePrefix());

  logInfo(
    `Prepared tree-sitter cache manifest ${manifest.digest} for ${manifest.grammars.length} grammars at ${currentManifestPath}`,
  );
}

main().catch(error => {
  logError(error.stack || error.message);
  process.exitCode = 1;
});
