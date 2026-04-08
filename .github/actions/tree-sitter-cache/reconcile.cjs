const path = require("node:path");

const {
  getCachedManifestPath,
  getCurrentManifestPath,
  getInput,
  logError,
  logInfo,
  pathExists,
  readJsonIfExists,
  removeDirIfExists,
  writeJson,
} = require("./shared.cjs");

function grammarMap(manifest) {
  return new Map((manifest?.grammars || []).map(entry => [entry.name, entry.hash]));
}

async function pruneStaleGrammarCaches(cacheRoot, target, profile, staleGrammarNames) {
  const generatedRoot = path.join(cacheRoot, "v1", "generated");
  const nativeRoot = path.join(cacheRoot, "v1", "native", target, profile);

  for (const grammarName of staleGrammarNames) {
    await removeDirIfExists(path.join(generatedRoot, grammarName));
    await removeDirIfExists(path.join(nativeRoot, grammarName));
  }
}

async function removeStagingCache(cacheRoot, target, profile) {
  await removeDirIfExists(path.join(cacheRoot, "v1", "staging", target, profile));
}

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

  const cachedManifest = await readJsonIfExists(cachedManifestPath);
  const current = grammarMap(currentManifest);
  const cached = grammarMap(cachedManifest);
  const staleGrammarNames = [];

  for (const [grammarName, cachedHash] of cached.entries()) {
    if (!current.has(grammarName) || current.get(grammarName) !== cachedHash) {
      staleGrammarNames.push(grammarName);
    }
  }

  await pruneStaleGrammarCaches(cacheRoot, target, profile, staleGrammarNames);
  await removeStagingCache(cacheRoot, target, profile);
  await writeJson(cachedManifestPath, currentManifest);

  const cacheState =
    (await pathExists(cachedManifestPath)) && cachedManifest ? "restored" : "cold";
  logInfo(
    `Reconciled tree-sitter cache (${cacheState}) for ${target}/${profile}: pruned ${staleGrammarNames.length} grammar directories`,
  );
}

main().catch(error => {
  logError(error.stack || error.message);
  process.exitCode = 1;
});
