use colored::Colorize;
use skillguard_core::SkillManifest;
use std::path::PathBuf;

pub async fn run(skill: &str, strict: bool, json: bool) -> anyhow::Result<()> {
    let path = PathBuf::from(skill);

    // Load manifest
    let manifest_path = if path.is_dir() {
        path.join("skillguard.yaml")
    } else {
        path.clone()
    };

    let manifest = SkillManifest::from_yaml_file(&manifest_path)?;

    let mut checks = Vec::new();
    let mut all_pass = true;

    // Check 1: Manifest validates
    checks.push(("Manifest schema", true, "Valid".to_string()));

    // Check 2: SLSA level
    let slsa_ok = manifest.security.slsa_level > 0;
    if !slsa_ok && strict {
        all_pass = false;
    }
    checks.push((
        "SLSA provenance",
        slsa_ok,
        format!("Level {}", manifest.security.slsa_level),
    ));

    // Check 3: Look for signature
    let skill_dir = manifest_path.parent().unwrap_or(&path);
    let sig_files: Vec<_> = std::fs::read_dir(skill_dir)?
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "sig" || ext == "json")
                .unwrap_or(false)
                && e.path().to_string_lossy().contains("sig")
        })
        .collect();

    let has_sig = !sig_files.is_empty();
    if !has_sig {
        if strict {
            all_pass = false;
        }
        checks.push(("Sigstore signature", false, "Not found".to_string()));
    } else {
        checks.push(("Sigstore signature", true, "Present".to_string()));
    }

    // Check 4: Audit issues
    let issues = skillguard_core::validate::audit_manifest(&manifest);
    let critical = issues
        .iter()
        .filter(|i| i.severity == skillguard_core::AuditSeverity::Critical)
        .count();
    if critical > 0 {
        all_pass = false;
    }
    checks.push((
        "Security audit",
        critical == 0,
        format!("{} issues ({} critical)", issues.len(), critical),
    ));

    if json {
        let output = serde_json::json!({
            "name": manifest.name,
            "version": manifest.version,
            "verified": all_pass,
            "checks": checks.iter().map(|(name, pass, detail)| {
                serde_json::json!({
                    "check": name,
                    "pass": pass,
                    "detail": detail,
                })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(
            "{} Verifying '{}' v{}",
            "→".blue().bold(),
            manifest.name.cyan(),
            manifest.version
        );

        for (name, pass, detail) in &checks {
            let icon = if *pass {
                "✓".green().bold()
            } else {
                "✗".red().bold()
            };
            println!("  {icon} {name}: {detail}");
        }

        println!();
        if all_pass {
            println!("{} Verification passed", "✓".green().bold());
        } else {
            println!("{} Verification failed", "✗".red().bold());
        }
    }

    if !all_pass {
        std::process::exit(1);
    }

    Ok(())
}
