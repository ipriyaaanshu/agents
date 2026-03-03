use skillguard_core::*;
use std::path::PathBuf;

fn skills_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("skills")
}

#[test]
fn test_parse_file_ops_skill() {
    let path = skills_dir().join("file-ops/skillguard.yaml");
    let manifest = SkillManifest::from_yaml_file(&path).expect("Failed to parse file-ops manifest");

    assert_eq!(manifest.name, "file-ops");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.author, "skillguard-official");
    assert_eq!(manifest.license, "Apache-2.0");

    // Permissions
    assert!(manifest.permissions.network.is_empty());
    assert_eq!(manifest.permissions.filesystem.len(), 1);
    assert_eq!(manifest.permissions.filesystem[0].path, "${WORKSPACE}/**");
    assert!(!manifest.permissions.subprocess);
    assert_eq!(manifest.permissions.level(), PermissionLevel::Restricted);

    // Actions
    assert_eq!(manifest.actions.len(), 4);
    let action_names: Vec<&str> = manifest.actions.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(action_names, vec!["read", "write", "list", "search"]);

    // Keywords
    assert_eq!(
        manifest.keywords,
        vec!["file", "filesystem", "read", "write", "search"]
    );
}

#[test]
fn test_parse_web_fetch_skill() {
    let path = skills_dir().join("web-fetch/skillguard.yaml");
    let manifest =
        SkillManifest::from_yaml_file(&path).expect("Failed to parse web-fetch manifest");

    assert_eq!(manifest.name, "web-fetch");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.description, "Fetch and parse web content securely");

    // Permissions
    assert_eq!(manifest.permissions.network.len(), 1);
    assert_eq!(manifest.permissions.network[0].domain, "*");
    assert_eq!(manifest.permissions.filesystem.len(), 1);
    assert_eq!(manifest.permissions.filesystem[0].path, "${TEMP}/**");
    assert!(!manifest.permissions.subprocess);
    assert_eq!(manifest.permissions.level(), PermissionLevel::Restricted);

    // Actions
    assert_eq!(manifest.actions.len(), 3);
    let action_names: Vec<&str> = manifest.actions.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(action_names, vec!["fetch", "fetch_json", "extract_text"]);

    // Build deps
    assert_eq!(manifest.build.dependencies.len(), 2);
}

#[test]
fn test_both_skills_roundtrip() {
    for skill in &["file-ops", "web-fetch"] {
        let path = skills_dir().join(format!("{skill}/skillguard.yaml"));
        let manifest = SkillManifest::from_yaml_file(&path).unwrap();
        let yaml = manifest.to_yaml_string().unwrap();
        let reparsed = SkillManifest::from_yaml_str(&yaml).unwrap();

        assert_eq!(manifest.name, reparsed.name);
        assert_eq!(manifest.version, reparsed.version);
        assert_eq!(manifest.actions.len(), reparsed.actions.len());
        assert_eq!(manifest.permissions.level(), reparsed.permissions.level());
    }
}
