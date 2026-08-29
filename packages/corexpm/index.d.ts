/**
 * TypeScript SDK definitions for CorexPM.
 */

export interface ExecResult<T = any> {
  code: number;
  stdout: string;
  stderr: string;
  data?: T;
}

export interface ExecOptions {
  cwd?: string;
  env?: Record<string, string>;
}

export interface InstallOptions extends ExecOptions {
  frozen?: boolean;
  offline?: boolean;
  linker?: "isolated" | "hoisted" | "symlink";
}

export interface AuditOptions extends ExecOptions {
  severity?: "low" | "medium" | "high" | "critical";
  ignore?: string[];
}

export function execCorex(args?: string[], options?: ExecOptions): ExecResult;
export function install(options?: InstallOptions): ExecResult;
export function migrate(options?: ExecOptions): ExecResult;
export function audit(options?: AuditOptions): ExecResult;
export function getStoreStatus(options?: ExecOptions): ExecResult;
export function doctor(options?: ExecOptions): ExecResult;
export function resolveBinaryPath(): string;
