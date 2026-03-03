use crate::sign::SignatureBundle;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Verification result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub signer_identity: Option<String>,
    pub transparency_log_verified: bool,
    pub details: Vec<String>,
}

/// Signature verifier.
pub struct Verifier;

impl Verifier {
    /// Verify a file against a signature bundle.
    pub fn verify_file(path: &Path, bundle: &SignatureBundle) -> Result<VerificationResult> {
        let content = std::fs::read(path)?;
        Self::verify_bytes(&content, bundle)
    }

    /// Verify raw bytes against a signature bundle.
    pub fn verify_bytes(data: &[u8], bundle: &SignatureBundle) -> Result<VerificationResult> {
        let _digest = Sha256::digest(data);
        let mut details = Vec::new();

        // Check if bundle has required fields
        if bundle.signature.is_empty() {
            return Ok(VerificationResult {
                valid: false,
                signer_identity: None,
                transparency_log_verified: false,
                details: vec!["Missing signature in bundle".into()],
            });
        }

        if bundle.certificate.is_empty() {
            details.push("No certificate present — cannot verify signer identity".into());
        }

        let has_rekor = bundle.rekor_log_id.is_some();
        if has_rekor {
            details.push("Transparency log entry present".into());
        }

        // Try cosign verify-blob as fallback
        match Self::cosign_verify(data, bundle) {
            Ok(true) => {
                details.push("Verified via cosign".into());
                Ok(VerificationResult {
                    valid: true,
                    signer_identity: None,
                    transparency_log_verified: has_rekor,
                    details,
                })
            }
            Ok(false) => {
                details.push("Cosign verification failed".into());
                Ok(VerificationResult {
                    valid: false,
                    signer_identity: None,
                    transparency_log_verified: false,
                    details,
                })
            }
            Err(e) => {
                details.push(format!("Cosign not available: {e}"));
                // Without cosign, we can't verify — report as unverifiable
                Ok(VerificationResult {
                    valid: false,
                    signer_identity: None,
                    transparency_log_verified: false,
                    details,
                })
            }
        }
    }

    fn cosign_verify(data: &[u8], bundle: &SignatureBundle) -> Result<bool> {
        use std::process::Command;

        let temp_dir = tempfile::tempdir()?;
        let data_path = temp_dir.path().join("data");
        let bundle_path = temp_dir.path().join("bundle.json");

        std::fs::write(&data_path, data)?;
        std::fs::write(&bundle_path, serde_json::to_string(bundle)?)?;

        let output = Command::new("cosign")
            .args([
                "verify-blob",
                "--bundle",
                bundle_path.to_str().unwrap(),
                data_path.to_str().unwrap(),
            ])
            .output()?;

        Ok(output.status.success())
    }
}
