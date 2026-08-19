//! Tarball fetching, verification, and extraction orchestration.

use corex_errors::{Diagnostic, ErrorFamily};
use corex_store::ContentAddressedStore;
use std::path::PathBuf;

/// Fetcher and expected-integrity verifier interface.
pub trait TarballFetcher: Send + Sync {
    /// Fetches package tarball and commits it to the Content-Addressed Store.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if fetching, verification, or extraction fails.
    fn fetch_and_verify(
        &self,
        package_name: &str,
        version: &str,
        tarball_url: &str,
        expected_integrity: &str,
    ) -> Result<PathBuf, Diagnostic>;
}

/// A mock tarball fetcher which reads pre-loaded tarball fixtures.
#[derive(Debug)]
pub struct MockTarballFetcher {
    fixtures_dir: PathBuf,
    store: std::sync::Arc<ContentAddressedStore>,
}

impl MockTarballFetcher {
    /// Creates a `MockTarballFetcher` using the specified fixtures directory and store reference.
    #[must_use]
    pub fn new(
        fixtures_dir: impl Into<PathBuf>,
        store: std::sync::Arc<ContentAddressedStore>,
    ) -> Self {
        Self {
            fixtures_dir: fixtures_dir.into(),
            store,
        }
    }
}

impl TarballFetcher for MockTarballFetcher {
    fn fetch_and_verify(
        &self,
        package_name: &str,
        version: &str,
        _tarball_url: &str,
        expected_integrity: &str,
    ) -> Result<PathBuf, Diagnostic> {
        let safe_name = package_name.replace('/', "__").replace('@', "_");
        let path = self.fixtures_dir.join(format!("{safe_name}-{version}.tgz"));
        if !path.exists() {
            return Err(Diagnostic::new(
                ErrorFamily::Store,
                29,
                format!("mock tarball fixture not found for `{package_name}@{version}`"),
            )
            .with_help(format!("expected tarball file at `{}`", path.display())));
        }

        let content = std::fs::read(&path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                30,
                format!("failed to read mock tarball for `{package_name}`: {e}"),
            )
        })?;

        Self::verify_integrity_bytes(&content, expected_integrity)?;

        let temp_dir = self.store.safe_extract(&content[..])?;
        let dir_hash = self.store.compute_directory_hash(temp_dir.path())?;
        self.store.commit(temp_dir.path(), &dir_hash)
    }
}

impl MockTarballFetcher {
    fn verify_integrity_bytes(content: &[u8], expected_integrity: &str) -> Result<(), Diagnostic> {
        use sha2::{Digest, Sha256, Sha512};
        if let Some(expected_hex_or_base64) = expected_integrity.strip_prefix("sha512-") {
            let expected_bytes = Self::decode_hash_string(expected_hex_or_base64)?;
            let mut hasher = Sha512::new();
            hasher.update(content);
            let computed = hasher.finalize();
            if computed[..] != expected_bytes[..] {
                return Err(Diagnostic::new(
                    ErrorFamily::Store,
                    31,
                    format!("tarball integrity mismatch! expected `{expected_integrity}` but computed different sha512"),
                ));
            }
        } else if let Some(expected_hex_or_base64) = expected_integrity.strip_prefix("sha256-") {
            let expected_bytes = Self::decode_hash_string(expected_hex_or_base64)?;
            let mut hasher = Sha256::new();
            hasher.update(content);
            let computed = hasher.finalize();
            if computed[..] != expected_bytes[..] {
                return Err(Diagnostic::new(
                    ErrorFamily::Store,
                    31,
                    format!("tarball integrity mismatch! expected `{expected_integrity}` but computed different sha256"),
                ));
            }
        }
        Ok(())
    }

    fn decode_hash_string(hash_str: &str) -> Result<Vec<u8>, Diagnostic> {
        Self::decode_base64(hash_str).or_else(|_| Self::decode_hex(hash_str))
    }

    fn decode_hex(hex_str: &str) -> Result<Vec<u8>, Diagnostic> {
        let mut bytes = Vec::new();
        let mut chars = hex_str.chars().peekable();
        while let Some(c1) = chars.next() {
            let c2 = chars
                .next()
                .ok_or_else(|| Diagnostic::new(ErrorFamily::Store, 32, "odd length hex string"))?;
            let val = u8::from_str_radix(&format!("{c1}{c2}"), 16).map_err(|e| {
                Diagnostic::new(ErrorFamily::Store, 33, format!("invalid hex char: {e}"))
            })?;
            bytes.push(val);
        }
        Ok(bytes)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn decode_base64(b64_str: &str) -> Result<Vec<u8>, Diagnostic> {
        const B64_CHARS: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let clean: Vec<u8> = b64_str
            .bytes()
            .filter(|&b| b != b'=' && b != b'\n' && b != b'\r' && b != b' ')
            .collect();
        let mut bytes = Vec::new();
        let mut buffer = 0u32;
        let mut bits = 0;
        for byte in clean {
            let val = B64_CHARS.iter().position(|&x| x == byte).ok_or_else(|| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    34,
                    format!("invalid base64 char: {}", byte as char),
                )
            })? as u32;
            buffer = (buffer << 6) | val;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                bytes.push((buffer >> bits) as u8);
            }
        }
        Ok(bytes)
    }
}
