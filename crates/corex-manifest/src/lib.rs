//! Domain types for npm-compatible package manifests.

use corex_errors::{Diagnostic, ErrorFamily};
use std::collections::BTreeMap;

/// A package name, including optional npm scope.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PackageName(String);

impl PackageName {
    /// Creates a package name after minimal structural validation.
    ///
    /// # Errors
    ///
    /// Returns [`ManifestError::InvalidPackageName`] when the value is empty,
    /// contains whitespace, or begins an npm scope without a package segment.
    pub fn parse(value: impl Into<String>) -> Result<Self, ManifestError> {
        let value = value.into();
        let valid = !value.is_empty()
            && !value.chars().any(char::is_whitespace)
            && (!value.starts_with('@') || value.split_once('/').is_some());
        if valid {
            Ok(Self(value))
        } else {
            Err(ManifestError::InvalidPackageName)
        }
    }

    /// Returns the original npm package name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl serde::Serialize for PackageName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for PackageName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse(s).map_err(serde::de::Error::custom)
    }
}

/// Minimal manifest representation needed by the initial resolver.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageManifest {
    /// Optional package name for private/root manifests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<PackageName>,
    /// Runtime dependency requirements.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<PackageName, String>,
    /// Development-only dependency requirements.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dev_dependencies: BTreeMap<PackageName, String>,
    /// Optional dependency requirements.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub optional_dependencies: BTreeMap<PackageName, String>,
    /// Peer dependency requirements.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub peer_dependencies: BTreeMap<PackageName, String>,
}

#[derive(serde::Deserialize)]
struct RawPackageManifest {
    name: Option<String>,
    dependencies: Option<BTreeMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<BTreeMap<String, String>>,
    #[serde(rename = "optionalDependencies")]
    optional_dependencies: Option<BTreeMap<String, String>>,
    #[serde(rename = "peerDependencies")]
    peer_dependencies: Option<BTreeMap<String, String>>,
}

impl PackageManifest {
    /// Parses and validates a `package.json` file.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] when the JSON syntax is invalid or
    /// any package names fail validation.
    pub fn parse_json(content: &str) -> Result<Self, Diagnostic> {
        let raw: RawPackageManifest = serde_json::from_str(content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                2,
                format!("failed to parse package.json: {e}"),
            )
            .with_help(format!(
                "invalid JSON at line {}, column {}",
                e.line(),
                e.column()
            ))
        })?;

        let name = if let Some(n) = raw.name {
            Some(PackageName::parse(&n).map_err(|_| {
                Diagnostic::new(
                    ErrorFamily::Resolve,
                    2,
                    format!("invalid package name `{n}` in package.json"),
                )
                .with_help("package names cannot contain spaces or be empty, and scoped names must match @scope/name")
            })?)
        } else {
            None
        };

        let convert_deps = |deps: Option<BTreeMap<String, String>>| -> Result<BTreeMap<PackageName, String>, Diagnostic> {
            let mut result = BTreeMap::new();
            if let Some(d) = deps {
                for (k, v) in d {
                    let name = PackageName::parse(&k).map_err(|_| {
                        Diagnostic::new(
                            ErrorFamily::Resolve,
                            2,
                            format!("invalid dependency package name `{k}` in package.json"),
                        )
                        .with_help("package names cannot contain spaces or be empty, and scoped names must match @scope/name")
                    })?;
                    result.insert(name, v);
                }
            }
            Ok(result)
        };

        let dependencies = convert_deps(raw.dependencies)?;
        let dev_dependencies = convert_deps(raw.dev_dependencies)?;
        let optional_dependencies = convert_deps(raw.optional_dependencies)?;
        let peer_dependencies = convert_deps(raw.peer_dependencies)?;

        Ok(Self {
            name,
            dependencies,
            dev_dependencies,
            optional_dependencies,
            peer_dependencies,
        })
    }
}

/// Errors emitted while validating package manifest values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestError {
    /// The supplied npm package name is structurally invalid.
    InvalidPackageName,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPackageName => write!(f, "invalid package name"),
        }
    }
}

impl std::error::Error for ManifestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_scoped_names() {
        assert_eq!(
            PackageName::parse("@corex/example").unwrap().as_str(),
            "@corex/example"
        );
    }

    #[test]
    fn rejects_incomplete_scopes() {
        assert!(PackageName::parse("@corex").is_err());
    }

    #[test]
    fn test_parse_valid_json() {
        let content = r#"{
            "name": "@corex/example",
            "dependencies": {
                "react": "^18.0.0"
            },
            "devDependencies": {
                "typescript": "^5.0.0"
            }
        }"#;

        let manifest = PackageManifest::parse_json(content).unwrap();
        assert_eq!(manifest.name.unwrap().as_str(), "@corex/example");
        assert_eq!(manifest.dependencies.len(), 1);
        assert_eq!(manifest.dev_dependencies.len(), 1);
    }

    #[test]
    fn test_parse_invalid_json() {
        let content = r#"{
            "name": "@corex/example",
            "dependencies": {
                "react":
            }
        }"#;

        let res = PackageManifest::parse_json(content);
        assert!(res.is_err());
        let diag = res.unwrap_err();
        assert_eq!(diag.code(), "CXRESOLVE0002");
        assert!(diag.help().unwrap().contains("line"));
    }

    #[test]
    fn test_parse_invalid_package_name() {
        let content = r#"{
            "name": "invalid name with spaces"
        }"#;

        let res = PackageManifest::parse_json(content);
        assert!(res.is_err());
        let diag = res.unwrap_err();
        assert_eq!(diag.code(), "CXRESOLVE0002");
    }
}
