use colored::Colorize;
use skillguard_core::anthropic::export_as_anthropic;
use skillguard_core::SkillManifest;
use std::path::PathBuf;

pub fn run(format: &str, path: &str, json: bool) -> anyhow::Result<()> {
    if format != "anthropic" {
        anyhow::bail!(
            "Unsupported export format '{}'. Supported: anthropic",
            format
        );
    }

    let skill_dir = PathBuf::from(path);
    let manifest_path = skill_dir.join("skillguard.yaml");

    if !manifest_path.exists() {
        anyhow::bail!("No skillguard.yaml found in '{}'", path);
    }

    let manifest = SkillManifest::from_yaml_file(&manifest_path)?;

    if !json {
        println!(
            "{} Exporting '{}' as Anthropic Agent Skill...",
            "→".blue().bold(),
            manifest.name.cyan()
        );
    }

    // Generate SKILL.md
    let skill_md = export_as_anthropic(&manifest);

    // Write to output directory
    let output_dir = skill_dir.join("export-anthropic");
    std::fs::create_dir_all(&output_dir)?;

    let skill_md_path = output_dir.join("SKILL.md");
    std::fs::write(&skill_md_path, &skill_md)?;

    // Generate wrapper scripts that call skillguard run
    let scripts_dir = output_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    for action in &manifest.actions {
        let script = format!(
            r#"#!/usr/bin/env bash
# Auto-generated wrapper for skillguard action: {action_name}
# This script delegates to the SkillGuard sandbox for secure execution.
set -euo pipefail

PARAMS="${{1:-{{}}}}"
exec skillguard run "{skill_name}" --action "{action_name}" --params "$PARAMS" --output-json
"#,
            skill_name = manifest.name,
            action_name = action.name
        );

        let script_path = scripts_dir.join(format!("{}.sh", action.name));
        std::fs::write(&script_path, &script)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;
        }
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "exported",
                "format": "anthropic",
                "name": manifest.name,
                "output": output_dir.display().to_string(),
                "files": {
                    "skill_md": skill_md_path.display().to_string(),
                    "scripts": manifest.actions.iter().map(|a| format!("{}.sh", a.name)).collect::<Vec<_>>(),
                },
            }))?
        );
    } else {
        println!(
            "{} Exported to {}",
            "✓".green().bold(),
            output_dir.display().to_string().dimmed()
        );
        println!("  SKILL.md + {} wrapper scripts", manifest.actions.len());
    }

    Ok(())
}
