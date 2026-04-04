const path = require("node:path");

const {
  getCachedManifestPath,
  getCurrentManifestPath,
  getInput,
  logError,
  logInfo,
  readJsonIfExists,
  removeDirIfExists,
  writeJson,
} = require("./shared.cjs");

async function main() {
  const cacheRoot = getInput("cache-root", ".build-cache/tree-sitter-cache");
  const target = getInput("target");
  const profile = getInput("profile", "release");

  if (!target) {
    throw new Error("Missing required input: target");
  }

  const currentManifestPath = getCurrentManifestPath(cacheRoot, target, profile);
  const cachedManifestPath = getCachedManifestPath(cacheRoot, target, profile);
  const currentManifest = await readJsonIfExists(currentManifestPath);

  if (!currentManifest) {
    throw new Error(`Missing current manifest at ${currentManifestPath}`);
  }

  await removeDirIfExists(path.join(cacheRoot, "v1", "staging", target, profile));
  await writeJson(cachedManifestPath, currentManifest);

  logInfo(`Finalized tree-sitter cache metadata for ${target}/${profile}`);
}

main().catch(error => {
  logError(error.stack || error.message);
  process.exitCode = 1;
});
