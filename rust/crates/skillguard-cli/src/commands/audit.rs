use colored::Colorize;
use skillguard_core::validate::{audit_manifest, scan_source_code, AuditSeverity};
use skillguard_core::SkillManifest;
use std::path::PathBuf;

pub fn run(path: &str, _fix: bool, json: bool) -> anyhow::Result<()> {
    let skill_dir = PathBuf::from(path);
    let manifest_path = skill_dir.join("skillguard.yaml");

    if !manifest_path.exists() {
        anyhow::bail!(
            "No skillguard.yaml found in '{}'. Run 'skillguard init' first.",
            path
        );
    }

    let manifest = SkillManifest::from_yaml_file(&manifest_path)?;
    let mut all_issues = audit_manifest(&manifest);

    // Scan source files for dangerous patterns
    let scan_extensions = ["py", "rs", "js", "ts", "sh"];
    if let Ok(entries) = std::fs::read_dir(&skill_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if scan_extensions.contains(&ext) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let filename = path.file_name().unwrap().to_string_lossy().to_string();
                        all_issues.extend(scan_source_code(&content, &filename));
                    }
                }
            }
        }
    }

    // Also scan scripts/ directory (for Anthropic Agent Skills)
    let scripts_dir = skill_dir.join("scripts");
    if scripts_dir.exists() {
        scan_directory_recursive(&scripts_dir, &scan_extensions, &mut all_issues)?;
    }

    if json {
        let output: Vec<serde_json::Value> = all_issues
            .iter()
            .map(|issue| {
                serde_json::json!({
                    "severity": issue.severity.to_string(),
                    "message": issue.message,
                    "file": issue.file,
                    "line": issue.line,
                    "fix": issue.fix_suggestion,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(
            "{} Auditing skill '{}'",
            "→".blue().bold(),
            manifest.name.cyan()
        );
        println!(
            "  Permission level: {}",
            format_permission_level(&manifest.permissions.level())
        );
        println!();

        if all_issues.is_empty() {
            println!("{} No issues found", "✓".green().bold());
        } else {
            let criticals = all_issues
                .iter()
                .filter(|i| i.severity == AuditSeverity::Critical)
                .count();
            let errors = all_issues
                .iter()
                .filter(|i| i.severity == AuditSeverity::Error)
                .count();
            let warnings = all_issues
                .iter()
                .filter(|i| i.severity == AuditSeverity::Warning)
                .count();
            let infos = all_issues
                .iter()
                .filter(|i| i.severity == AuditSeverity::Info)
                .count();

            for issue in &all_issues {
                let severity = match issue.severity {
                    AuditSeverity::Critical => "CRITICAL".red().bold(),
                    AuditSeverity::Error => "ERROR".red(),
                    AuditSeverity::Warning => "WARNING".yellow(),
                    AuditSeverity::Info => "INFO".blue(),
                };

                let location = match (&issue.file, issue.line) {
                    (Some(f), Some(l)) => format!(" ({f}:{l})"),
                    (Some(f), None) => format!(" ({f})"),
                    _ => String::new(),
                };

                println!("  [{severity}]{location} {}", issue.message);

                if let Some(fix) = &issue.fix_suggestion {
                    println!("    Fix: {}", fix.dimmed());
                }
            }

            println!();
            println!(
                "  Summary: {} critical, {} errors, {} warnings, {} info",
                criticals, errors, warnings, infos
            );
        }
    }

    Ok(())
}

fn scan_directory_recursive(
    dir: &PathBuf,
    extensions: &[&str],
    issues: &mut Vec<skillguard_core::validate::AuditIssue>,
) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_directory_recursive(&path, extensions, issues)?;
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.contains(&ext) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let filename = path.to_string_lossy().to_string();
                    issues.extend(scan_source_code(&content, &filename));
                }
            }
        }
    }
    Ok(())
}

fn format_permission_level(level: &skillguard_core::PermissionLevel) -> colored::ColoredString {
    match level {
        skillguard_core::PermissionLevel::Minimal => "MINIMAL".green(),
        skillguard_core::PermissionLevel::Restricted => "RESTRICTED".yellow(),
        skillguard_core::PermissionLevel::Standard => "STANDARD".yellow().bold(),
        skillguard_core::PermissionLevel::Privileged => "PRIVILEGED".red().bold(),
    }
}
