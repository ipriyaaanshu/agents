use skillguard_core::{FilesystemAccess, Permission};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Mapped WASI capabilities derived from skill permissions.
#[derive(Debug, Clone)]
pub struct CapabilityGrants {
    /// Directories to preopen as read-only.
    pub readonly_dirs: Vec<PathBuf>,
    /// Directories to preopen as read-write.
    pub readwrite_dirs: Vec<PathBuf>,
    /// Allowed network hosts (domain:port).
    pub allowed_hosts: Vec<String>,
    /// Allowed environment variable names.
    pub allowed_env_vars: HashSet<String>,
    /// Allowed subprocess commands.
    pub subprocess_allowlist: Vec<String>,
    /// Whether subprocess execution is allowed at all.
    pub subprocess_enabled: bool,
}

impl CapabilityGrants {
    /// Build capability grants from a Permission declaration and workspace path.
    pub fn from_permission(perm: &Permission, workspace: &Path, temp_dir: &Path) -> Self {
        let mut readonly_dirs = Vec::new();
        let mut readwrite_dirs = Vec::new();

        for fs_perm in &perm.filesystem {
            let resolved = fs_perm
                .path
                .replace("${WORKSPACE}", &workspace.to_string_lossy())
                .replace("${TEMP}", &temp_dir.to_string_lossy());

            // Strip glob suffixes for directory preopens
            let dir_path = resolved.trim_end_matches("/**").trim_end_matches("/*");
            let path = PathBuf::from(dir_path);

            let has_write = fs_perm
                .access
                .iter()
                .any(|a| matches!(a, FilesystemAccess::Write));

            if has_write {
                readwrite_dirs.push(path);
            } else {
                readonly_dirs.push(path);
            }
        }

        let mut allowed_hosts = Vec::new();
        for net_perm in &perm.network {
            for port in &net_perm.ports {
                allowed_hosts.push(format!("{}:{}", net_perm.domain, port));
            }
        }

        let allowed_env_vars: HashSet<String> =
            perm.environment.iter().map(|e| e.name.clone()).collect();

        Self {
            readonly_dirs,
            readwrite_dirs,
            allowed_hosts,
            allowed_env_vars,
            subprocess_allowlist: perm.subprocess_allowlist.clone(),
            subprocess_enabled: perm.subprocess,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skillguard_core::{FilesystemPermission, HttpMethod, NetworkPermission};

    #[test]
    fn test_capability_from_permission() {
        let perm = Permission {
            filesystem: vec![FilesystemPermission {
                path: "${WORKSPACE}/**".into(),
                access: vec![FilesystemAccess::Read, FilesystemAccess::Write],
            }],
            network: vec![NetworkPermission {
                domain: "api.example.com".into(),
                methods: vec![HttpMethod::Get],
                ports: vec![443],
            }],
            ..Default::default()
        };

        let workspace = PathBuf::from("/home/user/project");
        let temp = PathBuf::from("/tmp/skill");
        let caps = CapabilityGrants::from_permission(&perm, &workspace, &temp);

        assert_eq!(
            caps.readwrite_dirs,
            vec![PathBuf::from("/home/user/project")]
        );
        assert!(caps.readonly_dirs.is_empty());
        assert_eq!(caps.allowed_hosts, vec!["api.example.com:443"]);
    }
}
