//! Lifecycle and package script process control with secret redaction and build overlays.

#![forbid(unsafe_code)]

use corex_errors::{Diagnostic, ErrorFamily};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// npm-compatible dependency lifecycle hooks.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LifecycleHook {
    /// `preinstall` hook.
    PreInstall,
    /// `install` hook.
    Install,
    /// `postinstall` hook.
    PostInstall,
    /// `prepare` hook.
    Prepare,
}

impl LifecycleHook {
    /// Returns standard npm lifecycle hook name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreInstall => "preinstall",
            Self::Install => "install",
            Self::PostInstall => "postinstall",
            Self::Prepare => "prepare",
        }
    }
}

/// Execution outcome for a package script.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptExecutionResult {
    /// Target package name.
    pub package_name: String,
    /// Script hook or name.
    pub hook: String,
    /// Raw script command string executed.
    pub command: String,
    /// True if command exited with code 0.
    pub success: bool,
    /// Redacted process stdout output.
    pub stdout: String,
    /// Redacted process stderr output.
    pub stderr: String,
    /// Execution duration in milliseconds.
    pub duration_ms: u128,
}

/// Script executor handling sanitized environment, secret redaction, and build overlays.
#[derive(Clone, Debug, Default)]
pub struct ScriptExecutor;

impl ScriptExecutor {
    /// Creates a new `ScriptExecutor`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Redacts sensitive tokens, headers, and secret strings from output text.
    #[must_use]
    pub fn redact_secrets(input: &str) -> String {
        let mut result = input.to_string();
        let sensitive_keys = ["bearer ", "ghp_", "npm_", "secret_", "api_key="];

        for key in sensitive_keys {
            if let Some(pos) = result.to_lowercase().find(key) {
                let end = result[pos..]
                    .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '\n')
                    .map_or(result.len(), |e| pos + e);
                result.replace_range(pos..end, "[REDACTED]");
            }
        }
        result
    }

    /// Creates a writable build overlay directory for running lifecycle scripts without mutating CAS source.
    ///
    /// # Errors
    /// Returns `Diagnostic` if copying package files fails.
    pub fn create_build_overlay(src_dir: &Path, overlay_dir: &Path) -> Result<(), Diagnostic> {
        if overlay_dir.exists() {
            let _ = fs::remove_dir_all(overlay_dir);
        }
        copy_dir_all(src_dir, overlay_dir)?;
        Ok(())
    }

    /// Executes a package lifecycle script or custom command string.
    ///
    /// # Errors
    /// Returns `Diagnostic` if process spawning fails.
    pub fn execute_script(
        &self,
        project_root: &Path,
        working_dir: &Path,
        package_name: &str,
        hook: &str,
        command_str: &str,
    ) -> Result<ScriptExecutionResult, Diagnostic> {
        let start_time = Instant::now();
        let bin_dir = project_root.join("node_modules").join(".bin");

        let current_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{current_path}", bin_dir.display());

        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/C", command_str]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", command_str]);
            c
        };

        cmd.current_dir(working_dir)
            .env("PATH", new_path)
            .env("COREX_GUARD", "1")
            .env("NODE_ENV", "development");

        let output = cmd.output().map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Script,
                1,
                format!("failed launching script process for `{package_name}` ({hook}): {e}"),
            )
        })?;

        let raw_stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let raw_stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let stdout = Self::redact_secrets(&raw_stdout);
        let stderr = Self::redact_secrets(&raw_stderr);

        Ok(ScriptExecutionResult {
            package_name: package_name.to_string(),
            hook: hook.to_string(),
            command: command_str.to_string(),
            success: output.status.success(),
            stdout,
            stderr,
            duration_ms: start_time.elapsed().as_millis(),
        })
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), Diagnostic> {
    if src.is_file() {
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
            }
        }
        fs::copy(src, dst).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Script,
                2,
                format!(
                    "failed copying file `{}` to `{}`: {e}",
                    src.display(),
                    dst.display()
                ),
            )
        })?;
        return Ok(());
    }

    fs::create_dir_all(dst).map_err(|e| {
        Diagnostic::new(
            ErrorFamily::Script,
            3,
            format!("failed creating directory `{}`: {e}", dst.display()),
        )
    })?;

    for entry in fs::read_dir(src)
        .map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Script,
                4,
                format!("failed reading directory `{}`: {e}", src.display()),
            )
        })?
        .flatten()
    {
        let path = entry.path();
        let target_path = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_all(&path, &target_path)?;
        } else if path.is_file() {
            fs::copy(&path, &target_path).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Script,
                    5,
                    format!("failed copying file `{}`: {e}", path.display()),
                )
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_redaction() {
        let raw = "Installing with token bearer ghp_123456789SecretKey string";
        let redacted = ScriptExecutor::redact_secrets(raw);
        assert!(!redacted.contains("ghp_123456789SecretKey"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_execute_echo_script() {
        let temp_dir = env::temp_dir().join("corex_test_script_exec");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let executor = ScriptExecutor::new();
        let res = executor
            .execute_script(
                &temp_dir,
                &temp_dir,
                "test-pkg",
                "postinstall",
                "echo Hello CorexPM",
            )
            .unwrap();

        assert!(res.success);
        assert!(res.stdout.contains("Hello CorexPM"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
