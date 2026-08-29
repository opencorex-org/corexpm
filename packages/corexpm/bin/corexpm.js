#!/usr/bin/env node
"use strict";

const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");
const os = require("os");

function getBinaryName() {
  return os.platform() === "win32" ? "corexpm.exe" : "corexpm";
}

function resolveBinaryPath() {
  if (process.env.COREX_BINARY_PATH && fs.existsSync(process.env.COREX_BINARY_PATH)) {
    return process.env.COREX_BINARY_PATH;
  }

  const binaryName = getBinaryName();
  const pkgRoot = path.join(__dirname, "..");
  const workspaceRoot = path.join(pkgRoot, "..", "..");

  const releaseBin = path.join(workspaceRoot, "target", "release", binaryName);
  if (fs.existsSync(releaseBin)) {
    return releaseBin;
  }

  const debugBin = path.join(workspaceRoot, "target", "debug", binaryName);
  if (fs.existsSync(debugBin)) {
    return debugBin;
  }

  const platform = os.platform();
  const arch = os.arch();
  const vendorBin = path.join(pkgRoot, "vendor", `${platform}-${arch}`, binaryName);
  if (fs.existsSync(vendorBin)) {
    return vendorBin;
  }

  return binaryName;
}

function main() {
  const binaryPath = resolveBinaryPath();
  const args = process.argv.slice(2);

  const result = spawnSync(binaryPath, args, {
    stdio: "inherit",
    env: process.env,
    windowsHide: true,
  });

  if (result.error) {
    if (result.error.code === "ENOENT") {
      console.error(`[Error] CorexPM native binary not found at '${binaryPath}'.`);
      console.error("Please build the binary (`cargo build --workspace`) or reinstall `corexpm`.");
    } else {
      console.error(`[Error] Failed to launch CorexPM binary:`, result.error.message);
    }
    process.exit(1);
  }

  process.exit(result.status ?? 0);
}

main();
