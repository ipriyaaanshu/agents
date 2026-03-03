use colored::Colorize;
use skillguard_core::SkillManifest;
use std::path::PathBuf;

pub fn run(
    skill: &str,
    action: &str,
    params: Option<&str>,
    dry_run: bool,
    json: bool,
) -> anyhow::Result<()> {
    let skill_dir = resolve_skill_path(skill)?;
    let manifest_path = skill_dir.join("skillguard.yaml");
    let manifest = SkillManifest::from_yaml_file(&manifest_path)?;

    // Verify action exists
    let _skill_action = manifest.find_action(action).ok_or_else(|| {
        anyhow::anyhow!(
            "Action '{}' not found in skill '{}'. Available: {}",
            action,
            manifest.name,
            manifest
                .actions
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let params_value: serde_json::Value = match params {
        Some(p) => {
            serde_json::from_str(p).map_err(|e| anyhow::anyhow!("Invalid JSON params: {e}"))?
        }
        None => serde_json::Value::Object(Default::default()),
    };

    if dry_run {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "dry_run": true,
                    "skill": manifest.name,
                    "action": action,
                    "params": params_value,
                    "permission_level": manifest.permissions.level().to_string(),
                }))?
            );
        } else {
            println!("{} Dry run:", "→".blue().bold());
            println!("  Skill: {}", manifest.name.cyan());
            println!("  Action: {}", action.yellow());
            println!("  Params: {}", params_value);
            println!(
                "  Permission level: {}",
                manifest.permissions.level().to_string().yellow()
            );
            println!("  Would execute in Wasmtime WASI sandbox");
        }
        return Ok(());
    }

    if !json {
        println!(
            "{} Running '{}.{}'...",
            "→".blue().bold(),
            manifest.name.cyan(),
            action.yellow()
        );
    }

    // Look for a compiled .wasm file
    let wasm_path = skill_dir.join("skill.wasm");
    if wasm_path.exists() {
        // Execute in sandbox
        let workspace = std::env::current_dir()?;
        let temp = std::env::temp_dir().join(format!("skillguard-{}", manifest.name));
        std::fs::create_dir_all(&temp)?;

        let capabilities = skillguard_sandbox::CapabilityGrants::from_permission(
            &manifest.permissions,
            &workspace,
            &temp,
        );
        let limits = skillguard_sandbox::ResourceLimits::default();

        let executor = skillguard_sandbox::SandboxExecutor::new()?;
        let result = executor.execute(&wasm_path, action, &params_value, &capabilities, &limits)?;

        if json {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            match result.status {
                skillguard_core::SkillStatus::Success => {
                    println!("{} Success", "✓".green().bold());
                    if let Some(data) = &result.data {
                        println!("{}", serde_json::to_string_pretty(data)?);
                    }
                }
                skillguard_core::SkillStatus::Error => {
                    println!(
                        "{} Error: {}",
                        "✗".red().bold(),
                        result.error_message.unwrap_or_default()
                    );
                }
                skillguard_core::SkillStatus::Denied => {
                    println!(
                        "{} Denied: {}",
                        "✗".red().bold(),
                        result.error_message.unwrap_or_default()
                    );
                }
                skillguard_core::SkillStatus::Timeout => {
                    println!(
                        "{} Timeout: {}",
                        "✗".yellow().bold(),
                        result.error_message.unwrap_or_default()
                    );
                }
            }
        }
    } else {
        anyhow::bail!(
            "No compiled skill.wasm found in '{}'. Run 'skillguard build' first.",
            skill_dir.display()
        );
    }

    Ok(())
}

fn resolve_skill_path(skill: &str) -> anyhow::Result<PathBuf> {
    // Try direct path
    let path = PathBuf::from(skill);
    if path.is_dir() && path.join("skillguard.yaml").exists() {
        return Ok(path);
    }

    // Try current directory
    let cwd = std::env::current_dir()?;
    let cwd_path = cwd.join(skill);
    if cwd_path.is_dir() && cwd_path.join("skillguard.yaml").exists() {
        return Ok(cwd_path);
    }

    // Try installed skills
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let installed = PathBuf::from(&home)
        .join(".skillguard")
        .join("skills")
        .join(skill);
    if installed.is_dir() && installed.join("skillguard.yaml").exists() {
        return Ok(installed);
    }

    anyhow::bail!("Skill '{}' not found", skill)
}
