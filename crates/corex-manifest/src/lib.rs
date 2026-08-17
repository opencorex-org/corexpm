//! Domain types for npm-compatible package manifests.
//!
//! JSON parsing will be introduced with the resolver milestone. Keeping the
//! domain model independent prevents registry and installer concerns from
//! leaking into manifest handling.

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

/// Minimal manifest representation needed by the initial resolver.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PackageManifest {
    /// Optional package name for private/root manifests.
    pub name: Option<PackageName>,
    /// Runtime dependency requirements.
    pub dependencies: BTreeMap<PackageName, String>,
    /// Development-only dependency requirements.
    pub dev_dependencies: BTreeMap<PackageName, String>,
    /// Optional dependency requirements.
    pub optional_dependencies: BTreeMap<PackageName, String>,
    /// Peer dependency requirements.
    pub peer_dependencies: BTreeMap<PackageName, String>,
}

/// Errors emitted while validating package manifest values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestError {
    /// The supplied npm package name is structurally invalid.
    InvalidPackageName,
}

#[cfg(test)]
mod tests {
    use super::PackageName;

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
}
