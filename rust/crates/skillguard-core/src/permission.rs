use serde::{Deserialize, Serialize};

/// HTTP methods for network permissions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

/// Filesystem access levels.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilesystemAccess {
    Read,
    Write,
    Execute,
}

/// Computed permission level (not serialized).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    Minimal,
    Restricted,
    Standard,
    Privileged,
}

impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimal => write!(f, "minimal"),
            Self::Restricted => write!(f, "restricted"),
            Self::Standard => write!(f, "standard"),
            Self::Privileged => write!(f, "privileged"),
        }
    }
}

/// Network access permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermission {
    pub domain: String,
    #[serde(default = "default_http_methods")]
    pub methods: Vec<HttpMethod>,
    #[serde(default = "default_ports")]
    pub ports: Vec<u16>,
}

fn default_http_methods() -> Vec<HttpMethod> {
    vec![HttpMethod::Get]
}

fn default_ports() -> Vec<u16> {
    vec![443, 80]
}

/// Filesystem access permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemPermission {
    pub path: String,
    #[serde(default = "default_fs_access")]
    pub access: Vec<FilesystemAccess>,
}

fn default_fs_access() -> Vec<FilesystemAccess> {
    vec![FilesystemAccess::Read]
}

/// Environment variable permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentPermission {
    pub name: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub sensitive: bool,
}

/// Aggregate permissions for a skill.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Permission {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub network: Vec<NetworkPermission>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filesystem: Vec<FilesystemPermission>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub environment: Vec<EnvironmentPermission>,
    #[serde(default)]
    pub subprocess: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subprocess_allowlist: Vec<String>,
}

impl Permission {
    /// Compute the permission level based on what's declared.
    pub fn level(&self) -> PermissionLevel {
        let has_network = !self.network.is_empty();
        let has_filesystem = !self.filesystem.is_empty();
        let has_subprocess = self.subprocess;

        if !has_network && !has_filesystem && !has_subprocess {
            return PermissionLevel::Minimal;
        }

        if has_subprocess && self.subprocess_allowlist.is_empty() {
            return PermissionLevel::Privileged;
        }

        if has_subprocess {
            return PermissionLevel::Standard;
        }

        PermissionLevel::Restricted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_level_minimal() {
        let perm = Permission::default();
        assert_eq!(perm.level(), PermissionLevel::Minimal);
    }

    #[test]
    fn test_permission_level_restricted() {
        let perm = Permission {
            network: vec![NetworkPermission {
                domain: "example.com".into(),
                methods: vec![HttpMethod::Get],
                ports: vec![443],
            }],
            ..Default::default()
        };
        assert_eq!(perm.level(), PermissionLevel::Restricted);
    }

    #[test]
    fn test_permission_level_standard() {
        let perm = Permission {
            subprocess: true,
            subprocess_allowlist: vec!["ls".into()],
            ..Default::default()
        };
        assert_eq!(perm.level(), PermissionLevel::Standard);
    }

    #[test]
    fn test_permission_level_privileged() {
        let perm = Permission {
            subprocess: true,
            subprocess_allowlist: vec![],
            ..Default::default()
        };
        assert_eq!(perm.level(), PermissionLevel::Privileged);
    }
}
