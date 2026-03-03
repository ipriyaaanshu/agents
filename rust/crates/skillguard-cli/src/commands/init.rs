use colored::Colorize;
use std::path::PathBuf;

pub fn run(name: &str, path: Option<&str>, template: &str, json: bool) -> anyhow::Result<()> {
    // Validate name
    skillguard_core::SkillName::new(name)?;

    let skill_dir = match path {
        Some(p) => PathBuf::from(p).join(name),
        None => PathBuf::from(name),
    };

    if skill_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", skill_dir.display());
    }

    std::fs::create_dir_all(&skill_dir)?;

    // Generate manifest based on template
    let manifest_content = match template {
        "api" => generate_api_template(name),
        "file-ops" => generate_file_ops_template(name),
        _ => generate_basic_template(name),
    };

    std::fs::write(skill_dir.join("skillguard.yaml"), &manifest_content)?;

    // Create a basic skill.py
    let skill_py = format!(
        r#"""
{name} — A SkillGuard skill.
"""


def execute(action: str, params: dict) -> dict:
    """Main entry point for the skill."""
    return {{"status": "success", "data": f"Action {{action}} executed"}}
"#,
        name = name
    );
    std::fs::write(skill_dir.join("skill.py"), &skill_py)?;

    if json {
        let output = serde_json::json!({
            "status": "created",
            "name": name,
            "path": skill_dir.display().to_string(),
            "template": template,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(
            "{} Created skill '{}' at {}",
            "✓".green().bold(),
            name.cyan(),
            skill_dir.display().to_string().dimmed()
        );
        println!("  Template: {}", template.yellow());
        println!(
            "\n  Next steps:\n    cd {}\n    skillguard audit\n    skillguard build",
            skill_dir.display()
        );
    }

    Ok(())
}

fn generate_basic_template(name: &str) -> String {
    format!(
        r#"name: {name}
version: 0.1.0
description: A new SkillGuard skill
author: your-name

permissions:
  subprocess: false

actions:
  - name: execute
    description: Execute the skill
    parameters:
      input:
        type: string
        description: Input data
    returns:
      type: object
      description: Execution result

keywords: []
"#
    )
}

fn generate_api_template(name: &str) -> String {
    format!(
        r#"name: {name}
version: 0.1.0
description: An API-connected SkillGuard skill
author: your-name

permissions:
  network:
    - domain: api.example.com
      methods: [GET, POST]
      ports: [443]
  environment:
    - name: API_KEY
      required: true
      sensitive: true
  subprocess: false

build:
  dependencies:
    - httpx>=0.27.0

actions:
  - name: fetch
    description: Fetch data from the API
    parameters:
      endpoint:
        type: string
        description: API endpoint path
    returns:
      type: object
      description: API response

keywords: [api, http]
"#
    )
}

fn generate_file_ops_template(name: &str) -> String {
    format!(
        r#"name: {name}
version: 0.1.0
description: A file operations SkillGuard skill
author: your-name

permissions:
  filesystem:
    - path: "${{WORKSPACE}}/**"
      access: [read, write]
  subprocess: false

actions:
  - name: read
    description: Read a file
    parameters:
      path:
        type: string
        description: File path relative to workspace
    returns:
      type: string
      description: File contents

  - name: write
    description: Write to a file
    parameters:
      path:
        type: string
        description: File path relative to workspace
      content:
        type: string
        description: Content to write
    returns:
      type: boolean
      description: Success status

keywords: [file, filesystem]
"#
    )
}
