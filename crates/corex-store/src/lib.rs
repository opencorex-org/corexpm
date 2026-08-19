//! Content-addressed storage (CAS) and safe archive extraction for `CorexPM`.

use corex_errors::{Diagnostic, ErrorFamily};

use std::path::{Path, PathBuf};

/// Statistics report for the Content-Addressed Store.
#[derive(Clone, Debug, serde::Serialize)]
pub struct StoreStats {
    /// Number of unique packages in the store.
    pub unique_packages: usize,
    /// Physical bytes occupied in the store.
    pub physical_bytes: u64,
    /// Logical bytes referenced by local projects.
    pub logical_bytes: u64,
    /// Ratio of logical bytes to physical bytes.
    pub reuse_ratio: f64,
}

/// Content-addressed package store manager.
#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct ContentAddressedStore {
    _root_dir: PathBuf,
    temp_dir: PathBuf,
    packages_dir: PathBuf,
}

impl ContentAddressedStore {
    /// Creates a new `ContentAddressedStore` under the specified root path.
    #[must_use]
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        let root = root_dir.into();
        Self {
            temp_dir: root.join("store").join("v1").join("temp"),
            packages_dir: root
                .join("store")
                .join("v1")
                .join("packages")
                .join("sha256"),
            _root_dir: root,
        }
    }

    /// Returns the global packages directory.
    #[must_use]
    pub fn packages_dir(&self) -> &Path {
        &self.packages_dir
    }

    /// Unpacks a package tarball safely checking for path traversal, link escapes, and sizing limits.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if extraction fails, resource limits are exceeded,
    /// or directory creation fails.
    #[allow(clippy::too_many_lines)]
    pub fn safe_extract(
        &self,
        tarball_reader: impl std::io::Read,
    ) -> Result<tempfile::TempDir, Diagnostic> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        const MAX_FILES: usize = 10_000;
        const MAX_FILE_SIZE: u64 = 500 * 1024 * 1024; // 500 MB
        const MAX_TOTAL_SIZE: u64 = 1024 * 1024 * 1024; // 1 GB

        std::fs::create_dir_all(&self.temp_dir).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed to create temp directory base: {e}"),
            )
        })?;

        let tar = GzDecoder::new(tarball_reader);
        let mut archive = Archive::new(tar);

        let temp_dir = tempfile::Builder::new()
            .prefix("pkg_")
            .tempdir_in(&self.temp_dir)
            .map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    2,
                    format!("failed to create temporary staging directory: {e}"),
                )
            })?;

        let extraction_root = temp_dir.path();

        let mut file_count = 0;
        let mut total_bytes = 0;

        for entry_res in archive.entries().map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                3,
                format!("failed to read tarball entries: {e}"),
            )
        })? {
            let mut entry = entry_res.map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    4,
                    format!("failed to read tarball entry: {e}"),
                )
            })?;

            let path = entry
                .path()
                .map_err(|e| {
                    Diagnostic::new(
                        ErrorFamily::Store,
                        5,
                        format!("invalid entry path in tarball: {e}"),
                    )
                })?
                .to_path_buf();

            if path.is_absolute() {
                return Err(Diagnostic::new(
                    ErrorFamily::Store,
                    6,
                    format!(
                        "security violation: absolute path in tarball entry `{}`",
                        path.display()
                    ),
                ));
            }

            for component in path.components() {
                if let std::path::Component::ParentDir = component {
                    return Err(Diagnostic::new(
                        ErrorFamily::Store,
                        7,
                        format!(
                            "security violation: path traversal segment `..` in tarball entry `{}`",
                            path.display()
                        ),
                    ));
                }
            }

            let target_path = extraction_root.join(&path);
            let entry_type = entry.header().entry_type();
            if entry_type.is_symlink() || entry_type.is_hard_link() {
                return Err(Diagnostic::new(
                    ErrorFamily::Store,
                    8,
                    format!("security violation: symlinks/hardlinks are forbidden in package tarballs: `{}`", path.display()),
                ));
            }

            if !entry_type.is_dir() && !entry_type.is_file() {
                return Err(Diagnostic::new(
                    ErrorFamily::Store,
                    9,
                    format!(
                        "security violation: unsupported special file type in tarball entry `{}`",
                        path.display()
                    ),
                ));
            }

            if entry_type.is_dir() {
                std::fs::create_dir_all(&target_path).map_err(|e| {
                    Diagnostic::new(
                        ErrorFamily::Store,
                        10,
                        format!("failed to create directory `{}`: {e}", path.display()),
                    )
                })?;
            } else if entry_type.is_file() {
                file_count += 1;
                if file_count > MAX_FILES {
                    return Err(Diagnostic::new(
                        ErrorFamily::Store,
                        11,
                        "resource exhaustion: tarball contains too many files",
                    ));
                }

                let size = entry.header().size().map_err(|e| {
                    Diagnostic::new(
                        ErrorFamily::Store,
                        12,
                        format!("failed to read file size in tarball: {e}"),
                    )
                })?;

                if size > MAX_FILE_SIZE {
                    return Err(Diagnostic::new(
                        ErrorFamily::Store,
                        13,
                        format!(
                            "resource exhaustion: file `{}` exceeds maximum size limit",
                            path.display()
                        ),
                    ));
                }

                total_bytes += size;
                if total_bytes > MAX_TOTAL_SIZE {
                    return Err(Diagnostic::new(
                        ErrorFamily::Store,
                        14,
                        "resource exhaustion: total unpacked package size exceeds maximum limit",
                    ));
                }

                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        Diagnostic::new(
                            ErrorFamily::Store,
                            10,
                            format!("failed to create directory `{}`: {e}", parent.display()),
                        )
                    })?;
                }

                entry.unpack(&target_path).map_err(|e| {
                    Diagnostic::new(
                        ErrorFamily::Store,
                        15,
                        format!("failed to unpack file entry `{}`: {e}", path.display()),
                    )
                })?;
            }
        }

        Ok(temp_dir)
    }

    /// Computes the deterministic canonical directory hash (SHA-256) of a package.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if a file cannot be read or directory walk fails.
    pub fn compute_directory_hash(&self, dir: &Path) -> Result<String, Diagnostic> {
        use sha2::{Digest, Sha256};

        let mut files = Vec::new();
        Self::collect_files_recursive(dir, dir, &mut files)?;

        files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut hasher = Sha256::new();
        for (rel_path, abs_path) in files {
            hasher.update(rel_path.as_bytes());

            let metadata = std::fs::metadata(&abs_path).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    16,
                    format!("failed to read metadata for file `{rel_path}`: {e}"),
                )
            })?;
            hasher.update(metadata.len().to_be_bytes());

            let mut file = std::fs::File::open(&abs_path).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    17,
                    format!("failed to open file `{rel_path}`: {e}"),
                )
            })?;
            let mut file_hasher = Sha256::new();
            std::io::copy(&mut file, &mut file_hasher).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    18,
                    format!("failed to compute hash for file `{rel_path}`: {e}"),
                )
            })?;
            hasher.update(file_hasher.finalize());
        }

        let hash_bytes = hasher.finalize();
        let mut s = String::with_capacity(hash_bytes.len() * 2);
        for b in hash_bytes {
            use std::fmt::Write;
            let _ = write!(&mut s, "{b:02x}");
        }
        Ok(s)
    }

    fn collect_files_recursive(
        base_dir: &Path,
        current_dir: &Path,
        files: &mut Vec<(String, PathBuf)>,
    ) -> Result<(), Diagnostic> {
        for entry in std::fs::read_dir(current_dir).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                19,
                format!("failed to read directory `{}`: {e}", current_dir.display()),
            )
        })? {
            let entry = entry.map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    20,
                    format!("failed to read directory entry: {e}"),
                )
            })?;
            let path = entry.path();
            if path.is_dir() {
                Self::collect_files_recursive(base_dir, &path, files)?;
            } else if path.is_file() {
                let rel_path = path.strip_prefix(base_dir).unwrap();
                let rel_str = rel_path
                    .to_str()
                    .ok_or_else(|| {
                        Diagnostic::new(
                            ErrorFamily::Store,
                            21,
                            format!("invalid non-UTF8 filename: `{}`", rel_path.display()),
                        )
                    })?
                    .to_string();
                files.push((rel_str, path));
            }
        }
        Ok(())
    }

    /// Commits a staging directory atomically to its final content-addressed location.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if renaming files or setting permissions fails.
    pub fn commit(&self, temp_dir: &Path, content_hash: &str) -> Result<PathBuf, Diagnostic> {
        let prefix = &content_hash[0..2];
        let target_dir = self.packages_dir.join(prefix).join(content_hash);

        if target_dir.exists() {
            if let Ok(existing_hash) = self.compute_directory_hash(&target_dir) {
                if existing_hash == content_hash {
                    let _ = std::fs::remove_dir_all(temp_dir);
                    return Ok(target_dir);
                }
            }
            let _ = std::fs::remove_dir_all(&target_dir);
        }

        if let Some(parent) = target_dir.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    22,
                    format!("failed to create destination parent directory: {e}"),
                )
            })?;
        }

        std::fs::rename(temp_dir, &target_dir).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                23,
                format!("failed to atomically rename directory to store destination: {e}"),
            )
        })?;

        Self::make_readonly_recursive(&target_dir)?;

        Ok(target_dir)
    }

    fn make_readonly_recursive(path: &Path) -> Result<(), Diagnostic> {
        let metadata = std::fs::metadata(path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                24,
                format!("failed to read metadata for `{}`: {e}", path.display()),
            )
        })?;

        let mut permissions = metadata.permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if path.is_dir() {
                permissions.set_mode(0o555);
            } else {
                permissions.set_mode(0o444);
            }
        }
        #[cfg(not(unix))]
        {
            permissions.set_readonly(true);
        }

        std::fs::set_permissions(path, permissions).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                25,
                format!(
                    "failed to set read-only permissions for `{}`: {e}",
                    path.display()
                ),
            )
        })?;

        if path.is_dir() {
            for entry in std::fs::read_dir(path).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    19,
                    format!("failed to read directory `{}`: {e}", path.display()),
                )
            })? {
                let entry = entry.map_err(|e| {
                    Diagnostic::new(
                        ErrorFamily::Store,
                        20,
                        format!("failed to read directory entry: {e}"),
                    )
                })?;
                Self::make_readonly_recursive(&entry.path())?;
            }
        }

        Ok(())
    }

    /// Verifies store integrity recomputing directory hashes.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if directories cannot be read.
    pub fn validate_integrity(&self) -> Result<Vec<(String, Diagnostic)>, Diagnostic> {
        let mut corruptions = Vec::new();
        if !self.packages_dir.exists() {
            return Ok(corruptions);
        }

        for prefix_entry in std::fs::read_dir(&self.packages_dir).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                19,
                format!("failed to read packages directory: {e}"),
            )
        })? {
            let prefix_entry = prefix_entry.map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    20,
                    format!("failed to read directory entry: {e}"),
                )
            })?;
            let prefix_path = prefix_entry.path();
            if prefix_path.is_dir() {
                for pkg_entry in std::fs::read_dir(&prefix_path).map_err(|e| {
                    Diagnostic::new(
                        ErrorFamily::Store,
                        19,
                        format!(
                            "failed to read package hash directory `{}`: {e}",
                            prefix_path.display()
                        ),
                    )
                })? {
                    let pkg_entry = pkg_entry.map_err(|e| {
                        Diagnostic::new(
                            ErrorFamily::Store,
                            20,
                            format!("failed to read directory entry: {e}"),
                        )
                    })?;
                    let pkg_path = pkg_entry.path();
                    if pkg_path.is_dir() {
                        let hash_key = pkg_path
                            .file_name()
                            .map_or_else(String::new, |f| f.to_string_lossy().into_owned());
                        match self.compute_directory_hash(&pkg_path) {
                            Ok(computed_hash) => {
                                if computed_hash != hash_key {
                                    corruptions.push((
                                        hash_key.clone(),
                                        Diagnostic::new(
                                            ErrorFamily::Store,
                                            26,
                                            format!("integrity corruption detected: package directory `{}` recomputed hash `{}` mismatches target key", pkg_path.display(), computed_hash),
                                        ),
                                    ));
                                }
                            }
                            Err(diag) => {
                                corruptions.push((hash_key.clone(), diag));
                            }
                        }
                    }
                }
            }
        }

        Ok(corruptions)
    }

    /// Computes Content-Addressed Store statistics.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if filesystem read operations fail.
    pub fn stats(&self) -> Result<StoreStats, Diagnostic> {
        let mut unique_packages = 0;
        let mut physical_bytes = 0;

        if self.packages_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&self.packages_dir) {
                for prefix_entry in entries.flatten() {
                    if prefix_entry.path().is_dir() {
                        if let Ok(pkg_entries) = std::fs::read_dir(prefix_entry.path()) {
                            for pkg in pkg_entries.flatten() {
                                if pkg.path().is_dir() {
                                    unique_packages += 1;
                                    physical_bytes += Self::measure_dir_size(&pkg.path())?;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(StoreStats {
            unique_packages,
            physical_bytes,
            logical_bytes: physical_bytes,
            reuse_ratio: 1.0,
        })
    }

    fn measure_dir_size(path: &Path) -> Result<u64, Diagnostic> {
        let mut total = 0;
        for entry in std::fs::read_dir(path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                19,
                format!("failed to read directory `{}`: {e}", path.display()),
            )
        })? {
            let entry = entry.map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    20,
                    format!("failed to read directory entry: {e}"),
                )
            })?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                total += Self::measure_dir_size(&entry_path)?;
            } else if entry_path.is_file() {
                let metadata = std::fs::metadata(&entry_path).map_err(|e| {
                    Diagnostic::new(
                        ErrorFamily::Store,
                        24,
                        format!(
                            "failed to read metadata for `{}`: {e}",
                            entry_path.display()
                        ),
                    )
                })?;
                total += metadata.len();
            }
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    fn create_mock_tarball(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut tar_bytes = Vec::new();
        {
            let enc = GzEncoder::new(&mut tar_bytes, Compression::default());
            let mut builder = Builder::new(enc);
            for (path, content) in files {
                let mut header = tar::Header::new_gnu();
                header.set_size(content.len() as u64);
                let bytes = path.as_bytes();
                let len = bytes.len().min(99);
                header.as_mut_bytes()[0..len].copy_from_slice(&bytes[0..len]);
                header.set_cksum();
                builder.append(&header, *content).unwrap();
            }
            builder.finish().unwrap();
        }
        tar_bytes
    }

    #[test]
    fn test_safe_extract_valid() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ContentAddressedStore::new(tmp.path());

        let tarball = create_mock_tarball(&[
            ("package/package.json", b"{\"name\": \"foo\"}"),
            ("package/index.js", b"console.log('hello')"),
        ]);

        let staging = store.safe_extract(&tarball[..]).unwrap();
        let staging_path = staging.path();

        assert!(staging_path.join("package/package.json").exists());
        assert!(staging_path.join("package/index.js").exists());
    }

    #[test]
    fn test_safe_extract_traversal_absolute() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ContentAddressedStore::new(tmp.path());

        let tarball = create_mock_tarball(&[("/absolute/path.js", b"content")]);
        assert!(store.safe_extract(&tarball[..]).is_err());
    }

    #[test]
    fn test_safe_extract_traversal_relative() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ContentAddressedStore::new(tmp.path());

        let tarball = create_mock_tarball(&[("package/../../escape.js", b"content")]);
        assert!(store.safe_extract(&tarball[..]).is_err());
    }

    #[test]
    fn test_safe_extract_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ContentAddressedStore::new(tmp.path());

        let mut tar_bytes = Vec::new();
        {
            let enc = GzEncoder::new(&mut tar_bytes, Compression::default());
            let mut builder = Builder::new(enc);
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_path("package/symlink").unwrap();
            header.set_link_name("../outside").unwrap();
            builder.append(&header, &[][..]).unwrap();
            builder.finish().unwrap();
        }

        assert!(store.safe_extract(&tar_bytes[..]).is_err());
    }

    #[test]
    fn test_directory_hashing_and_commit() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ContentAddressedStore::new(tmp.path());

        let tarball1 = create_mock_tarball(&[
            ("package/package.json", b"{\"name\": \"foo\"}"),
            ("package/index.js", b"console.log('hello')"),
        ]);

        let staging = store.safe_extract(&tarball1[..]).unwrap();
        let hash = store.compute_directory_hash(staging.path()).unwrap();
        assert!(!hash.is_empty());

        let committed = store.commit(staging.path(), &hash).unwrap();
        assert!(committed.exists());

        let file_meta = std::fs::metadata(committed.join("package/index.js")).unwrap();
        assert!(file_meta.permissions().readonly());

        let stats = store.stats().unwrap();
        assert_eq!(stats.unique_packages, 1);
    }
}
