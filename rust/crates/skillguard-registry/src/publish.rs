use crate::index::{IndexEntry, RegistryIndex};
use crate::package::PackageBuilder;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Publishes skills to the registry.
pub struct Publisher {
    index: RegistryIndex,
}

impl Publisher {
    pub fn new(index_path: PathBuf) -> Self {
        Self {
            index: RegistryIndex::open(index_path),
        }
    }

    /// Publish a skill to the registry.
    pub async fn publish(
        &self,
        skill_dir: &Path,
        manifest: &skillguard_core::SkillManifest,
    ) -> Result<()> {
        // Check if version already exists
        if let Some(existing) = self
            .index
            .lookup_version(&manifest.name, &manifest.version)?
        {
            if !existing.yanked {
                anyhow::bail!(
                    "Version {} of '{}' already published. Bump the version to publish again.",
                    manifest.version,
                    manifest.name
                );
            }
        }

        // Build the package
        let output_dir = tempfile::tempdir()?;
        let package_path = output_dir
            .path()
            .join(format!("{}-{}.tar.gz", manifest.name, manifest.version));
        let checksum = PackageBuilder::build(skill_dir, &package_path)?;

        // Upload to GitHub Releases
        self.upload_package(&manifest.name, &manifest.version, &package_path)
            .await?;

        // Add to index
        let entry = IndexEntry {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            checksum,
            yanked: false,
            description: Some(manifest.description.clone()),
            keywords: manifest.keywords.clone(),
        };
        self.index.add_entry(&entry)?;

        // TODO: Submit index PR via GitHub API
        tracing::info!(
            "Published {}@{} — submit index PR manually for now",
            manifest.name,
            manifest.version
        );

        Ok(())
    }

    async fn upload_package(&self, name: &str, version: &str, path: &Path) -> Result<()> {
        // TODO: Use GitHub Releases API to upload
        tracing::info!(
            "Would upload {} ({} bytes) as {name}-v{version}",
            path.display(),
            std::fs::metadata(path)?.len()
        );
        Ok(())
    }
}
