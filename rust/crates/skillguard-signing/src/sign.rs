use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Trait abstraction for signing — allows fallback to cosign CLI.
pub trait SignerBackend: Send + Sync {
    fn sign(&self, digest: &[u8]) -> Result<SignatureBundle>;
}

/// A signature bundle (Sigstore format).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignatureBundle {
    /// Base64-encoded signature.
    pub signature: String,
    /// PEM-encoded certificate from Fulcio.
    pub certificate: String,
    /// Rekor transparency log entry.
    pub rekor_log_id: Option<String>,
}

/// Sigstore keyless signer.
pub struct Signer {
    backend: Box<dyn SignerBackend>,
}

impl Signer {
    /// Create a signer using the Sigstore Rust crate.
    pub fn sigstore() -> Self {
        Self {
            backend: Box::new(SigstoreBackend),
        }
    }

    /// Create a signer that falls back to the cosign CLI.
    pub fn cosign_cli() -> Self {
        Self {
            backend: Box::new(CosignCliBackend),
        }
    }

    /// Sign a file and return the signature bundle.
    pub fn sign_file(&self, path: &Path) -> Result<SignatureBundle> {
        let content = std::fs::read(path)?;
        let digest = Sha256::digest(&content);
        self.backend.sign(&digest)
    }

    /// Sign raw bytes.
    pub fn sign_bytes(&self, data: &[u8]) -> Result<SignatureBundle> {
        let digest = Sha256::digest(data);
        self.backend.sign(&digest)
    }
}

/// Sigstore Rust crate backend.
struct SigstoreBackend;

impl SignerBackend for SigstoreBackend {
    fn sign(&self, digest: &[u8]) -> Result<SignatureBundle> {
        // TODO: Implement with sigstore crate when stable enough.
        // For now, return an error indicating keyless signing requires OIDC.
        let _ = digest;
        anyhow::bail!(
            "Sigstore keyless signing not yet implemented. \
             Use --backend cosign or install cosign CLI."
        )
    }
}

/// Cosign CLI fallback backend.
struct CosignCliBackend;

impl SignerBackend for CosignCliBackend {
    fn sign(&self, digest: &[u8]) -> Result<SignatureBundle> {
        use std::process::Command;

        let hex_digest = hex_encode(digest);

        // Check if cosign is available
        let status = Command::new("cosign").arg("version").output();
        if status.is_err() {
            anyhow::bail!(
                "cosign CLI not found. Install from https://docs.sigstore.dev/cosign/installation/"
            );
        }

        // Use cosign to sign the digest
        // cosign sign-blob --yes --bundle <bundle-path> - <<< <digest>
        let temp_dir = tempfile::tempdir()?;
        let bundle_path = temp_dir.path().join("bundle.json");
        let digest_file = temp_dir.path().join("digest.txt");
        std::fs::write(&digest_file, &hex_digest)?;

        let output = Command::new("cosign")
            .args([
                "sign-blob",
                "--yes",
                "--bundle",
                bundle_path.to_str().unwrap(),
                digest_file.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cosign sign-blob failed: {stderr}");
        }

        // Parse the bundle
        let bundle_json = std::fs::read_to_string(&bundle_path)?;
        let bundle: serde_json::Value = serde_json::from_str(&bundle_json)?;

        Ok(SignatureBundle {
            signature: bundle
                .get("Payload")
                .and_then(|p| p.get("body"))
                .and_then(|b| b.as_str())
                .unwrap_or("")
                .to_string(),
            certificate: bundle
                .get("Cert")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
            rekor_log_id: bundle
                .get("RekorBundle")
                .and_then(|r| r.get("LogID"))
                .and_then(|l| l.as_str())
                .map(String::from),
        })
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
