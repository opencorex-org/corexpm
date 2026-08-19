//! npm-compatible semantic version range matching.

use corex_errors::{Diagnostic, ErrorFamily};
use std::fmt;
use std::str::FromStr;

/// A validated semantic version.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version(js_semver::Version);

impl std::hash::Hash for Version {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Version {
    /// Parses a version string.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] when the value is not a valid semantic version.
    pub fn parse(value: &str) -> Result<Self, Diagnostic> {
        js_semver::Version::parse(value).map(Self).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                1,
                format!("invalid semantic version `{value}`: {e}"),
            )
        })
    }

    /// Returns the string representation.
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for Version {
    type Err = Diagnostic;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// An npm-compatible semantic version range constraint.
#[derive(Clone, Debug)]
pub struct Range(js_semver::Range);

impl Range {
    /// Parses an npm-compatible semver range.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] when the range expression is invalid.
    pub fn parse(value: &str) -> Result<Self, Diagnostic> {
        js_semver::Range::parse(value).map(Self).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Resolve,
                1,
                format!("invalid semantic version range `{value}`: {e}"),
            )
        })
    }

    /// Checks if the given version satisfies this range constraint.
    #[must_use]
    pub fn satisfies(&self, version: &Version) -> bool {
        self.0.satisfies(&version.0)
    }
}

impl FromStr for Range {
    type Err = Diagnostic;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string() == other.0.to_string()
    }
}

impl Eq for Range {}

impl serde::Serialize for Range {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Range {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert!(Version::parse("1.2.3").is_ok());
        assert!(Version::parse("1.2.3-alpha.1").is_ok());
        assert!(Version::parse("invalid").is_err());
    }

    #[test]
    fn test_range_matching() {
        let range = Range::parse("^1.2.3").unwrap();
        assert!(range.satisfies(&Version::parse("1.2.3").unwrap()));
        assert!(range.satisfies(&Version::parse("1.5.0").unwrap()));
        assert!(!range.satisfies(&Version::parse("2.0.0").unwrap()));
        assert!(!range.satisfies(&Version::parse("1.2.2").unwrap()));
    }
}
