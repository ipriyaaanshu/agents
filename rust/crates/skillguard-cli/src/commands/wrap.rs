use colored::Colorize;
use skillguard_core::anthropic::{infer_permissions_from_scripts, parse_skill_md};
use std::path::PathBuf;

pub fn run(skill_dir: &str, output: Option<&str>, json: bool) -> anyhow::Result<()> {
    let dir = PathBuf::from(skill_dir);
    let skill_md_path = dir.join("SKILL.md");

    if !skill_md_path.exists() {
        anyhow::bail!(
            "No SKILL.md found in '{}'. This command wraps Anthropic Agent Skills.",
            skill_dir
        );
    }

    let content = std::fs::read_to_string(&skill_md_path)?;
    let meta = parse_skill_md(&content)?;

    if !json {
        println!(
            "{} Wrapping Anthropic Agent Skill '{}'...",
            "→".blue().bold(),
            meta.name.cyan()
        );
    }

    // Infer permissions from scripts
    let scripts_dir = dir.join("scripts");
    let permissions = infer_permissions_from_scripts(&scripts_dir)?;

    if !json {
        println!(
            "  Inferred permission level: {}",
            permissions.level().to_string().yellow()
        );
        if !permissions.network.is_empty() {
            println!("  Network access detected");
        }
        if !permissions.filesystem.is_empty() {
            println!("  Filesystem access detected");
        }
        if permissions.subprocess {
            println!("  {} Subprocess access detected", "⚠".yellow());
        }
    }

    // Generate skillguard.yaml
    let manifest = skillguard_core::SkillManifest {
        name: meta.name.clone(),
        version: "0.1.0".into(),
        description: meta.description.clone(),
        author: "unknown".into(),
        license: "Apache-2.0".into(),
        homepage: None,
        repository: None,
        permissions,
        adapters: Default::default(),
        build: Default::default(),
        security: Default::default(),
        actions: Vec::new(),
        keywords: Vec::new(),
    };

    let output_dir = match output {
        Some(o) => {
            let out = PathBuf::from(o);
            std::fs::create_dir_all(&out)?;
            out
        }
        None => dir.clone(),
    };

    let manifest_path = output_dir.join("skillguard.yaml");
    manifest.to_yaml_file(&manifest_path)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "wrapped",
                "name": meta.name,
                "manifest": manifest_path.display().to_string(),
                "permission_level": manifest.permissions.level().to_string(),
            }))?
        );
    } else {
        println!(
            "{} Generated {} alongside SKILL.md",
            "✓".green().bold(),
            manifest_path.display().to_string().dimmed()
        );
        println!(
            "\n  Review the generated manifest, then run:\n    skillguard audit {}",
            output_dir.display()
        );
    }

    Ok(())
}
