"""SkillGuard CLI - Main entry point."""

from pathlib import Path

import typer
from rich.console import Console

from skillguard import __version__

app = typer.Typer(
    name="skillguard",
    help="Chainguard for AI Agent Skills - Secure, signed, and verified skills.",
    no_args_is_help=True,
)

console = Console()


def version_callback(value: bool) -> None:
    if value:
        console.print(f"[bold blue]SkillGuard[/bold blue] version {__version__}")
        raise typer.Exit()


@app.callback()
def main(
    version: bool | None = typer.Option(
        None,
        "--version",
        "-v",
        help="Show version and exit.",
        callback=version_callback,
        is_eager=True,
    ),
) -> None:
    """SkillGuard - Secure AI agent skills."""
    pass


@app.command()
def init(
    name: str = typer.Argument(..., help="Name of the skill to create"),
    path: Path | None = typer.Option(
        None,
        "--path",
        "-p",
        help="Directory to create the skill in (defaults to current directory)",
    ),
    template: str = typer.Option(
        "basic",
        "--template",
        "-t",
        help="Template to use (basic, api, file-ops)",
    ),
) -> None:
    """Initialize a new skill project."""
    from skillguard.cli.commands.init import init_skill
    init_skill(name, path, template)


@app.command()
def build(
    path: Path = typer.Argument(
        Path("."),
        help="Path to the skill directory",
    ),
    sign: bool = typer.Option(
        False,
        "--sign",
        "-s",
        help="Sign the built skill with Sigstore",
    ),
    output: Path | None = typer.Option(
        None,
        "--output",
        "-o",
        help="Output directory for the built skill",
    ),
) -> None:
    """Build a skill package."""
    from skillguard.cli.commands.build import build_skill
    build_skill(path, sign, output)


@app.command()
def verify(
    skill: str = typer.Argument(..., help="Skill to verify (name@version or path)"),
    strict: bool = typer.Option(
        False,
        "--strict",
        help="Fail on any warnings",
    ),
) -> None:
    """Verify a skill's signature and integrity."""
    from skillguard.cli.commands.verify import verify_skill
    verify_skill(skill, strict)


@app.command()
def install(
    skill: str = typer.Argument(..., help="Skill to install (name@version)"),
    force: bool = typer.Option(
        False,
        "--force",
        "-f",
        help="Force reinstall even if already installed",
    ),
    skip_verify: bool = typer.Option(
        False,
        "--skip-verify",
        help="Skip signature verification (NOT recommended)",
    ),
) -> None:
    """Install a verified skill."""
    from skillguard.cli.commands.install import install_skill
    install_skill(skill, force, skip_verify)


@app.command()
def search(
    query: str = typer.Argument(..., help="Search query"),
    limit: int = typer.Option(
        10,
        "--limit",
        "-n",
        help="Maximum number of results",
    ),
) -> None:
    """Search for skills in the registry."""
    from skillguard.cli.commands.search import search_skills
    search_skills(query, limit)


@app.command()
def audit(
    path: Path = typer.Argument(
        Path("."),
        help="Path to audit (skill directory or workspace)",
    ),
    fix: bool = typer.Option(
        False,
        "--fix",
        help="Attempt to fix issues automatically",
    ),
) -> None:
    """Audit installed skills for vulnerabilities."""
    from skillguard.cli.commands.audit import audit_skills
    audit_skills(path, fix)


@app.command()
def run(
    skill: str = typer.Argument(..., help="Skill to run"),
    action: str = typer.Option(..., "--action", "-a", help="Action to execute"),
    params: str | None = typer.Option(
        None,
        "--params",
        "-p",
        help="JSON parameters for the action",
    ),
    dry_run: bool = typer.Option(
        False,
        "--dry-run",
        help="Run without making actual changes",
    ),
) -> None:
    """Run a skill action in sandbox."""
    from skillguard.cli.commands.run import run_skill
    run_skill(skill, action, params, dry_run)


@app.command()
def publish(
    path: Path = typer.Argument(
        Path("."),
        help="Path to the skill to publish",
    ),
    sign: bool = typer.Option(
        True,
        "--sign/--no-sign",
        help="Sign the skill before publishing",
    ),
) -> None:
    """Publish a skill to the registry."""
    from skillguard.cli.commands.publish import publish_skill
    publish_skill(path, sign)


@app.command("list")
def list_skills(
    installed: bool = typer.Option(
        True,
        "--installed/--all",
        help="List only installed skills",
    ),
) -> None:
    """List skills."""
    from skillguard.cli.commands.list import list_installed_skills
    list_installed_skills(installed)


@app.command()
def info(
    skill: str = typer.Argument(..., help="Skill name or path"),
) -> None:
    """Show detailed information about a skill."""
    from skillguard.cli.commands.info import show_skill_info
    show_skill_info(skill)


if __name__ == "__main__":
    app()
