//! npm registry client abstraction, HTTP client, and mock client implementation.

#![forbid(unsafe_code)]

use corex_errors::{Diagnostic, ErrorFamily};
use corex_manifest::PackageName;
use corex_semver::Version;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

/// Package distribution metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistryDist {
    /// URL of the package tarball.
    pub tarball: String,
    /// Expected integrity hash (e.g., sha512 or sha1).
    #[serde(default)]
    pub integrity: String,
    /// Legacy SHA-1 checksum hex string.
    #[serde(default)]
    pub shasum: Option<String>,
}

/// Metadata returned from the registry for a specific version.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistryVersionMetadata {
    /// Semantic version.
    pub version: Version,
    /// Runtime dependencies.
    #[serde(default)]
    pub dependencies: BTreeMap<PackageName, String>,
    /// Development-only dependencies.
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: BTreeMap<PackageName, String>,
    /// Optional dependencies.
    #[serde(default, rename = "optionalDependencies")]
    pub optional_dependencies: BTreeMap<PackageName, String>,
    /// Peer dependencies.
    #[serde(default, rename = "peerDependencies")]
    pub peer_dependencies: BTreeMap<PackageName, String>,
    /// Distribution information.
    pub dist: RegistryDist,
    /// Target engine constraints.
    #[serde(default)]
    pub engines: BTreeMap<String, String>,
    /// Target operating systems.
    #[serde(default)]
    pub os: Vec<String>,
    /// Target CPU architectures.
    #[serde(default)]
    pub cpu: Vec<String>,
}

/// Registry metadata for all versions of a package.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistryPackageMetadata {
    /// Original package name.
    pub name: PackageName,
    /// Distribution tags (e.g. latest -> 1.0.0).
    #[serde(default, rename = "dist-tags")]
    pub dist_tags: BTreeMap<String, String>,
    /// Mapping of all versions to their metadata.
    pub versions: BTreeMap<Version, RegistryVersionMetadata>,
}

/// Client interface for interacting with npm registry metadata.
pub trait RegistryClient: Send + Sync {
    /// Fetches package metadata from the registry.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] when the request or parsing fails.
    fn fetch_metadata(&self, name: &PackageName) -> Result<RegistryPackageMetadata, Diagnostic>;
}

impl<T: ?Sized + RegistryClient> RegistryClient for Box<T> {
    fn fetch_metadata(&self, name: &PackageName) -> Result<RegistryPackageMetadata, Diagnostic> {
        (**self).fetch_metadata(name)
    }
}

impl<T: ?Sized + RegistryClient> RegistryClient for &T {
    fn fetch_metadata(&self, name: &PackageName) -> Result<RegistryPackageMetadata, Diagnostic> {
        (**self).fetch_metadata(name)
    }
}

/// A real HTTP client for fetching metadata and tarballs from npm registries.
#[derive(Clone, Debug)]
pub struct HttpRegistryClient {
    base_url: String,
    auth_token: Option<String>,
    timeout: Duration,
}

impl Default for HttpRegistryClient {
    fn default() -> Self {
        Self::new("https://registry.npmjs.org")
    }
}

impl HttpRegistryClient {
    /// Creates a new `HttpRegistryClient` targeting the specified registry base URL.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut url = base_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        Self {
            base_url: url,
            auth_token: None,
            timeout: Duration::from_secs(30),
        }
    }

    /// Sets an optional Bearer authentication token for private registries.
    #[must_use]
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Sets the HTTP request timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Downloads a raw package tarball from a URL.
    ///
    /// # Errors
    /// Returns a [`Diagnostic`] if network download or connection fails.
    pub fn fetch_tarball(&self, tarball_url: &str) -> Result<Vec<u8>, Diagnostic> {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(self.timeout)
            .timeout_read(self.timeout)
            .build();

        let mut req = agent.get(tarball_url);
        req = req.set(
            "User-Agent",
            "corexpm/0.1.0 (https://github.com/opencorex-org/corexpm)",
        );

        if let Some(ref token) = self.auth_token {
            req = req.set("Authorization", &format!("Bearer {token}"));
        }

        let resp = req.call().map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Registry,
                10,
                format!("failed to fetch tarball from `{tarball_url}`: {e}"),
            )
            .with_help("check your internet connection or registry availability")
        })?;

        let mut bytes = Vec::new();
        let mut reader = resp.into_reader();
        std::io::Read::read_to_end(&mut reader, &mut bytes).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Registry,
                11,
                format!("failed reading tarball bytes from `{tarball_url}`: {e}"),
            )
        })?;

        Ok(bytes)
    }
}

