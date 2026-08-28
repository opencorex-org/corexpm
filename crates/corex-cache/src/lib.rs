//! Registry metadata and tarball conditional cache implementation.

#![forbid(unsafe_code)]

use corex_errors::{Diagnostic, ErrorFamily};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Operating modes for network and local cache operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CacheMode {
    /// Normal online mode: fetch remote, update cache.
    #[default]
    Online,
    /// Prefer offline: use cached entry if available, fetch remote if missing.
    PreferOffline,
    /// Strictly offline: fail if requested resource is not present in local cache.
    Offline,
}

/// Detailed status and disk space summary of the local cache.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CacheStatus {
    /// Absolute filesystem path to the cache root directory.
    pub path: PathBuf,
    /// Number of cached registry metadata documents.
    pub metadata_count: usize,
    /// Number of cached tarball archives.
    pub tarball_count: usize,
    /// Total byte size of all cached files.
    pub total_bytes: u64,
}

/// Manages local disk caching for registry responses and package tarballs.
#[derive(Clone, Debug)]
pub struct CacheManager {
    root: PathBuf,
    mode: CacheMode,
}

impl CacheManager {
    /// Creates a new `CacheManager` with given root directory and mode.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, mode: CacheMode) -> Self {
        Self {
            root: root.into(),
            mode,
        }
    }

    /// Returns reference to the cache root path.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns current cache mode.
    #[must_use]
    pub fn mode(&self) -> CacheMode {
        self.mode
    }

    /// Directory storing cached registry metadata JSON responses.
    #[must_use]
    pub fn registry_dir(&self) -> PathBuf {
        self.root.join("registry")
    }

    /// Directory storing downloaded raw package tarball archives.
    #[must_use]
    pub fn tarballs_dir(&self) -> PathBuf {
        self.root.join("tarballs")
    }

    fn sanitize_name(name: &str) -> String {
        name.replace('/', "__").replace('@', "_")
    }

    fn metadata_path(&self, package_name: &str) -> PathBuf {
        let filename = format!("{}.json", Self::sanitize_name(package_name));
        self.registry_dir().join(filename)
    }

    fn tarball_path(&self, tarball_hash: &str) -> PathBuf {
        let safe_hash = if tarball_hash.is_empty() {
            "empty".to_owned()
        } else {
            tarball_hash.replace(['/', '\\', ':'], "_")
        };
        self.tarballs_dir().join(format!("{safe_hash}.tgz"))
    }

    /// Retrieves cached metadata for `package_name` if present and allowed by current `CacheMode`.
    ///
    /// # Errors
    /// Returns `Diagnostic` if offline mode is active and metadata is missing, or on read failure.
    pub fn get_metadata(&self, package_name: &str) -> Result<Option<String>, Diagnostic> {
        let p = self.metadata_path(package_name);
        if p.exists() {
            let content = fs::read_to_string(&p).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Registry,
                    1,
                    format!("failed reading cached metadata for `{package_name}`: {e}"),
                )
            })?;
            return Ok(Some(content));
        }

        if self.mode == CacheMode::Offline {
            return Err(Diagnostic::new(
                ErrorFamily::Registry,
                2,
                format!("offline mode: metadata for `{package_name}` is not in local cache"),
            )
            .with_help("run corexpm online or run without --offline to update cache"));
        }

        Ok(None)
    }

    /// Caches registry metadata JSON response for `package_name`.
    ///
    /// # Errors
    /// Returns `Diagnostic` if writing to disk fails.
    pub fn put_metadata(&self, package_name: &str, metadata_json: &str) -> Result<(), Diagnostic> {
        let dir = self.registry_dir();
        fs::create_dir_all(&dir).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Registry,
                1,
                format!(
                    "failed creating metadata cache directory `{}`: {e}",
                    dir.display()
                ),
            )
        })?;

        let p = self.metadata_path(package_name);
        fs::write(&p, metadata_json).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Registry,
                1,
                format!("failed writing metadata cache for `{package_name}`: {e}"),
            )
        })?;

        Ok(())
    }

    /// Retrieves cached tarball bytes for `tarball_key` if available.
    ///
    /// # Errors
    /// Returns `Diagnostic` if offline mode is active and tarball is missing, or on read failure.
    pub fn get_tarball(&self, tarball_key: &str) -> Result<Option<Vec<u8>>, Diagnostic> {
        let key = if tarball_key.starts_with("sha") || tarball_key.len() >= 32 {
            tarball_key.to_owned()
        } else {
            let mut hasher = Sha256::new();
            hasher.update(tarball_key.as_bytes());
            hex::encode(hasher.finalize())
        };

        let p = self.tarball_path(&key);
        if p.exists() {
            let bytes = fs::read(&p).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    1,
                    format!("failed reading cached tarball `{key}`: {e}"),
                )
            })?;
            return Ok(Some(bytes));
        }

        if self.mode == CacheMode::Offline {
            return Err(Diagnostic::new(
                ErrorFamily::Store,
                2,
                format!("offline mode: tarball `{tarball_key}` is not in local cache"),
            )
            .with_help("connect to registry network or provide local fixture tarballs"));
        }

        Ok(None)
    }

    /// Stores package tarball archive bytes into local cache.
    ///
    /// # Errors
    /// Returns `Diagnostic` if writing to cache directory fails.
    pub fn put_tarball(&self, tarball_key: &str, bytes: &[u8]) -> Result<PathBuf, Diagnostic> {
        let dir = self.tarballs_dir();
        fs::create_dir_all(&dir).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed creating tarball cache dir `{}`: {e}", dir.display()),
            )
        })?;

        let key = if tarball_key.starts_with("sha") || tarball_key.len() >= 32 {
            tarball_key.to_owned()
        } else {
            let mut hasher = Sha256::new();
            hasher.update(tarball_key.as_bytes());
            hex::encode(hasher.finalize())
        };

        let p = self.tarball_path(&key);
        fs::write(&p, bytes).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed writing tarball cache `{key}`: {e}"),
            )
        })?;

        Ok(p)
    }

    /// Computes summary stats of all cached metadata and tarball files.
    ///
    /// # Errors
    /// Returns `Diagnostic` if directory scan fails.
    pub fn status(&self) -> Result<CacheStatus, Diagnostic> {
        let mut metadata_count = 0usize;
        let mut tarball_count = 0usize;
        let mut total_bytes = 0u64;

        let reg_dir = self.registry_dir();
        if reg_dir.exists() {
            if let Ok(entries) = fs::read_dir(&reg_dir) {
                for entry in entries.flatten() {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_file() {
                            metadata_count += 1;
                            total_bytes += meta.len();
                        }
                    }
                }
            }
        }

        let tb_dir = self.tarballs_dir();
        if tb_dir.exists() {
            if let Ok(entries) = fs::read_dir(&tb_dir) {
                for entry in entries.flatten() {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_file() {
                            tarball_count += 1;
                            total_bytes += meta.len();
                        }
                    }
                }
            }
        }

        Ok(CacheStatus {
            path: self.root.clone(),
            metadata_count,
            tarball_count,
            total_bytes,
        })
    }

    /// Cleans and removes all cached metadata and tarball contents.
    ///
    /// # Errors
    /// Returns `Diagnostic` if cache directory removal fails.
    pub fn clean(&self) -> Result<(), Diagnostic> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    1,
                    format!(
                        "failed cleaning cache directory `{}`: {e}",
                        self.root.display()
                    ),
                )
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_metadata_put_get() {
        let temp_dir = std::env::temp_dir().join("corex_test_cache_meta");
        let _ = fs::remove_dir_all(&temp_dir);

        let manager = CacheManager::new(&temp_dir, CacheMode::Online);
        assert!(manager.get_metadata("react").unwrap().is_none());

        let sample_json = r#"{"name": "react"}"#;
        manager.put_metadata("react", sample_json).unwrap();

        let fetched = manager.get_metadata("react").unwrap().unwrap();
        assert_eq!(fetched, sample_json);

        let status = manager.status().unwrap();
        assert_eq!(status.metadata_count, 1);
        assert!(status.total_bytes > 0);

        manager.clean().unwrap();
        assert!(!temp_dir.exists());
    }

    #[test]
    fn test_cache_offline_mode() {
        let temp_dir = std::env::temp_dir().join("corex_test_cache_offline");
        let _ = fs::remove_dir_all(&temp_dir);

        let manager = CacheManager::new(&temp_dir, CacheMode::Offline);
        let res = manager.get_metadata("nonexistent");
        assert!(res.is_err());
        assert!(res.unwrap_err().code().starts_with("CXREG"));
    }
}
