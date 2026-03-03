use crate::action::SkillAction;
use crate::error::{Result, SkillGuardError};
use crate::permission::Permission;
use crate::validate::SkillName;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Adapter framework version constraints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdapterConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openclaw: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub langchain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crewai: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autogpt: Option<String>,
}

/// Build configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_true")]
    pub reproducible: bool,
    #[serde(default = "default_base_image")]
    pub base: String,
    #[serde(default = "default_python_version")]
    pub python_version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_base_image() -> String {
    "skillguard/python:3.11-minimal".into()
}

fn default_python_version() -> String {
    "3.11".into()
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            reproducible: true,
            base: default_base_image(),
            python_version: default_python_version(),
            dependencies: Vec::new(),
        }
    }
}

/// Security metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auditor: Option<String>,
    #[serde(default)]
    pub slsa_level: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cve_scan_date: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub known_vulnerabilities: Vec<String>,
}

/// The complete skill manifest — the security contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    #[serde(default = "default_license")]
    pub license: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(default)]
    pub permissions: Permission,
    #[serde(default, skip_serializing_if = "is_default_adapter")]
    pub adapters: AdapterConfig,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub security: SecurityMetadata,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<SkillAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

fn default_license() -> String {
    "Apache-2.0".into()
}

fn is_default_adapter(config: &AdapterConfig) -> bool {
    config.openclaw.is_none()
        && config.langchain.is_none()
        && config.crewai.is_none()
        && config.mcp.is_none()
        && config.autogpt.is_none()
}

impl SkillManifest {
    /// Load and validate a manifest from a YAML file.
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::from_yaml_str(&content)
    }

    /// Parse and validate a manifest from a YAML string.
    pub fn from_yaml_str(yaml: &str) -> Result<Self> {
        let manifest: Self = serde_yaml::from_str(yaml)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Write the manifest to a YAML file.
    pub fn to_yaml_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path.as_ref(), yaml)?;
        Ok(())
    }

    /// Serialize to a YAML string.
    pub fn to_yaml_string(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    /// Validate the manifest fields.
    pub fn validate(&self) -> Result<()> {
        // Validate skill name
        SkillName::new(&self.name)?;

        // Validate version
        semver::Version::parse(&self.version).map_err(|e| {
            SkillGuardError::InvalidManifest(format!("Invalid version '{}': {}", self.version, e))
        })?;

        // Validate SLSA level
        if self.security.slsa_level > 4 {
            return Err(SkillGuardError::InvalidManifest(format!(
                "SLSA level must be 0-4, got {}",
                self.security.slsa_level
            )));
        }

        // Validate actions have names
        for action in &self.actions {
            if action.name.is_empty() {
                return Err(SkillGuardError::InvalidManifest(
                    "Action name cannot be empty".into(),
                ));
            }
        }

        // Validate network permissions
        for net in &self.permissions.network {
            if net.domain.is_empty() {
                return Err(SkillGuardError::InvalidManifest(
                    "Network permission domain cannot be empty".into(),
                ));
            }
        }

        Ok(())
    }

    /// Find an action by name.
    pub fn find_action(&self, name: &str) -> Option<&SkillAction> {
        self.actions.iter().find(|a| a.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_manifest() {
        let yaml = r#"
name: test-skill
version: 1.0.0
description: A test skill
author: test-author
"#;
        let manifest = SkillManifest::from_yaml_str(yaml).unwrap();
        assert_eq!(manifest.name, "test-skill");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.license, "Apache-2.0");
        assert!(manifest.actions.is_empty());
    }

    #[test]
    fn test_invalid_name() {
        let yaml = r#"
name: Invalid_Name
version: 1.0.0
description: Bad
author: test
"#;
        assert!(SkillManifest::from_yaml_str(yaml).is_err());
    }

    #[test]
    fn test_invalid_version() {
        let yaml = r#"
name: test-skill
version: not-semver
description: Bad
author: test
"#;
        assert!(SkillManifest::from_yaml_str(yaml).is_err());
    }

    #[test]
    fn test_roundtrip_yaml() {
        let yaml = r#"
name: test-skill
version: 1.0.0
description: A test skill
author: test-author
permissions:
  network:
    - domain: example.com
      methods: [GET, POST]
      ports: [443]
  filesystem:
    - path: "${WORKSPACE}/**"
      access: [read, write]
actions:
  - name: do-thing
    description: Does a thing
keywords:
  - test
"#;
        let manifest = SkillManifest::from_yaml_str(yaml).unwrap();
        let output = manifest.to_yaml_string().unwrap();
        let reparsed = SkillManifest::from_yaml_str(&output).unwrap();
        assert_eq!(manifest.name, reparsed.name);
        assert_eq!(manifest.actions.len(), reparsed.actions.len());
    }
}
