use crate::error::{Result, SkillGuardError};
use crate::manifest::SkillManifest;
use crate::permission::{
    EnvironmentPermission, FilesystemAccess, FilesystemPermission, HttpMethod, NetworkPermission,
    Permission,
};
use std::path::{Path, PathBuf};

/// Format of the skill source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillFormat {
    /// Native skillguard.yaml + skill code
    Native,
    /// Anthropic SKILL.md + optional scripts/
    AnthropicAgentSkill,
    /// SKILL.md + skillguard.yaml sidecar
    Wrapped,
}

/// Metadata parsed from an Anthropic SKILL.md file.
#[derive(Debug, Clone)]
pub struct AnthropicSkillMeta {
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub resources: Vec<PathBuf>,
}

/// A unified skill that can represent any format.
#[derive(Debug, Clone)]
pub struct UnifiedSkill {
    pub format: SkillFormat,
    pub manifest: SkillManifest,
    pub anthropic_metadata: Option<AnthropicSkillMeta>,
}

/// Parse YAML frontmatter from a SKILL.md file.
/// Expected format:
/// ```text
/// ---
/// name: my-skill
/// description: Does something
/// ---
/// Instructions body here...
/// ```
pub fn parse_skill_md(content: &str) -> Result<AnthropicSkillMeta> {
    let content = content.trim();
    if !content.starts_with("---") {
        return Err(SkillGuardError::AnthropicParseError(
            "SKILL.md must start with YAML frontmatter (---)".into(),
        ));
    }

    let after_first = &content[3..];
    let end = after_first.find("---").ok_or_else(|| {
        SkillGuardError::AnthropicParseError("Missing closing --- for frontmatter".into())
    })?;

    let frontmatter = &after_first[..end];
    let body = after_first[end + 3..].trim();

    // Parse frontmatter as YAML
    let fm: serde_yaml::Value = serde_yaml::from_str(frontmatter).map_err(|e| {
        SkillGuardError::AnthropicParseError(format!("Invalid YAML frontmatter: {e}"))
    })?;

    let name = fm
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            SkillGuardError::AnthropicParseError("Missing 'name' in frontmatter".into())
        })?
        .to_string();

    let description = fm
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(AnthropicSkillMeta {
        name,
        description,
        instructions: body.to_string(),
        resources: Vec::new(),
    })
}

/// Infer permissions by scanning script contents for common patterns.
pub fn infer_permissions_from_scripts(scripts_dir: &Path) -> Result<Permission> {
    let mut perm = Permission::default();

    if !scripts_dir.exists() || !scripts_dir.is_dir() {
        return Ok(perm);
    }

    let entries: Vec<_> = std::fs::read_dir(scripts_dir)?
        .filter_map(|e| e.ok())
        .collect();

    for entry in entries {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Detect network usage
        let network_patterns = [
            "requests.get",
            "requests.post",
            "requests.put",
            "requests.delete",
            "httpx.",
            "urllib.request",
            "http.client",
            "aiohttp",
            "fetch(",
            "curl",
        ];
        if network_patterns.iter().any(|p| content.contains(p)) && perm.network.is_empty() {
            perm.network.push(NetworkPermission {
                domain: "*".into(),
                methods: vec![HttpMethod::Get, HttpMethod::Post],
                ports: vec![443, 80],
            });
        }

        // Detect filesystem usage
        let fs_patterns = [
            "open(",
            "Path(",
            "pathlib",
            "os.path",
            "shutil",
            "glob.glob",
            "with open",
        ];
        if fs_patterns.iter().any(|p| content.contains(p)) && perm.filesystem.is_empty() {
            perm.filesystem.push(FilesystemPermission {
                path: "${WORKSPACE}/**".into(),
                access: vec![FilesystemAccess::Read, FilesystemAccess::Write],
            });
        }

        // Detect env var usage
        let env_patterns = ["os.environ", "os.getenv", "env::", "std::env"];
        if env_patterns.iter().any(|p| content.contains(p)) && perm.environment.is_empty() {
            perm.environment.push(EnvironmentPermission {
                name: "*".into(),
                required: false,
                sensitive: false,
            });
        }

        // Detect subprocess usage
        let subprocess_patterns = [
            "subprocess",
            "os.system",
            "os.popen",
            "Popen",
            "Command::new",
        ];
        if subprocess_patterns.iter().any(|p| content.contains(p)) {
            perm.subprocess = true;
        }
    }

    Ok(perm)
}

