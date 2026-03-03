use colored::Colorize;
use skillguard_core::SkillManifest;
use std::path::PathBuf;

pub async fn run(path: &str, sign: bool, output: Option<&str>, json: bool) -> anyhow::Result<()> {
    let skill_dir = PathBuf::from(path);
    let manifest_path = skill_dir.join("skillguard.yaml");

    if !manifest_path.exists() {
        anyhow::bail!("No skillguard.yaml found in '{}'", path);
    }

    let manifest = SkillManifest::from_yaml_file(&manifest_path)?;

    if !json {
        println!(
            "{} Building skill '{}'...",
            "→".blue().bold(),
            manifest.name.cyan()
        );
    }

    // Security scan before building
    let issues = skillguard_core::validate::audit_manifest(&manifest);
    let critical_count = issues
        .iter()
        .filter(|i| i.severity == skillguard_core::AuditSeverity::Critical)
        .count();
    if critical_count > 0 {
        anyhow::bail!(
            "Cannot build: {} critical security issues found. Run 'skillguard audit' for details.",
            critical_count
        );
    }

    // Determine output path
    let output_path = match output {
        Some(o) => PathBuf::from(o),
        None => skill_dir.join(format!("{}-{}.tar.gz", manifest.name, manifest.version)),
    };

    // Build the package
    let checksum = skillguard_registry::PackageBuilder::build(&skill_dir, &output_path)?;

    // Generate provenance
    let provenance = skillguard_signing::Provenance::new(
        &format!("{}-{}.tar.gz", manifest.name, manifest.version),
        &output_path,
        &format!("skillguard-cli/{}", env!("CARGO_PKG_VERSION")),
        manifest.build.reproducible,
    )?;
    let provenance_path = output_path.with_extension("provenance.json");
    provenance.to_file(&provenance_path)?;

    // Sign if requested
    let signature = if sign {
        if !json {
            println!("  Signing with Sigstore...");
        }
        match skillguard_signing::Signer::cosign_cli().sign_file(&output_path) {
            Ok(sig) => {
                let sig_path = output_path.with_extension("sig.json");
                std::fs::write(&sig_path, serde_json::to_string_pretty(&sig)?)?;
                Some(sig_path)
            }
            Err(e) => {
                if !json {
                    eprintln!(
                        "  {} Signing failed: {}. Package built without signature.",
                        "⚠".yellow(),
                        e
                    );
                }
                None
            }
        }
    } else {
        None
    };

    let file_size = std::fs::metadata(&output_path)?.len();

    if json {
        let mut result = serde_json::json!({
            "status": "built",
            "name": manifest.name,
            "version": manifest.version,
            "package": output_path.display().to_string(),
            "checksum": checksum,
            "size_bytes": file_size,
            "provenance": provenance_path.display().to_string(),
        });
        if let Some(sig) = signature {
            result["signature"] = serde_json::Value::String(sig.display().to_string());
        }
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "{} Built {} ({} bytes)",
            "✓".green().bold(),
            output_path.display().to_string().dimmed(),
            file_size
        );
        println!("  Checksum: {}", checksum.dimmed());
        if let Some(sig) = signature {
            println!("  Signature: {}", sig.display().to_string().dimmed());
        }
    }

    Ok(())
}
