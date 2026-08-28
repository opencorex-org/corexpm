//! Streaming package fetcher, integrity verifier, and safe archive extractor.

#![forbid(unsafe_code)]

use corex_errors::{Diagnostic, ErrorFamily};
use flate2::read::GzDecoder;
use sha1::{Digest as Sha1DigestTrait, Sha1};
use sha2::{Sha256, Sha512};
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use tar::Archive;

/// Resource allocation limits enforced during package extraction to prevent resource exhaustion attacks.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExtractionLimits {
    /// Maximum allowed file count in an archive.
    pub max_file_count: usize,
    /// Maximum allowed individual file size in bytes.
    pub max_single_file_size: u64,
    /// Maximum allowed total extracted size in bytes.
    pub max_total_size: u64,
    /// Maximum allowed relative path depth.
    pub max_path_depth: usize,
}

impl Default for ExtractionLimits {
    fn default() -> Self {
        Self {
            max_file_count: 50_000,
            max_single_file_size: 500 * 1024 * 1024, // 500 MB
            max_total_size: 2 * 1024 * 1024 * 1024,  // 2 GB
            max_path_depth: 32,
        }
    }
}

/// Summary details returned after successfully extracting an archive.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExtractionSummary {
    /// Total number of extracted files.
    pub file_count: usize,
    /// Total byte size of all extracted files.
    pub total_bytes: u64,
    /// List of relative file paths extracted within the root directory.
    pub files: Vec<PathBuf>,
}

/// Simple RFC 4648 standard base64 encoder.
#[must_use]
pub fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let triple = (u32::from(b0) << 16) | (u32::from(b1) << 8) | u32::from(b2);

        result.push(char::from(ALPHABET[((triple >> 18) & 0x3F) as usize]));
        result.push(char::from(ALPHABET[((triple >> 12) & 0x3F) as usize]));

        if i + 1 < data.len() {
            result.push(char::from(ALPHABET[((triple >> 6) & 0x3F) as usize]));
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(char::from(ALPHABET[(triple & 0x3F) as usize]));
        } else {
            result.push('=');
        }

        i += 3;
    }
    result
}

