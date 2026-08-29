//! `CorexPM` integrity, cryptographic provenance, and security enforcement status orchestration.

pub mod enforcement;
pub mod provenance;

pub use enforcement::{
    CapabilityCategory, CapabilityEnforcementEvaluator, CapabilityStatus, EnforcementLevel,
    PlatformEnforcementReport,
};
pub use provenance::{ArtifactChecksum, BuildProvenance, ProvenanceVerifier};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn create_temp_dir() -> PathBuf {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut dir = std::env::temp_dir();
        dir.push(format!("corex_sec_test_{pid}_{nanos}_{count}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_provenance_generation_and_verification() {
        let temp = create_temp_dir();
        let file_path = temp.join("binary.node");
        let content = b"binary-build-artifact-payload";
        fs::write(&file_path, content).unwrap();

        let verifier = ProvenanceVerifier::new();
        let provenance = verifier.generate_provenance(
            "@corex/native",
            "1.0.0",
            &[(PathBuf::from("binary.node"), content.to_vec())],
        );

        assert_eq!(provenance.artifacts.len(), 1);
        assert!(!provenance.signature_sha256.is_empty());

        let is_valid = verifier.verify_provenance(&temp, &provenance).unwrap();
        assert!(is_valid);

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn test_provenance_tamper_detection() {
        let temp = create_temp_dir();
        let file_path = temp.join("binary.node");
        let content = b"original-artifact";
        fs::write(&file_path, content).unwrap();

        let verifier = ProvenanceVerifier::new();
        let provenance = verifier.generate_provenance(
            "@corex/native",
            "1.0.0",
            &[(PathBuf::from("binary.node"), content.to_vec())],
        );

        // Mutate file payload
        fs::write(&file_path, b"tampered-artifact").unwrap();

        let res = verifier.verify_provenance(&temp, &provenance);
        assert!(res.is_err());
        let diag = res.unwrap_err();
        assert_eq!(diag.code(), "CXSEC0001");
        assert!(diag.message().contains("checksum mismatch"));

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn test_platform_capability_enforcement_evaluation() {
        let evaluator = CapabilityEnforcementEvaluator::new();
        let report = evaluator.evaluate_current_platform();

        assert!(!report.os.is_empty());
        assert_eq!(report.capabilities.len(), 6);
        assert!(report
            .capabilities
            .iter()
            .any(|c| c.level == EnforcementLevel::Enforced));
    }
}