impl RegistryClient for HttpRegistryClient {
    fn fetch_metadata(&self, name: &PackageName) -> Result<RegistryPackageMetadata, Diagnostic> {
        let raw_name = name.as_str();
        let encoded_name = if raw_name.starts_with('@') {
            raw_name.replace('/', "%2F")
        } else {
            raw_name.to_string()
        };

        let url = format!("{}/{}", self.base_url, encoded_name);

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(self.timeout)
            .timeout_read(self.timeout)
            .build();

        let mut req = agent.get(&url);
        req = req
            .set(
                "Accept",
                "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*",
            )
            .set(
                "User-Agent",
                "corexpm/0.1.0 (https://github.com/opencorex-org/corexpm)",
            );

        if let Some(ref token) = self.auth_token {
            req = req.set("Authorization", &format!("Bearer {token}"));
        }

        let resp = req.call().map_err(|e| match e {
            ureq::Error::Status(404, _) => Diagnostic::new(
                ErrorFamily::Registry,
                404,
                format!(
                    "package `{raw_name}` not found in registry `{}`",
                    self.base_url
                ),
            )
            .with_help("verify that the package name is spelled correctly and published"),
            ureq::Error::Status(401 | 403, _) => Diagnostic::new(
                ErrorFamily::Registry,
                403,
                format!(
                    "authentication required or forbidden for `{raw_name}` in registry `{}`",
                    self.base_url
                ),
            )
            .with_help("check your npm authentication token in ~/.npmrc or corex configuration"),
            other => Diagnostic::new(
                ErrorFamily::Registry,
                1,
                format!("failed to fetch metadata for `{raw_name}` from `{url}`: {other}"),
            )
            .with_help("check your internet connection or registry availability"),
        })?;

        let text = resp.into_string().map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Registry,
                2,
                format!("failed reading registry response for `{raw_name}`: {e}"),
            )
        })?;

        let mut metadata: RegistryPackageMetadata = serde_json::from_str(&text).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Registry,
                3,
                format!("failed to parse registry packument for `{raw_name}`: {e}"),
            )
        })?;

        // Fill missing integrity from shasum if needed
        for ver_meta in metadata.versions.values_mut() {
            if ver_meta.dist.integrity.is_empty() {
                if let Some(ref shasum) = ver_meta.dist.shasum {
                    ver_meta.dist.integrity = shasum.clone();
                }
            }
        }

        Ok(metadata)
    }
}

/// A mock registry client loading local JSON files.
#[derive(Debug)]
pub struct MockRegistryClient {
    fixtures_dir: PathBuf,
}

impl MockRegistryClient {
    /// Creates a new `MockRegistryClient` reading from the specified directory.
    #[must_use]
    pub fn new(fixtures_dir: impl Into<PathBuf>) -> Self {
        Self {
            fixtures_dir: fixtures_dir.into(),
        }
    }
}

impl RegistryClient for MockRegistryClient {
    fn fetch_metadata(&self, name: &PackageName) -> Result<RegistryPackageMetadata, Diagnostic> {
        let safe_name = name.as_str().replace('/', "__").replace('@', "_");
        let path = self.fixtures_dir.join(format!("{safe_name}.json"));
        if !path.exists() {
            return Err(Diagnostic::new(
                ErrorFamily::Registry,
                1,
                format!("mock package metadata not found for `{}`", name.as_str()),
            )
            .with_help(format!("expected fixture file at `{}`", path.display())));
        }

        let content = std::fs::read_to_string(&path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Registry,
                2,
                format!("failed to read mock fixture for `{}`: {e}", name.as_str()),
            )
        })?;

        let mut metadata: RegistryPackageMetadata =
            serde_json::from_str(&content).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Registry,
                    3,
                    format!("failed to parse mock fixture for `{}`: {e}", name.as_str()),
                )
            })?;

        for ver_meta in metadata.versions.values_mut() {
            if ver_meta.dist.integrity.is_empty() {
                if let Some(ref shasum) = ver_meta.dist.shasum {
                    ver_meta.dist.integrity = shasum.clone();
                }
            }
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_creation() {
        let client = HttpRegistryClient::default();
        assert_eq!(client.base_url, "https://registry.npmjs.org");
        let custom =
            HttpRegistryClient::new("https://custom.registry.io/").with_auth_token("secret-token");
        assert_eq!(custom.base_url, "https://custom.registry.io");
        assert_eq!(custom.auth_token.as_deref(), Some("secret-token"));
    }
}
