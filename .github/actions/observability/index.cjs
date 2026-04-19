const fs = require("node:fs/promises");
const path = require("node:path");

function getInput(name, fallback = "") {
  return process.env[`INPUT_${name.replace(/[\s-]+/g, "_").toUpperCase()}`] ?? fallback;
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

async function pathExists(targetPath) {
  try {
    await fs.access(targetPath);
    return true;
  } catch {
    return false;
  }
}

async function listCargoTimingsDirs(rootDir) {
  const dirs = [];

  async function visit(currentDir) {
    let entries;
    try {
      entries = await fs.readdir(currentDir, { withFileTypes: true });
    } catch (error) {
      if (error.code === "ENOENT") {
        return;
      }
      throw error;
    }

    for (const entry of entries) {
      if (!entry.isDirectory()) {
        continue;
      }

      const fullPath = path.join(currentDir, entry.name);
      if (entry.name === "cargo-timings") {
        dirs.push(fullPath);
        continue;
      }

      await visit(fullPath);
    }
  }

  await visit(rootDir);
  dirs.sort();
  return dirs;
}

async function copyIfExists(sourcePath, outputDir) {
  if (!sourcePath || !(await pathExists(sourcePath))) {
    return null;
  }

  const fileName = path.basename(sourcePath);
  const destinationPath = path.join(outputDir, fileName);
  await fs.cp(sourcePath, destinationPath, { force: true });
  return fileName;
}

async function copyCargoTimingsDirs(rootDir, outputDir) {
  const timingsDirs = await listCargoTimingsDirs(rootDir);

  for (const timingsDir of timingsDirs) {
    const relativePath = path.relative(rootDir, timingsDir);
    const destinationPath = path.join(outputDir, relativePath);
    await fs.mkdir(path.dirname(destinationPath), { recursive: true });
    await fs.cp(timingsDir, destinationPath, { recursive: true, force: true });
  }

  return timingsDirs.length;
}

async function countLines(filePath) {
  const content = await fs.readFile(filePath, "utf8");
  if (content.length === 0) {
    return 0;
  }
  return content.split(/\r?\n/).filter(line => line.length !== 0).length;
}

async function main() {
  const artifactName = getInput("artifact-name");
  const summaryTitle = getInput("summary-title", "Observability");
  const outputDir = getInput("output-dir");
  const cargoTimingsRoot = getInput("cargo-timings-root", "target");
  const linkLog = getInput("link-log");
  const writeSummary = getInput("write-summary", "true").toLowerCase() !== "false";

  if (!artifactName) {
    throw new Error("Missing required input: artifact-name");
  }
  if (!outputDir) {
    throw new Error("Missing required input: output-dir");
  }

  await fs.rm(outputDir, { recursive: true, force: true });
  await fs.mkdir(outputDir, { recursive: true });

  const linkLogFile = await copyIfExists(linkLog, outputDir);
  const cargoTimingsCount = await copyCargoTimingsDirs(cargoTimingsRoot, outputDir);
  const linkLogRecordCount =
    linkLogFile === null ? 0 : await countLines(path.join(outputDir, linkLogFile));

  await setOutput("cargo-timings-count", String(cargoTimingsCount));
  await setOutput("link-log-present", linkLogFile === null ? "false" : "true");
  await setOutput("link-log-file", linkLogFile ?? "");
  await setOutput("link-log-record-count", String(linkLogRecordCount));

  if (!writeSummary) {
    return;
  }

  const summaryLines = [
    `## ${summaryTitle}`,
    "",
    `- Artifact: \`${artifactName}\``,
  ];

  if (linkLogFile === null) {
    summaryLines.push("- Link timing records: not generated");
  } else {
    summaryLines.push(`- Link timing records: \`${linkLogRecordCount}\``);
  }

  if (cargoTimingsCount === 0) {
    summaryLines.push("- Cargo timings directories: not generated");
  } else {
    summaryLines.push(`- Cargo timings directories: \`${cargoTimingsCount}\``);
  }

  await appendFileFromEnv("GITHUB_STEP_SUMMARY", summaryLines.join("\n"));
}

main().catch(error => {
  process.stderr.write(`${error.stack || error.message}\n`);
  process.exitCode = 1;
});
