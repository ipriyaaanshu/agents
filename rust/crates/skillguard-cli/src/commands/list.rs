use colored::Colorize;
use comfy_table::{Cell, Table};
use skillguard_core::SkillManifest;
use std::path::PathBuf;

pub fn run(_installed: bool, _all: bool, json: bool) -> anyhow::Result<()> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let skills_dir = PathBuf::from(&home).join(".skillguard").join("skills");

    let mut skills = Vec::new();

    if skills_dir.exists() {
        for entry in std::fs::read_dir(&skills_dir)?.flatten() {
            if entry.path().is_dir() {
                let manifest_path = entry.path().join("skillguard.yaml");
                if manifest_path.exists() {
                    if let Ok(manifest) = SkillManifest::from_yaml_file(&manifest_path) {
                        skills.push(manifest);
                    }
                }
            }
        }
    }

    if json {
        let output: Vec<serde_json::Value> = skills
            .iter()
            .map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "version": m.version,
                    "description": m.description,
                    "permission_level": m.permissions.level().to_string(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if skills.is_empty() {
        println!("No skills installed.");
        println!(
            "  Install skills with: {}",
            "skillguard install <name>".cyan()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Name", "Version", "Description", "Permission Level"]);

    for m in &skills {
        table.add_row(vec![
            Cell::new(&m.name),
            Cell::new(&m.version),
            Cell::new(&m.description),
            Cell::new(m.permissions.level().to_string()),
        ]);
    }

    println!("{}", "Installed Skills:".bold());
    println!("{table}");

    Ok(())
}
