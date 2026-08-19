//! Conditional registry metadata and package tarball caches.

use corex_config::ProjectConfig;
use corex_errors::{Diagnostic, ErrorFamily};
use std::path::PathBuf;

/// Caching policies profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheProfile {
    /// Normal caching behavior.
    Default,
    /// Low disk profile cleans up tarballs immediately.
    LowDisk,
    /// Offline heavy keeps archives indefinitely.
    OfflineHeavy,
}

/// Cache manager directory locator and metadata cache.
#[derive(Debug)]
pub struct CacheManager {
    registry_cache_dir: PathBuf,
    tarball_cache_dir: PathBuf,
    profile: CacheProfile,
}

impl CacheManager {
    /// Creates a `CacheManager` under the specified root directory.
    #[must_use]
    pub fn new(root_dir: impl Into<PathBuf>, config: &ProjectConfig) -> Self {
        let root = root_dir.into();
        let profile = if config.offline {
            CacheProfile::OfflineHeavy
        } else {
            CacheProfile::Default
        };
        Self {
            registry_cache_dir: root.join("cache").join("registry"),
            tarball_cache_dir: root.join("cache").join("tarballs"),
            profile,
        }
    }

    /// Returns the active cache profile.
    #[must_use]
    pub const fn profile(&self) -> CacheProfile {
        self.profile
    }

    /// Sets custom cache profile.
    #[must_use]
    pub const fn with_profile(mut self, profile: CacheProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Locates the cached metadata path for a package.
    #[must_use]
    pub fn get_metadata_path(&self, package_name: &str) -> PathBuf {
        let safe_name = package_name.replace('/', "__").replace('@', "_");
        self.registry_cache_dir.join(format!("{safe_name}.json"))
    }

    /// Locates the cached tarball path for a package version.
    #[must_use]
    pub fn get_tarball_path(&self, package_name: &str, version: &str) -> PathBuf {
        let safe_name = package_name.replace('/', "__").replace('@', "_");
        self.tarball_cache_dir
            .join(format!("{safe_name}-{version}.tgz"))
    }

    /// Reads package metadata cache if exists.
    #[must_use]
    pub fn read_metadata(&self, package_name: &str) -> Option<String> {
        let path = self.get_metadata_path(package_name);
        if path.exists() {
            std::fs::read_to_string(path).ok()
        } else {
            None
        }
    }

    /// Writes package metadata to cache.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if directories cannot be created or file writing fails.
    pub fn write_metadata(&self, package_name: &str, content: &str) -> Result<(), Diagnostic> {
        std::fs::create_dir_all(&self.registry_cache_dir).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                27,
                format!("failed to create registry cache directory: {e}"),
            )
        })?;
        let path = self.get_metadata_path(package_name);
        std::fs::write(&path, content).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                28,
                format!(
                    "failed to write registry metadata cache file `{}`: {e}",
                    path.display()
                ),
            )
        })
    }
}
