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
  const pkgRoot = __dirname;
  const workspaceRoot = path.join(pkgRoot, "..", "..");

  const releaseBin = path.join(workspaceRoot, "target", "release", binaryName);
  if (fs.existsSync(releaseBin)) {
    return releaseBin;
  }

  const debugBin = path.join(workspaceRoot, "target", "debug", binaryName);
  if (fs.existsSync(debugBin)) {
    return debugBin;
  }

  const vendorBin = path.join(pkgRoot, "vendor", `${os.platform()}-${os.arch()}`, binaryName);
  if (fs.existsSync(vendorBin)) {
    return vendorBin;
  }

  return binaryName;
}

/**
 * Executes a CorexPM CLI command programmatically.
 *
 * @param {string[]} args CLI arguments (e.g. ['install', '--frozen', '--json'])
 * @param {object} [options] Options for execution
 * @param {string} [options.cwd] Current working directory
 * @param {object} [options.env] Environment variables
 * @returns {{ code: number, stdout: string, stderr: string, data?: any }} Execution result
 */
function execCorex(args = [], options = {}) {
  const binaryPath = resolveBinaryPath();
  const result = spawnSync(binaryPath, args, {
    cwd: options.cwd || process.cwd(),
    env: { ...process.env, ...options.env },
    encoding: "utf8",
    windowsHide: true,
  });

  if (result.error) {
    throw new Error(`Failed to execute CorexPM: ${result.error.message}`);
  }

  const stdout = result.stdout || "";
  const stderr = result.stderr || "";
  let data;

  if (args.includes("--json") && stdout.trim()) {
    try {
      data = JSON.parse(stdout.trim());
    } catch {
      // Non-JSON output fallback
    }
  }

  return {
    code: result.status ?? 1,
    stdout,
    stderr,
    data,
  };
}

/**
 * Programmatically runs `corexpm install`.
 */
function install(options = {}) {
  const args = ["install", "--json"];
  if (options.frozen) args.push("--frozen");
  if (options.offline) args.push("--offline");
  if (options.linker) args.push(`--linker=${options.linker}`);
  return execCorex(args, options);
}

/**
 * Programmatically runs `corexpm migrate`.
 */
function migrate(options = {}) {
  return execCorex(["migrate", "--json"], options);
}

/**
 * Programmatically runs `corexpm audit`.
 */
function audit(options = {}) {
  const args = ["audit", "--json"];
  if (options.severity) args.push(`--severity=${options.severity}`);
  if (Array.isArray(options.ignore)) {
    options.ignore.forEach(id => args.push(`--ignore=${id}`));
  }
  return execCorex(args, options);
}

/**
 * Programmatically runs `corexpm store status`.
 */
function getStoreStatus(options = {}) {
  return execCorex(["store", "status", "--json"], options);
}

/**
 * Programmatically runs `corexpm doctor`.
 */
function doctor(options = {}) {
  return execCorex(["doctor", "--json"], options);
}

module.exports = {
  execCorex,
  install,
  migrate,
  audit,
  getStoreStatus,
  doctor,
  resolveBinaryPath,
};
