use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// In-toto v1 statement with SLSA provenance predicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    #[serde(rename = "_type")]
    pub statement_type: String,
    pub subject: Vec<ProvenanceSubject>,
    pub predicate_type: String,
    pub predicate: SlsaPredicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceSubject {
    pub name: String,
    pub digest: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlsaPredicate {
    pub build_type: String,
    pub builder: SlsaBuilder,
    pub invocation: SlsaInvocation,
    pub metadata: SlsaMetadata,
    pub materials: Vec<SlsaMaterial>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlsaBuilder {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlsaInvocation {
    pub config_source: HashMap<String, String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlsaMetadata {
    pub build_started_on: String,
    pub build_finished_on: Option<String>,
    pub reproducible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlsaMaterial {
    pub uri: String,
    pub digest: HashMap<String, String>,
}

impl Provenance {
    /// Create a new provenance attestation for a built artifact.
    pub fn new(
        artifact_name: &str,
        artifact_path: &Path,
        builder_id: &str,
        reproducible: bool,
    ) -> anyhow::Result<Self> {
        let content = std::fs::read(artifact_path)?;
        let digest = Sha256::digest(&content);
        let hex = digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();

        let now = chrono_now();

        Ok(Self {
            statement_type: "https://in-toto.io/Statement/v1".into(),
            subject: vec![ProvenanceSubject {
                name: artifact_name.to_string(),
                digest: HashMap::from([("sha256".into(), hex)]),
            }],
            predicate_type: "https://slsa.dev/provenance/v1".into(),
            predicate: SlsaPredicate {
                build_type: "https://skillguard.dev/build/v1".into(),
                builder: SlsaBuilder {
                    id: builder_id.to_string(),
                },
                invocation: SlsaInvocation {
                    config_source: HashMap::new(),
                    parameters: HashMap::new(),
                },
                metadata: SlsaMetadata {
                    build_started_on: now.clone(),
                    build_finished_on: Some(now),
                    reproducible,
                },
                materials: Vec::new(),
            },
        })
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Write to a file.
    pub fn to_file(&self, path: &Path) -> anyhow::Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

fn chrono_now() -> String {
    // Simple UTC timestamp without chrono dependency
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}Z", duration.as_secs())
}
