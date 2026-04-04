const crypto = require("node:crypto");
const fs = require("node:fs/promises");
const path = require("node:path");

function getInput(name, fallback = "") {
  return process.env[`INPUT_${name.replace(/[\s-]+/g, "_").toUpperCase()}`] ?? fallback;
}

function logInfo(message) {
  process.stdout.write(`${message}\n`);
}

function logError(message) {
  process.stderr.write(`${message}\n`);
}

async function appendFileFromEnv(envName, line) {
  const target = process.env[envName];
  if (!target) {
    throw new Error(`Missing required GitHub Actions file command path: ${envName}`);
  }
  await fs.appendFile(target, `${line}\n`, "utf8");
}

async function setOutput(name, value) {
  await appendFileFromEnv("GITHUB_OUTPUT", `${name}=${value}`);
}

async function mkdirp(dirPath) {
  await fs.mkdir(dirPath, { recursive: true });
}

async function pathExists(targetPath) {
  try {
    await fs.access(targetPath);
    return true;
  } catch {
    return false;
  }
}

async function removeDirIfExists(targetPath) {
  await fs.rm(targetPath, { recursive: true, force: true });
}

async function readRegistryGrammarNames(registryPath) {
  const content = await fs.readFile(registryPath, "utf8");
  const grammarNames = [];
  const blockPattern = /^\s*\[\[grammar\]\]\s*$/gm;
  const positions = [...content.matchAll(blockPattern)].map(match => match.index ?? 0);

  for (let index = 0; index < positions.length; index += 1) {
    const start = positions[index];
    const end = index + 1 < positions.length ? positions[index + 1] : content.length;
    const block = content.slice(start, end);
    const nameMatch = block.match(/^\s*name\s*=\s*"([^"]+)"\s*$/m);
    if (!nameMatch) {
      throw new Error(`Failed to parse grammar name from block:\n${block}`);
    }
    grammarNames.push(nameMatch[1]);
  }

  if (grammarNames.length === 0) {
    throw new Error(`No [[grammar]] entries found in ${registryPath}`);
  }

  return grammarNames;
}

async function listFilesRecursively(rootDir) {
  const entries = await fs.readdir(rootDir, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const fullPath = path.join(rootDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await listFilesRecursively(fullPath)));
    } else if (entry.isFile()) {
      files.push(fullPath);
    }
  }

  files.sort();
  return files;
}

async function computeDirectoryHash(dirPath) {
  const hash = crypto.createHash("sha256");
  const files = await listFilesRecursively(dirPath);

  for (const filePath of files) {
    const relativePath = path.relative(dirPath, filePath).split(path.sep).join("/");
    hash.update(`path:${relativePath}\n`);
    hash.update(await fs.readFile(filePath));
    hash.update("\n");
  }

  return hash.digest("hex");
}

function computeManifestDigest(manifest) {
  const hash = crypto.createHash("sha256");
  hash.update(
    JSON.stringify(
      manifest.grammars.map(grammar => ({
        name: grammar.name,
        hash: grammar.hash,
      })),
    ),
  );
  return hash.digest("hex");
}

function getRestorePrefix() {
  const keyPrefix = getInput("key-prefix");
  const profile = getInput("profile", "release");

  if (!keyPrefix) {
    throw new Error("Missing required input: key-prefix");
  }

  return `${keyPrefix}-${profile}-`;
}

function getCacheKey(manifest) {
  return `${getRestorePrefix()}${manifest.digest}`;
}

function getMetadataDir(cacheRoot, target, profile) {
  return path.join(cacheRoot, ".action-metadata", target, profile);
}

function getCurrentManifestPath(cacheRoot, target, profile) {
  return path.join(getMetadataDir(cacheRoot, target, profile), "current-manifest.json");
}

function getCachedManifestPath(cacheRoot, target, profile) {
  return path.join(getMetadataDir(cacheRoot, target, profile), "cached-manifest.json");
}

async function buildManifest() {
  const cacheRoot = getInput("cache-root", ".build-cache/tree-sitter-cache");
  const registryPath = getInput("registry-path", "grammars/registry.toml");
  const target = getInput("target");
  const profile = getInput("profile", "release");

  if (!target) {
    throw new Error("Missing required input: target");
  }

  const grammarNames = await readRegistryGrammarNames(registryPath);
  const grammars = [];

  for (const name of grammarNames) {
    const grammarDir = path.join(path.dirname(registryPath), name);
    grammars.push({
      name,
      hash: await computeDirectoryHash(grammarDir),
    });
  }

  grammars.sort((left, right) => left.name.localeCompare(right.name));

  const digest = computeManifestDigest({ grammars });

  return {
    version: 1,
    target,
    profile,
    digest,
    generatedAt: new Date().toISOString(),
    grammars,
    cacheRoot,
  };
}

async function writeJson(filePath, value) {
  await mkdirp(path.dirname(filePath));
  await fs.writeFile(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

async function readJsonIfExists(filePath) {
  if (!(await pathExists(filePath))) {
    return null;
  }
  return JSON.parse(await fs.readFile(filePath, "utf8"));
}

module.exports = {
  buildManifest,
  getCacheKey,
  getCachedManifestPath,
  getCurrentManifestPath,
  getInput,
  getRestorePrefix,
  logError,
  logInfo,
  pathExists,
  readJsonIfExists,
  removeDirIfExists,
  setOutput,
  writeJson,
};