/// Simple RFC 4648 standard base64 decoder.
///
/// # Errors
/// Returns `Diagnostic` if an invalid base64 character is encountered.
#[allow(clippy::cast_possible_truncation)]
pub fn base64_decode(input: &str) -> Result<Vec<u8>, Diagnostic> {
    let clean = input.trim_end_matches('=');
    let mut buf = 0u32;
    let mut bits = 0u32;
    let mut out = Vec::new();

    for c in clean.chars() {
        let val = match c {
            'A'..='Z' => u32::from(c as u8 - b'A'),
            'a'..='z' => u32::from(c as u8 - b'a' + 26),
            '0'..='9' => u32::from(c as u8 - b'0' + 52),
            '+' => 62,
            '/' => 63,
            _ => {
                return Err(Diagnostic::new(
                    ErrorFamily::Security,
                    1,
                    format!("invalid base64 character: '{c}'"),
                ))
            }
        };
        buf = (buf << 6) | val;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Ok(out)
}

/// Verifies that data matches the expected integrity algorithm and checksum string.
///
/// Supported formats:
/// - `sha512-<base64>`
/// - `sha256-<base64>`
/// - `sha1-<base64>`
/// - 40-character hex string (SHA-1)
///
/// # Errors
/// Returns `Diagnostic` with code `CXSEC0001` if integrity verification fails.
pub fn verify_integrity(data: &[u8], expected: &str) -> Result<(), Diagnostic> {
    let expected_trimmed = expected.trim();
    if let Some(rest) = expected_trimmed.strip_prefix("sha512-") {
        let expected_bytes = base64_decode(rest)?;
        let mut hasher = Sha512::new();
        hasher.update(data);
        let actual = hasher.finalize();
        if actual.as_slice() != expected_bytes.as_slice() {
            let actual_b64 = base64_encode(actual.as_slice());
            return Err(Diagnostic::new(
                ErrorFamily::Security,
                1,
                format!(
                    "SHA-512 integrity mismatch: expected `sha512-{rest}`, got `sha512-{actual_b64}`"
                ),
            )
            .with_help("the downloaded package tarball may be corrupted or tampered with"));
        }
        Ok(())
    } else if let Some(rest) = expected_trimmed.strip_prefix("sha256-") {
        let expected_bytes = base64_decode(rest)?;
        let mut hasher = Sha256::new();
        hasher.update(data);
        let actual = hasher.finalize();
        if actual.as_slice() != expected_bytes.as_slice() {
            let actual_b64 = base64_encode(actual.as_slice());
            return Err(Diagnostic::new(
                ErrorFamily::Security,
                1,
                format!(
                    "SHA-256 integrity mismatch: expected `sha256-{rest}`, got `sha256-{actual_b64}`"
                ),
            )
            .with_help("the downloaded package tarball may be corrupted or tampered with"));
        }
        Ok(())
    } else if let Some(rest) = expected_trimmed.strip_prefix("sha1-") {
        let expected_bytes = base64_decode(rest)?;
        let mut hasher = Sha1::new();
        hasher.update(data);
        let actual = hasher.finalize();
        if actual.as_slice() != expected_bytes.as_slice() {
            let actual_b64 = base64_encode(actual.as_slice());
            return Err(Diagnostic::new(
                ErrorFamily::Security,
                1,
                format!(
                    "SHA-1 integrity mismatch: expected `sha1-{rest}`, got `sha1-{actual_b64}`"
                ),
            )
            .with_help("the downloaded package tarball may be corrupted or tampered with"));
        }
        Ok(())
    } else if expected_trimmed.len() == 40
        && expected_trimmed.chars().all(|c| c.is_ascii_hexdigit())
    {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let actual = hasher.finalize();
        let actual_hex = hex::encode(actual);
        if actual_hex.to_lowercase() != expected_trimmed.to_lowercase() {
            return Err(Diagnostic::new(
                ErrorFamily::Security,
                1,
                format!("SHA-1 shasum mismatch: expected `{expected_trimmed}`, got `{actual_hex}`"),
            )
            .with_help("the downloaded package tarball may be corrupted or tampered with"));
        }
        Ok(())
    } else {
        Err(Diagnostic::new(
            ErrorFamily::Security,
            1,
            format!("unsupported or malformed integrity format: `{expected_trimmed}`"),
        )
        .with_help(
            "supported formats: sha512-<base64>, sha256-<base64>, sha1-<base64>, or 40-char hex",
        ))
    }
}

/// Safely extracts a `.tar.gz` stream into `destination_dir`.
///
/// Ensures:
/// - No path traversal (`..` or absolute paths).
/// - No symlink/hardlink escapes outside `destination_dir`.
/// - Leading top-level directory (e.g. `package/`) is stripped.
/// - Limits on file count, single file size, total size, and path depth are enforced.
///
/// # Errors
/// Returns `Diagnostic` with code `CXSEC0002` on path traversal or security limit violations,
/// or `CXSTORE0001` on IO errors.
#[allow(clippy::too_many_lines)]
pub fn extract_tarball_stream<R: Read>(
    reader: R,
    destination_dir: &Path,
    limits: Option<ExtractionLimits>,
) -> Result<ExtractionSummary, Diagnostic> {
    let limits = limits.unwrap_or_default();
    let gz = GzDecoder::new(reader);
    let mut archive = Archive::new(gz);

    fs::create_dir_all(destination_dir).map_err(|e| {
        Diagnostic::new(
            ErrorFamily::Store,
            1,
            format!(
                "failed to create extraction directory `{}`: {e}",
                destination_dir.display()
            ),
        )
    })?;

    let canonical_dest = destination_dir.canonicalize().map_err(|e| {
        Diagnostic::new(
            ErrorFamily::Store,
            1,
            format!("failed to canonicalize target extraction dir: {e}"),
        )
    })?;

    let entries = archive.entries().map_err(|e| {
        Diagnostic::new(
            ErrorFamily::Security,
            2,
            format!("invalid or corrupted tar archive header: {e}"),
        )
    })?;

    let mut file_count = 0usize;
    let mut total_bytes = 0u64;
    let mut files = Vec::new();

    for entry_result in entries {
        let mut entry = entry_result.map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Security,
                2,
                format!("corrupted entry in tar archive: {e}"),
            )
        })?;

        let entry_path = entry.path().map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Security,
                2,
                format!("malformed entry path in tar archive: {e}"),
            )
        })?;

        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink()
            || entry_type.is_hard_link()
            || entry_type.is_character_special()
            || entry_type.is_block_special()
            || entry_type.is_fifo()
        {
            return Err(Diagnostic::new(
                ErrorFamily::Security,
                2,
                format!(
                    "unsafe entry type in tar archive: `{}`",
                    entry_path.display()
                ),
            )
            .with_help(
                "CorexPM rejects special files, hardlinks, and symlinks during package extraction",
            ));
        }

        // Strip leading top-level directory (like "package/" in npm tarballs)
        let mut components: Vec<_> = entry_path.components().collect();
        if components.is_empty() {
            continue;
        }

        // Check path traversal in raw components
        for comp in &components {
            match comp {
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(Diagnostic::new(
                        ErrorFamily::Security,
                        2,
                        format!(
                            "path traversal detected in archive entry: `{}`",
                            entry_path.display()
                        ),
                    )
                    .with_help("archive entry contains `..` or absolute root path"));
                }
                _ => {}
            }
        }

        if components.len() > 1 {
            // Strip top level component (e.g. `package/`)
            components.remove(0);
        } else if entry.header().entry_type().is_dir() {
            // Top-level directory itself
            continue;
        }

        if components.is_empty() {
            continue;
        }

        if components.len() > limits.max_path_depth {
            return Err(Diagnostic::new(
                ErrorFamily::Security,
                2,
                format!(
                    "archive entry path depth {} exceeds maximum limit {}",
                    components.len(),
                    limits.max_path_depth
                ),
            ));
        }

        let rel_path: PathBuf = components.iter().map(|c| c.as_os_str()).collect();
        let target_path = canonical_dest.join(&rel_path);

        // Security check: ensure target_path stays inside canonical_dest
        if !target_path.starts_with(&canonical_dest) {
            return Err(Diagnostic::new(
                ErrorFamily::Security,
                2,
                format!(
                    "archive path escape detected: `{}` target `{}`",
                    entry_path.display(),
                    target_path.display()
                ),
            ));
        }

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&target_path).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Store,
                    1,
                    format!(
                        "failed to create directory `{}`: {e}",
                        target_path.display()
                    ),
                )
            })?;
            continue;
        }

        file_count += 1;
        if file_count > limits.max_file_count {
            return Err(Diagnostic::new(
                ErrorFamily::Security,
                2,
                format!(
                    "archive exceeds maximum file count limit of {}",
                    limits.max_file_count
                ),
            ));
        }

        let size = entry.size();
        if size > limits.max_single_file_size {
            return Err(Diagnostic::new(
                ErrorFamily::Security,
                2,
                format!(
                    "file `{}` size {size} bytes exceeds max single file limit {}",
                    rel_path.display(),
                    limits.max_single_file_size
                ),
            ));
        }

        total_bytes += size;
        if total_bytes > limits.max_total_size {
            return Err(Diagnostic::new(
                ErrorFamily::Security,
                2,
                format!(
                    "archive total extracted size exceeds limit of {} bytes",
                    limits.max_total_size
                ),
            ));
        }

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    Diagnostic::new(
                        ErrorFamily::Store,
                        1,
                        format!("failed to create parent dir `{}`: {e}", parent.display()),
                    )
                })?;
            }
        }

        let mut out_file = fs::File::create(&target_path).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!(
                    "failed to create output file `{}`: {e}",
                    target_path.display()
                ),
            )
        })?;

        io::copy(&mut entry, &mut out_file).map_err(|e| {
            Diagnostic::new(
                ErrorFamily::Store,
                1,
                format!("failed writing entry `{}`: {e}", rel_path.display()),
            )
        })?;

        files.push(rel_path);
    }

    Ok(ExtractionSummary {
        file_count,
        total_bytes,
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;

    #[test]
    fn test_base64_encode_decode() {
        let input = b"Hello, CorexPM!";
        let encoded = base64_encode(input);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_verify_integrity_sha512() {
        let data = b"package contents";
        let mut hasher = Sha512::new();
        hasher.update(data);
        let digest = hasher.finalize();
        let b64 = base64_encode(digest.as_slice());
        let expected = format!("sha512-{b64}");

        assert!(verify_integrity(data, &expected).is_ok());
        assert!(verify_integrity(b"wrong contents", &expected).is_err());
    }

    #[test]
    fn test_verify_integrity_sha1_hex() {
        let data = b"test payload";
        let mut hasher = Sha1::new();
        hasher.update(data);
        let hex_str = hex::encode(hasher.finalize());

        assert!(verify_integrity(data, &hex_str).is_ok());
        assert!(verify_integrity(b"tampered", &hex_str).is_err());
    }

    #[test]
    fn test_safe_tarball_extraction() {
        let temp_dir = std::env::temp_dir().join("corex_test_extract_safe");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut tar_builder = tar::Builder::new(Vec::new());

        let mut header = tar::Header::new_gnu();
        header.set_path("package/index.js").unwrap();
        header.set_size(13);
        header.set_cksum();
        tar_builder.append(&header, &b"console.log()"[..]).unwrap();

        let uncompressed = tar_builder.into_inner().unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        std::io::Write::write_all(&mut encoder, &uncompressed).unwrap();
        let compressed = encoder.finish().unwrap();

        let summary = extract_tarball_stream(&compressed[..], &temp_dir, None).unwrap();
        assert_eq!(summary.file_count, 1);
        assert!(temp_dir.join("index.js").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_safe_tarball_rejects_path_traversal() {
        let temp_dir = std::env::temp_dir().join("corex_test_extract_traversal");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut tar_builder = tar::Builder::new(Vec::new());

        let mut header = tar::Header::new_gnu();
        let name_bytes = b"package/../evil.txt";
        if let Some(gnu) = header.as_gnu_mut() {
            gnu.name[..name_bytes.len()].copy_from_slice(name_bytes);
        }
        header.set_size(4);
        header.set_cksum();
        tar_builder.append(&header, &b"evil"[..]).unwrap();

        let uncompressed = tar_builder.into_inner().unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        std::io::Write::write_all(&mut encoder, &uncompressed).unwrap();
        let compressed = encoder.finish().unwrap();

        let err = extract_tarball_stream(&compressed[..], &temp_dir, None).unwrap_err();
        assert!(err.code().starts_with("CXSEC"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
