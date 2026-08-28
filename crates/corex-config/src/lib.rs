//! Configuration types shared by `CorexPM` components.

use corex_errors::{Diagnostic, ErrorFamily};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Project installation strategy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkerMode {
    /// A strict, tool-compatible `node_modules` layout backed by the global CAS.
    #[default]
    Isolated,
    /// A future loader-based layout without traditional `node_modules`.
    Virtual,
}

/// Default policy for dependency lifecycle scripts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptPolicy {
    /// Refuse scripts unless the package is explicitly trusted.
    #[default]
    Deny,
    /// Ask on interactive terminals and deny in non-interactive environments.
    Prompt,
    /// Allow scripts. Intended only for explicit compatibility profiles.
    Allow,
}

/// Effective project configuration after precedence has been resolved.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Selected installation strategy.
    pub linker: LinkerMode,
    /// Default dependency lifecycle policy.
    pub scripts: ScriptPolicy,
    /// Whether registry access should be avoided completely.
    pub offline: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTomlConfig {
    install: Option<RawInstallSection>,
    scripts: Option<RawScriptsSection>,
    network: Option<RawNetworkSection>,
    #[serde(rename = "workspace")]
    _workspace: Option<serde::de::IgnoredAny>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInstallSection {
    linker: Option<LinkerMode>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScriptsSection {
    default: Option<ScriptPolicy>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNetworkSection {
    offline: Option<bool>,
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

fn global_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".corex").join("corex.toml"))
}

fn find_project_root() -> Option<PathBuf> {
    let current = std::env::current_dir().ok()?;
    let mut current = current.as_path();
    loop {
        if current.join("corex.toml").exists() || current.join("package.json").exists() {
            return Some(current.to_path_buf());
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }
    None
}

fn parse_env_linker() -> Result<Option<LinkerMode>, Diagnostic> {
    if let Some(val) = std::env::var_os("COREX_LINKER") {
        let val_str = val.to_string_lossy();
        match val_str.trim().to_lowercase().as_str() {
            "isolated" => Ok(Some(LinkerMode::Isolated)),
            "virtual" => Ok(Some(LinkerMode::Virtual)),
            other => Err(Diagnostic::new(
                ErrorFamily::Cli,
                2,
                format!("invalid COREX_LINKER environment value: `{other}`"),
            )
            .with_help("supported values: isolated, virtual")),
        }
    } else {
        Ok(None)
    }
}

fn parse_env_scripts() -> Result<Option<ScriptPolicy>, Diagnostic> {
    if let Some(val) = std::env::var_os("COREX_SCRIPTS") {
        let val_str = val.to_string_lossy();
        match val_str.trim().to_lowercase().as_str() {
            "deny" => Ok(Some(ScriptPolicy::Deny)),
            "prompt" => Ok(Some(ScriptPolicy::Prompt)),
            "allow" => Ok(Some(ScriptPolicy::Allow)),
            other => Err(Diagnostic::new(
                ErrorFamily::Cli,
                2,
                format!("invalid COREX_SCRIPTS environment value: `{other}`"),
            )
            .with_help("supported values: deny, prompt, allow")),
        }
    } else {
        Ok(None)
    }
}

fn parse_env_offline() -> Result<Option<bool>, Diagnostic> {
    if let Some(val) = std::env::var_os("COREX_OFFLINE") {
        let val_str = val.to_string_lossy();
        match val_str.trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(Some(true)),
            "false" | "0" | "no" => Ok(Some(false)),
            other => Err(Diagnostic::new(
                ErrorFamily::Cli,
                2,
                format!("invalid COREX_OFFLINE environment value: `{other}`"),
            )
            .with_help("supported values: true, false")),
        }
    } else {
        Ok(None)
    }
}

fn load_toml_file(path: &Path) -> Result<Option<RawTomlConfig>, Diagnostic> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path).map_err(|e| {
        Diagnostic::new(
            ErrorFamily::Cli,
            2,
            format!(
                "failed to read configuration file at `{}`: {e}",
                path.display()
            ),
        )
    })?;
    let config: RawTomlConfig = toml::from_str(&content).map_err(|e| {
        Diagnostic::new(
            ErrorFamily::Cli,
            2,
            format!(
                "failed to parse configuration file at `{}`: {e}",
                path.display()
            ),
        )
    })?;
    Ok(Some(config))
}

