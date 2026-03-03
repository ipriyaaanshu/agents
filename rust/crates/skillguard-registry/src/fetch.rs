use crate::index::RegistryIndex;
use crate::package::PackageReader;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Registry client for downloading and installing skills.
pub struct RegistryClient {
    index: RegistryIndex,
    cache_dir: PathBuf,
    skills_dir: PathBuf,
}

impl RegistryClient {
    pub fn new(index_path: PathBuf, cache_dir: PathBuf, skills_dir: PathBuf) -> Self {
        Self {
            index: RegistryIndex::open(index_path),
            cache_dir,
            skills_dir,
        }
    }

    /// Default client using ~/.skillguard paths.
    pub fn default_client() -> Result<Self> {
        let home = dirs_home()?;
        let sg_dir = home.join(".skillguard");
        Ok(Self::new(
            sg_dir.join("index"),
            sg_dir.join("cache"),
            sg_dir.join("skills"),
        ))
    }

    /// Install a skill by name (latest version).
    pub async fn install(&self, name: &str, force: bool) -> Result<PathBuf> {
        let entry = self
            .index
            .latest(name)?
            .ok_or_else(|| anyhow::anyhow!("Skill '{}' not found in registry", name))?;

        let dest = self.skills_dir.join(name);
        if dest.exists() && !force {
            anyhow::bail!(
                "Skill '{}' already installed. Use --force to overwrite.",
                name
            );
        }

        // Download the package
        let cache_file = self
            .cache_dir
            .join(format!("{}-{}.tar.gz", name, entry.version));

        if !cache_file.exists() {
            std::fs::create_dir_all(&self.cache_dir)?;
            self.download_package(name, &entry.version, &cache_file)
                .await?;
        }

        // Verify checksum
        if !PackageReader::verify_checksum(&cache_file, &entry.checksum)? {
            anyhow::bail!("Checksum mismatch for {name}@{}", entry.version);
        }

        // Extract
        std::fs::create_dir_all(&dest)?;
        PackageReader::extract(&cache_file, &dest)?;

        Ok(dest)
    }

    /// Download a package from the registry storage.
    async fn download_package(&self, name: &str, version: &str, dest: &Path) -> Result<()> {
        // For now, construct a GitHub Releases URL
        let url = format!(
            "https://github.com/skillguard/registry/releases/download/{name}-v{version}/{name}-{version}.tar.gz"
        );

        let response = reqwest::get(&url).await?;
        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to download {name}@{version}: HTTP {}",
                response.status()
            );
        }

        let bytes = response.bytes().await?;
        std::fs::write(dest, &bytes)?;
        Ok(())
    }

    /// List installed skills.
    pub fn list_installed(&self) -> Result<Vec<String>> {
        let mut skills = Vec::new();
        if !self.skills_dir.exists() {
            return Ok(skills);
        }

        for entry in std::fs::read_dir(&self.skills_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    skills.push(name.to_string());
                }
            }
        }

        skills.sort();
        Ok(skills)
    }

    /// Get the path to an installed skill.
    pub fn installed_skill_path(&self, name: &str) -> Option<PathBuf> {
        let path = self.skills_dir.join(name);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
}

fn dirs_home() -> Result<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .map_err(|_| anyhow::anyhow!("Cannot determine home directory"))
}
