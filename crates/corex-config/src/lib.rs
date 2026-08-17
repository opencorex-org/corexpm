//! Configuration types shared by `CorexPM` components.

/// Project installation strategy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LinkerMode {
    /// A strict, tool-compatible `node_modules` layout backed by the global CAS.
    #[default]
    Isolated,
    /// A future loader-based layout without traditional `node_modules`.
    Virtual,
}

/// Default policy for dependency lifecycle scripts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProjectConfig {
    /// Selected installation strategy.
    pub linker: LinkerMode,
    /// Default dependency lifecycle policy.
    pub scripts: ScriptPolicy,
    /// Whether registry access should be avoided completely.
    pub offline: bool,
}
