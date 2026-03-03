use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillGuardError {
    #[error("Invalid skill name '{name}': {reason}")]
    InvalidSkillName { name: String, reason: String },

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Failed to parse YAML: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid version: {0}")]
    InvalidVersion(#[from] semver::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Action not found: {0}")]
    ActionNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),

    #[error("Build error: {0}")]
    BuildError(String),

    #[error("Signing error: {0}")]
    SigningError(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("Anthropic skill parse error: {0}")]
    AnthropicParseError(String),
}

pub type Result<T> = std::result::Result<T, SkillGuardError>;
