const crypto = require("node:crypto");
const fs = require("node:fs/promises");
const os = require("node:os");
const path = require("node:path");
const { spawn } = require("node:child_process");

function getInput(name, fallback = "") {
  return process.env[`INPUT_${name.replace(/ /g, "_").toUpperCase()}`] ?? fallback;
}

async function appendFileFromEnv(envName, line) {
  const target = process.env[envName];
  if (!target) {
    throw new Error(`Missing required GitHub Actions file command path: ${envName}`);
  }
  await fs.appendFile(target, `${line}\n`, "utf8");
}

async function exportVariable(name, value) {
  await appendFileFromEnv("GITHUB_ENV", `${name}=${value}`);
}

async function addPath(dir) {
  await appendFileFromEnv("GITHUB_PATH", dir);
}

async function saveState(name, value) {
  await appendFileFromEnv("GITHUB_STATE", `${name}=${value}`);
}

function logInfo(message) {
  process.stdout.write(`${message}\n`);
}

function logError(message) {
  process.stderr.write(`${message}\n`);
}

async function execFile(cmd, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, {
      stdio: ["ignore", "pipe", "pipe"],
      ...options,
    });
    let stdout = "";
    let stderr = "";

    child.stdout.on("data", chunk => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", chunk => {
      stderr += chunk.toString();
    });
    child.on("error", reject);
    child.on("close", code => {
      if (code === 0) {
        resolve({ stdout, stderr });
      } else {
        reject(new Error(`${cmd} ${args.join(" ")} failed with exit code ${code}\n${stderr}`));
      }
    });
  });
}

function getArchiveComponents(version) {
  return {
    filename: `sccache-${version}-${getArch()}-${getPlatform()}.tar.gz`,
    dirname: `sccache-${version}-${getArch()}-${getPlatform()}`,
  };
}

function getArch() {
  switch (process.arch) {
    case "x64":
      return "x86_64";
    case "arm64":
      return "aarch64";
    case "arm":
      return "armv7";
    default:
      throw new Error(`Unsupported arch "${process.arch}"`);
  }
}

function getPlatform() {
  switch (process.platform) {
    case "darwin":
      return "apple-darwin";
    case "win32":
      return "pc-windows-msvc";
    case "linux":
      return process.arch === "arm" ? "unknown-linux-musleabi" : "unknown-linux-musl";
    default:
      throw new Error(`Unsupported platform "${process.platform}"`);
  }
}

async function computeSha256(filePath) {
  const hash = crypto.createHash("sha256");
  hash.update(await fs.readFile(filePath));
  return hash.digest("hex");
}

async function downloadToFile(url, outPath, headers = {}) {
  const response = await fetch(url, { headers });
  if (!response.ok) {
    throw new Error(`Failed to download ${url}: ${response.status} ${response.statusText}`);
  }
  const bytes = Buffer.from(await response.arrayBuffer());
  await fs.writeFile(outPath, bytes);
}

async function resolveVersion() {
  const requested = getInput("version");
  if (requested) {
    return requested;
  }

  const token = getInput("token");
  const headers = {
    "user-agent": "happy-proto-kat-sccache-action",
    accept: "application/vnd.github+json",
  };
  if (token) {
    headers.authorization = `Bearer ${token}`;
  }

  const response = await fetch("https://api.github.com/repos/mozilla/sccache/releases/latest", {
    headers,
  });
  if (!response.ok) {
    throw new Error(`Failed to resolve latest sccache release: ${response.status} ${response.statusText}`);
  }
  const release = await response.json();
  if (!release.tag_name) {
    throw new Error("Latest sccache release did not include tag_name");
  }
  return release.tag_name;
}

async function installSccache(version) {
  const { filename, dirname } = getArchiveComponents(version);
  const tmpRoot = await fs.mkdtemp(path.join(os.tmpdir(), "kat-sccache-"));
  const archivePath = path.join(tmpRoot, filename);
  const extractRoot = path.join(tmpRoot, "extract");
  const installRoot = path.join(tmpRoot, "tool-cache");
  const downloadUrl = `https://github.com/mozilla/sccache/releases/download/${version}/${filename}`;
  const shaUrl = `${downloadUrl}.sha256`;

  logInfo(`Downloading sccache ${version} from ${downloadUrl}`);
  await downloadToFile(downloadUrl, archivePath);
  const checksumResponse = await fetch(shaUrl, {
    headers: { "user-agent": "happy-proto-kat-sccache-action" },
  });
  if (!checksumResponse.ok) {
    throw new Error(`Failed to download ${shaUrl}: ${checksumResponse.status} ${checksumResponse.statusText}`);
  }
  const expectedChecksum = (await checksumResponse.text()).trim();
  const actualChecksum = await computeSha256(archivePath);
  if (actualChecksum !== expectedChecksum) {
    throw new Error(`Checksum verification failed for ${filename}`);
  }
  logInfo(`Verified sccache archive checksum: ${actualChecksum}`);

  await fs.mkdir(extractRoot, { recursive: true });
  await execFile("tar", ["-xzf", archivePath, "-C", extractRoot]);
  const sourceDir = path.join(extractRoot, dirname);
  const sourceBinary = path.join(sourceDir, "sccache");
  await fs.mkdir(installRoot, { recursive: true });
  const targetBinary = path.join(installRoot, "sccache");
  await fs.copyFile(sourceBinary, targetBinary);
  await fs.chmod(targetBinary, 0o755);
  return { installRoot, binaryPath: targetBinary, cleanupRoot: tmpRoot };
}

function sumCounter(counter) {
  if (!counter || !counter.counts) {
    return 0;
  }
  return Object.values(counter.counts).reduce((acc, value) => acc + value, 0);
}

function escapeCommandValue(value) {
  return String(value)
    .replace(/%/g, "%25")
    .replace(/\r/g, "%0D")
    .replace(/\n/g, "%0A");
}

async function getSccacheStats(binaryPath) {
  const human = await execFile(binaryPath, ["--show-stats"]);
  const json = await execFile(binaryPath, ["--show-stats", "--stats-format=json"]);
  return {
    human: human.stdout,
    json: JSON.parse(json.stdout),
  };
}

module.exports = {
  addPath,
  appendFileFromEnv,
  escapeCommandValue,
  execFile,
  exportVariable,
  getInput,
  getSccacheStats,
  installSccache,
  logError,
  logInfo,
  resolveVersion,
  saveState,
  sumCounter,
};
