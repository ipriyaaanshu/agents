use colored::Colorize;

pub async fn run(skill: &str, force: bool, skip_verify: bool, json: bool) -> anyhow::Result<()> {
    let client = skillguard_registry::RegistryClient::default_client()?;

    if !json {
        println!(
            "{} Installing skill '{}'...",
            "→".blue().bold(),
            skill.cyan()
        );
    }

    let install_path = client.install(skill, force).await?;

    // Verify unless skipped
    if !skip_verify {
        if !json {
            println!("  Verifying...");
        }
        // Run verification on installed skill
        let manifest_path = install_path.join("skillguard.yaml");
        if manifest_path.exists() {
            let manifest = skillguard_core::SkillManifest::from_yaml_file(&manifest_path)?;
            let issues = skillguard_core::validate::audit_manifest(&manifest);
            let criticals = issues
                .iter()
                .filter(|i| i.severity == skillguard_core::AuditSeverity::Critical)
                .count();
            if criticals > 0 && !json {
                eprintln!(
                    "  {} {} critical security issues found. Run 'skillguard audit {}' for details.",
                    "⚠".yellow(),
                    criticals,
                    skill
                );
            }
        }
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "installed",
                "skill": skill,
                "path": install_path.display().to_string(),
            }))?
        );
    } else {
        println!(
            "{} Installed '{}' to {}",
            "✓".green().bold(),
            skill.cyan(),
            install_path.display().to_string().dimmed()
        );
    }

    Ok(())
}
