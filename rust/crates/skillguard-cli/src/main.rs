mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "skillguard",
    about = "SkillGuard — Security-first marketplace for AI agent skills",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as JSON (machine-readable).
    #[arg(long, global = true)]
    output_json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new skill project.
    Init {
        /// Skill name.
        name: String,
        /// Directory to create the skill in.
        #[arg(short, long)]
        path: Option<String>,
        /// Template to use (basic, api, file-ops).
        #[arg(short, long, default_value = "basic")]
        template: String,
    },
    /// Build a skill package.
    Build {
        /// Path to skill directory.
        #[arg(default_value = ".")]
        path: String,
        /// Sign the package with Sigstore.
        #[arg(short, long)]
        sign: bool,
        /// Output path for the package.
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Verify a skill's signatures and provenance.
    Verify {
        /// Skill name or path.
        skill: String,
        /// Fail on warnings (strict mode).
        #[arg(long)]
        strict: bool,
    },
    /// Run a skill action.
    Run {
        /// Skill name or path.
        skill: String,
        /// Action to execute.
        #[arg(short, long)]
        action: String,
        /// Parameters as JSON string.
        #[arg(short, long)]
        params: Option<String>,
        /// Show what would be done without executing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Audit a skill for security issues.
    Audit {
        /// Path to skill directory.
        #[arg(default_value = ".")]
        path: String,
        /// Automatically fix issues where possible.
        #[arg(long)]
        fix: bool,
    },
    /// Install a skill from the registry.
    Install {
        /// Skill name or URL.
        skill: String,
        /// Force reinstall even if already installed.
        #[arg(short, long)]
        force: bool,
        /// Skip signature verification.
        #[arg(long)]
        skip_verify: bool,
    },
    /// List skills.
    List {
        /// Show only installed skills.
        #[arg(long)]
        installed: bool,
        /// Show all available skills.
        #[arg(long)]
        all: bool,
    },
    /// Show detailed skill information.
    Info {
        /// Skill name or path.
        skill: String,
    },
    /// Search the registry for skills.
    Search {
        /// Search query.
        query: String,
        /// Maximum number of results.
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
    },
    /// Publish a skill to the registry.
    Publish {
        /// Path to skill directory.
        #[arg(default_value = ".")]
        path: String,
        /// Sign the package before publishing.
        #[arg(long, default_value = "true")]
        sign: bool,
    },
    /// Wrap an Anthropic Agent Skill (SKILL.md) with a SkillGuard security sidecar.
    Wrap {
        /// Path to the skill directory containing SKILL.md.
        skill_dir: String,
        /// Output directory for the wrapped skill.
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Export a SkillGuard skill in another format.
    Export {
        /// Export format (anthropic).
        #[arg(long)]
        format: String,
        /// Path to skill directory.
        #[arg(default_value = ".")]
        path: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let cli = Cli::parse();
    let json = cli.output_json;

    match cli.command {
        Commands::Init {
            name,
            path,
            template,
        } => commands::init::run(&name, path.as_deref(), &template, json)?,
        Commands::Build { path, sign, output } => {
            commands::build::run(&path, sign, output.as_deref(), json).await?
        }
        Commands::Verify { skill, strict } => commands::verify::run(&skill, strict, json).await?,
        Commands::Run {
            skill,
            action,
            params,
            dry_run,
        } => commands::run::run(&skill, &action, params.as_deref(), dry_run, json)?,
        Commands::Audit { path, fix } => commands::audit::run(&path, fix, json)?,
        Commands::Install {
            skill,
            force,
            skip_verify,
        } => commands::install::run(&skill, force, skip_verify, json).await?,
        Commands::List { installed, all } => commands::list::run(installed, all, json)?,
        Commands::Info { skill } => commands::info::run(&skill, json)?,
        Commands::Search { query, limit } => commands::search::run(&query, limit, json)?,
        Commands::Publish { path, sign } => commands::publish::run(&path, sign, json).await?,
        Commands::Wrap { skill_dir, output } => {
            commands::wrap::run(&skill_dir, output.as_deref(), json)?
        }
        Commands::Export { format, path } => commands::export::run(&format, &path, json)?,
    }

    Ok(())
}