/// Resolves configuration options using the standard precedence rules.
///
/// Precedence order (highest to lowest):
/// 1. CLI explicit overrides
/// 2. Environment variables (`COREX_LINKER`, `COREX_SCRIPTS`, `COREX_OFFLINE`)
/// 3. Local project configuration (`<project-root>/corex.toml`)
/// 4. User-global configuration (`~/.corex/corex.toml`)
/// 5. Defaults
///
/// # Errors
///
/// Returns a [`Diagnostic`] when parsing configuration files or environment variables fails.
pub fn resolve_config(
    project_root: Option<&Path>,
    cli_linker: Option<LinkerMode>,
    cli_scripts: Option<ScriptPolicy>,
    cli_offline: Option<bool>,
) -> Result<ProjectConfig, Diagnostic> {
    let global_config = global_config_path()
        .and_then(|p| load_toml_file(&p).transpose())
        .transpose()?;

    let resolved_project_root = project_root.map(PathBuf::from).or_else(find_project_root);

    let local_config = if let Some(ref root) = resolved_project_root {
        load_toml_file(&root.join("corex.toml"))?
    } else {
        None
    };

    let env_linker = parse_env_linker()?;
    let env_scripts = parse_env_scripts()?;
    let env_offline = parse_env_offline()?;

    let linker = cli_linker
        .or(env_linker)
        .or_else(|| {
            local_config
                .as_ref()
                .and_then(|c| c.install.as_ref().and_then(|i| i.linker))
        })
        .or_else(|| {
            global_config
                .as_ref()
                .and_then(|c| c.install.as_ref().and_then(|i| i.linker))
        })
        .unwrap_or_default();

    let scripts = cli_scripts
        .or(env_scripts)
        .or_else(|| {
            local_config
                .as_ref()
                .and_then(|c| c.scripts.as_ref().and_then(|s| s.default))
        })
        .or_else(|| {
            global_config
                .as_ref()
                .and_then(|c| c.scripts.as_ref().and_then(|s| s.default))
        })
        .unwrap_or_default();

    let offline = cli_offline
        .or(env_offline)
        .or_else(|| {
            local_config
                .as_ref()
                .and_then(|c| c.network.as_ref().and_then(|n| n.offline))
        })
        .or_else(|| {
            global_config
                .as_ref()
                .and_then(|c| c.network.as_ref().and_then(|n| n.offline))
        })
        .unwrap_or_default();

    Ok(ProjectConfig {
        linker,
        scripts,
        offline,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn env_lock() -> &'static std::sync::Mutex<()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
    }

    fn create_temp_dir() -> PathBuf {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut dir = std::env::temp_dir();
        dir.push(format!("corex_config_test_{pid}_{nanos}_{count}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_defaults() {
        let _lock = env_lock().lock().unwrap();
        let temp = create_temp_dir();
        std::env::remove_var("COREX_LINKER");
        std::env::remove_var("COREX_SCRIPTS");
        std::env::remove_var("COREX_OFFLINE");

        let config = resolve_config(Some(&temp), None, None, None).unwrap();
        assert_eq!(config.linker, LinkerMode::Isolated);
        assert_eq!(config.scripts, ScriptPolicy::Deny);
        assert!(!config.offline);

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn test_local_toml() {
        let _lock = env_lock().lock().unwrap();
        let temp = create_temp_dir();
        std::env::remove_var("COREX_LINKER");
        std::env::remove_var("COREX_SCRIPTS");
        std::env::remove_var("COREX_OFFLINE");

        let toml_content = r#"
            [install]
            linker = "virtual"
            [scripts]
            default = "allow"
            [network]
            offline = true
        "#;
        fs::write(temp.join("corex.toml"), toml_content).unwrap();

        let config = resolve_config(Some(&temp), None, None, None).unwrap();
        assert_eq!(config.linker, LinkerMode::Virtual);
        assert_eq!(config.scripts, ScriptPolicy::Allow);
        assert!(config.offline);

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn test_env_precedence() {
        let _lock = env_lock().lock().unwrap();
        let temp = create_temp_dir();
        std::env::remove_var("COREX_LINKER");
        std::env::remove_var("COREX_SCRIPTS");
        std::env::remove_var("COREX_OFFLINE");

        let toml_content = r#"
            [install]
            linker = "virtual"
        "#;
        fs::write(temp.join("corex.toml"), toml_content).unwrap();

        std::env::set_var("COREX_LINKER", "isolated");
        let config = resolve_config(Some(&temp), None, None, None).unwrap();
        assert_eq!(config.linker, LinkerMode::Isolated);

        std::env::remove_var("COREX_LINKER");
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn test_cli_precedence() {
        let _lock = env_lock().lock().unwrap();
        let temp = create_temp_dir();
        std::env::remove_var("COREX_LINKER");
        std::env::remove_var("COREX_SCRIPTS");
        std::env::remove_var("COREX_OFFLINE");

        std::env::set_var("COREX_LINKER", "isolated");

        let config = resolve_config(Some(&temp), Some(LinkerMode::Virtual), None, None).unwrap();
        assert_eq!(config.linker, LinkerMode::Virtual);

        std::env::remove_var("COREX_LINKER");
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn test_invalid_env() {
        let _lock = env_lock().lock().unwrap();
        std::env::remove_var("COREX_LINKER");
        std::env::remove_var("COREX_SCRIPTS");
        std::env::remove_var("COREX_OFFLINE");

        std::env::set_var("COREX_LINKER", "invalid");
        let res = resolve_config(None, None, None, None);
        assert!(res.is_err());
        std::env::remove_var("COREX_LINKER");
    }

    #[test]
    fn test_invalid_toml() {
        let _lock = env_lock().lock().unwrap();
        std::env::remove_var("COREX_LINKER");
        std::env::remove_var("COREX_SCRIPTS");
        std::env::remove_var("COREX_OFFLINE");

        let temp = create_temp_dir();
        fs::write(temp.join("corex.toml"), "invalid = toml = syntax").unwrap();
        let res = resolve_config(Some(&temp), None, None, None);
        assert!(res.is_err());
        fs::remove_dir_all(temp).unwrap();
    }
}
