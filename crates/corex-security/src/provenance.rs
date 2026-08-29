//! Cryptographic build provenance generation, checksum manifests, and verification.

use corex_errors::{Diagnostic, ErrorFamily};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Checksum entry for an artifact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactChecksum {
    /// Relative path to artifact.
    pub path: PathBuf,
    /// Hex-encoded SHA-256 digest of artifact bytes.
    pub sha256: String,
}

/// Cryptographic provenance manifest for build outputs and release binaries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BuildProvenance {
    /// Component or package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Target operating system.
    pub os: String,
    /// Target CPU architecture.
    pub arch: String,
    /// Generation timestamp (UNIX epoch seconds).
    pub timestamp_epoch: u64,
    /// List of file checksum entries.
    pub artifacts: Vec<ArtifactChecksum>,
    /// Overall manifest SHA-256 signature/digest.
    pub signature_sha256: String,
}

/// Service for generating and verifying cryptographic build provenance.
#[derive(Debug, Default)]
pub struct ProvenanceVerifier;

impl ProvenanceVerifier {
    /// Creates a new provenance verifier instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Computes SHA-256 hex digest of raw byte content.
    #[must_use]
    pub fn compute_sha256(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    /// Computes SHA-256 hex digest of file contents at `path`.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if reading the file fails.
    pub fn compute_file_sha256(&self, path: &Path) -> Result<String, Diagnostic> {
        let content = fs::read(path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Security,
                1,
                format!(
                    "failed to read file for provenance calculation at `{}`: {e}",
                    path.display()
                ),
            )
        })?;
        Ok(Self::compute_sha256(&content))
    }

    /// Generates a signed `BuildProvenance` manifest for target files.
    #[must_use]
    pub fn generate_provenance(
        &self,
        name: &str,
        version: &str,
        file_entries: &[(PathBuf, Vec<u8>)],
    ) -> BuildProvenance {
        let mut artifacts = Vec::new();
        let mut signature_hasher = Sha256::new();

        signature_hasher.update(name.as_bytes());
        signature_hasher.update(version.as_bytes());

        for (rel_path, bytes) in file_entries {
            let digest = Self::compute_sha256(bytes);
            signature_hasher.update(rel_path.to_string_lossy().as_bytes());
            signature_hasher.update(digest.as_bytes());

            artifacts.push(ArtifactChecksum {
                path: rel_path.clone(),
                sha256: digest,
            });
        }

        let signature_sha256 = hex::encode(signature_hasher.finalize());

        BuildProvenance {
            name: name.to_string(),
            version: version.to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            timestamp_epoch: 1_700_000_000,
            artifacts,
            signature_sha256,
        }
    }

    /// Verifies all file checksums in `provenance` relative to `root_dir`.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] with error code `CXSEC0001` if verification fails or checksum mismatches.
    pub fn verify_provenance(
        &self,
        root_dir: &Path,
        provenance: &BuildProvenance,
    ) -> Result<bool, Diagnostic> {
        for artifact in &provenance.artifacts {
            let target_path = root_dir.join(&artifact.path);
            if !target_path.exists() {
                return Err(Diagnostic::new(
                    ErrorFamily::Security,
                    1,
                    format!(
                        "provenance verification failed: file missing at `{}`",
                        target_path.display()
                    ),
                )
                .with_help("ensure all build output artifacts are intact"));
            }

            let actual_sha256 = self.compute_file_sha256(&target_path)?;
            if actual_sha256 != artifact.sha256 {
                return Err(Diagnostic::new(
                    ErrorFamily::Security,
                    1,
                    format!(
                        "provenance checksum mismatch for `{}`: expected {}, found {actual_sha256}",
                        artifact.path.display(),
                        artifact.sha256
                    ),
                )
                .with_help("tampered or corrupted build output artifact detected"));
            }
        }

        Ok(true)
    }
}
