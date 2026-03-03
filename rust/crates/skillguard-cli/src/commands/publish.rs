use colored::Colorize;
use skillguard_core::SkillManifest;
use std::path::PathBuf;

pub async fn run(path: &str, sign: bool, json: bool) -> anyhow::Result<()> {
    let skill_dir = PathBuf::from(path);
    let manifest_path = skill_dir.join("skillguard.yaml");

    if !manifest_path.exists() {
        anyhow::bail!("No skillguard.yaml found in '{}'", path);
    }

    let manifest = SkillManifest::from_yaml_file(&manifest_path)?;

    if !json {
        println!(
            "{} Publishing '{}' v{}...",
            "→".blue().bold(),
            manifest.name.cyan(),
            manifest.version
        );
    }

    // Build first if no package exists
    let package_name = format!("{}-{}.tar.gz", manifest.name, manifest.version);
    let package_path = skill_dir.join(&package_name);
    if !package_path.exists() {
        if !json {
            println!("  Building package...");
        }
        super::build::run(path, sign, None, false).await?;
    }

    // Publish
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let index_path = PathBuf::from(&home).join(".skillguard").join("index");

    let publisher = skillguard_registry::Publisher::new(index_path);
    publisher.publish(&skill_dir, &manifest).await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "published",
                "name": manifest.name,
                "version": manifest.version,
            }))?
        );
    } else {
        println!(
            "{} Published '{}' v{}",
            "✓".green().bold(),
            manifest.name.cyan(),
            manifest.version
        );
    }

    Ok(())
}
