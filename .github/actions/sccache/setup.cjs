const {
  addPath,
  exportVariable,
  getInput,
  installSccache,
  logError,
  logInfo,
  resolveVersion,
  saveState,
} = require("./shared.cjs");

async function main() {
  const version = await resolveVersion();
  const { installRoot, binaryPath, cleanupRoot } = await installSccache(version);

  await addPath(installRoot);
  await exportVariable("SCCACHE_PATH", binaryPath);
  await exportVariable("ACTIONS_CACHE_SERVICE_V2", "on");
  await exportVariable("ACTIONS_CACHE_URL", process.env.ACTIONS_CACHE_URL || "");
  await exportVariable("ACTIONS_RESULTS_URL", process.env.ACTIONS_RESULTS_URL || "");
  await exportVariable("ACTIONS_RUNTIME_TOKEN", process.env.ACTIONS_RUNTIME_TOKEN || "");

  await saveState("sccache_path", binaryPath);
  await saveState("disable_annotations", getInput("disable_annotations", "false"));
  await saveState("cleanup_root", cleanupRoot);

  logInfo(`Configured sccache ${version} at ${binaryPath}`);
}

main().catch(error => {
  logError(error.stack || error.message);
  process.exitCode = 1;
});