/// Detect the skill format in a directory.
pub fn detect_skill_format(dir: &Path) -> SkillFormat {
    let has_manifest = dir.join("skillguard.yaml").exists();
    let has_skill_md = dir.join("SKILL.md").exists();

    match (has_manifest, has_skill_md) {
        (true, true) => SkillFormat::Wrapped,
        (true, false) => SkillFormat::Native,
        (false, true) => SkillFormat::AnthropicAgentSkill,
        (false, false) => SkillFormat::Native, // default
    }
}

/// Load a unified skill from a directory, handling all formats.
pub fn load_unified_skill(dir: &Path) -> Result<UnifiedSkill> {
    let format = detect_skill_format(dir);

    match format {
        SkillFormat::Native => {
            let manifest_path = dir.join("skillguard.yaml");
            let manifest = SkillManifest::from_yaml_file(&manifest_path)?;
            Ok(UnifiedSkill {
                format,
                manifest,
                anthropic_metadata: None,
            })
        }
        SkillFormat::AnthropicAgentSkill => {
            let skill_md_path = dir.join("SKILL.md");
            let content = std::fs::read_to_string(&skill_md_path)?;
            let meta = parse_skill_md(&content)?;

            // Infer permissions from scripts
            let permissions = infer_permissions_from_scripts(&dir.join("scripts"))?;

            // Build a manifest from the SKILL.md metadata
            let manifest = SkillManifest {
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

            Ok(UnifiedSkill {
                format,
                manifest,
                anthropic_metadata: Some(meta),
            })
        }
        SkillFormat::Wrapped => {
            let manifest_path = dir.join("skillguard.yaml");
            let manifest = SkillManifest::from_yaml_file(&manifest_path)?;

            let skill_md_path = dir.join("SKILL.md");
            let content = std::fs::read_to_string(&skill_md_path)?;
            let meta = parse_skill_md(&content)?;

            Ok(UnifiedSkill {
                format,
                manifest,
                anthropic_metadata: Some(meta),
            })
        }
    }
}

/// Generate a SKILL.md from a SkillManifest.
pub fn export_as_anthropic(manifest: &SkillManifest) -> String {
    let mut output = String::new();

    // Frontmatter
    output.push_str("---\n");
    output.push_str(&format!("name: {}\n", manifest.name));
    output.push_str(&format!("description: {}\n", manifest.description));
    output.push_str("---\n\n");

    // Instructions body
    output.push_str(&format!("# {}\n\n", manifest.name));
    output.push_str(&format!("{}\n\n", manifest.description));

    if !manifest.actions.is_empty() {
        output.push_str("## Actions\n\n");
        for action in &manifest.actions {
            output.push_str(&format!("### {}\n\n", action.name));
            output.push_str(&format!("{}\n\n", action.description));

            if !action.parameters.is_empty() {
                output.push_str("**Parameters:**\n");
                for (name, schema) in &action.parameters {
                    let type_str = schema.get("type").and_then(|v| v.as_str()).unwrap_or("any");
                    let desc = schema
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    output.push_str(&format!("- `{name}` ({type_str}): {desc}\n"));
                }
                output.push('\n');
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_md() {
        let content = r#"---
name: my-skill
description: A useful skill
---
These are the instructions for the skill.

Use it wisely.
"#;
        let meta = parse_skill_md(content).unwrap();
        assert_eq!(meta.name, "my-skill");
        assert_eq!(meta.description, "A useful skill");
        assert!(meta.instructions.contains("Use it wisely"));
    }

    #[test]
    fn test_parse_skill_md_no_frontmatter() {
        let content = "Just some text";
        assert!(parse_skill_md(content).is_err());
    }

    #[test]
    fn test_export_as_anthropic() {
        let manifest = SkillManifest {
            name: "test-skill".into(),
            version: "1.0.0".into(),
            description: "A test skill".into(),
            author: "test".into(),
            license: "Apache-2.0".into(),
            homepage: None,
            repository: None,
            permissions: Default::default(),
            adapters: Default::default(),
            build: Default::default(),
            security: Default::default(),
            actions: Vec::new(),
            keywords: Vec::new(),
        };
        let md = export_as_anthropic(&manifest);
        assert!(md.contains("name: test-skill"));
        assert!(md.contains("# test-skill"));
    }
}
