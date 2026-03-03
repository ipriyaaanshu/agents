use colored::Colorize;
use comfy_table::{Cell, Table};
use skillguard_core::SkillManifest;
use std::path::PathBuf;

pub fn run(skill: &str, json: bool) -> anyhow::Result<()> {
    let manifest = load_manifest(skill)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&manifest)?);
        return Ok(());
    }

    println!(
        "{} {}",
        manifest.name.cyan().bold(),
        manifest.version.dimmed()
    );
    println!("  {}", manifest.description);
    println!("  Author: {}", manifest.author);
    println!("  License: {}", manifest.license);

    if let Some(ref url) = manifest.homepage {
        println!("  Homepage: {}", url);
    }
    if let Some(ref url) = manifest.repository {
        println!("  Repository: {}", url);
    }

    // Permissions
    println!("\n{}", "Permissions:".bold());
    println!(
        "  Level: {}",
        format!("{}", manifest.permissions.level()).yellow()
    );

    if !manifest.permissions.network.is_empty() {
        println!("  Network:");
        for net in &manifest.permissions.network {
            let methods: Vec<String> = net.methods.iter().map(|m| format!("{m:?}")).collect();
            println!(
                "    {} [{}] ports: {:?}",
                net.domain,
                methods.join(", "),
                net.ports
            );
        }
    }

    if !manifest.permissions.filesystem.is_empty() {
        println!("  Filesystem:");
        for fs in &manifest.permissions.filesystem {
            let access: Vec<String> = fs.access.iter().map(|a| format!("{a:?}")).collect();
            println!("    {} [{}]", fs.path, access.join(", "));
        }
    }

    if manifest.permissions.subprocess {
        println!(
            "  Subprocess: {} (allowlist: {:?})",
            "enabled".red(),
            manifest.permissions.subprocess_allowlist
        );
    }

    // Actions
    if !manifest.actions.is_empty() {
        println!("\n{}", "Actions:".bold());
        let mut table = Table::new();
        table.set_header(vec!["Name", "Description", "Parameters"]);

        for action in &manifest.actions {
            let params: Vec<String> = action.parameters.keys().cloned().collect();
            table.add_row(vec![
                Cell::new(&action.name),
                Cell::new(&action.description),
                Cell::new(params.join(", ")),
            ]);
        }
        println!("{table}");
    }

    // Security
    println!("\n{}", "Security:".bold());
    println!("  SLSA Level: {}", manifest.security.slsa_level);
    if let Some(ref date) = manifest.security.audit_date {
        println!("  Last Audit: {date}");
    }
    if let Some(ref auditor) = manifest.security.auditor {
        println!("  Auditor: {auditor}");
    }

    if !manifest.keywords.is_empty() {
        println!("\n  Keywords: {}", manifest.keywords.join(", ").dimmed());
    }

    Ok(())
}

fn load_manifest(skill: &str) -> anyhow::Result<SkillManifest> {
    // Try as direct path first
    let path = PathBuf::from(skill);
    if path.is_dir() {
        let manifest_path = path.join("skillguard.yaml");
        if manifest_path.exists() {
            return Ok(SkillManifest::from_yaml_file(&manifest_path)?);
        }
    }

    // Try as a manifest file path
    if path.is_file() {
        return Ok(SkillManifest::from_yaml_file(&path)?);
    }

    // Try installed skills
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let installed_path = PathBuf::from(&home)
        .join(".skillguard")
        .join("skills")
        .join(skill)
        .join("skillguard.yaml");
    if installed_path.exists() {
        return Ok(SkillManifest::from_yaml_file(&installed_path)?);
    }

    anyhow::bail!("Skill '{}' not found", skill)
}
