use anyhow::Result;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use std::path::Path;
use tar::{Archive, Builder};

/// Builds skill packages (tar.gz).
pub struct PackageBuilder;

impl PackageBuilder {
    /// Package a skill directory into a tar.gz archive.
    pub fn build(skill_dir: &Path, output: &Path) -> Result<String> {
        let file = std::fs::File::create(output)?;
        let enc = GzEncoder::new(file, Compression::default());
        let mut tar = Builder::new(enc);

        // Add all files from the skill directory
        tar.append_dir_all(".", skill_dir)?;
        tar.finish()?;

        // Compute checksum
        let content = std::fs::read(output)?;
        let digest = Sha256::digest(&content);
        let checksum = digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();

        Ok(format!("sha256:{checksum}"))
    }
}

/// Reads and extracts skill packages.
pub struct PackageReader;

impl PackageReader {
    /// Extract a tar.gz package to a directory.
    pub fn extract(archive_path: &Path, dest: &Path) -> Result<()> {
        let file = std::fs::File::open(archive_path)?;
        let dec = GzDecoder::new(file);
        let mut archive = Archive::new(dec);
        archive.unpack(dest)?;
        Ok(())
    }

    /// Verify the checksum of a package file.
    pub fn verify_checksum(path: &Path, expected: &str) -> Result<bool> {
        let content = std::fs::read(path)?;
        let digest = Sha256::digest(&content);
        let actual = format!(
            "sha256:{}",
            digest
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        Ok(actual == expected)
    }
}
