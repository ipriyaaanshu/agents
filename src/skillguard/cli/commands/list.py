"""List installed skills."""

from pathlib import Path

from rich.console import Console
from rich.table import Table

from skillguard.sdk.manifest import SkillManifest

console = Console()

SKILLS_DIR = Path.home() / ".skillguard" / "skills"


def list_installed_skills(_installed_only: bool) -> None:
    """List installed skills."""
    console.print("\n[bold]Installed Skills[/bold]\n")

    if not SKILLS_DIR.exists():
        console.print("[yellow]No skills installed yet[/yellow]")
        console.print(f"[dim]Skills directory: {SKILLS_DIR}[/dim]")
        console.print("\n[dim]Install skills with: skillguard install <name>[/dim]")
        return

    skills = []
    for skill_dir in SKILLS_DIR.iterdir():
        if not skill_dir.is_dir():
            continue

        manifest_path = skill_dir / "skillguard.yaml"
        if manifest_path.exists():
            try:
                manifest = SkillManifest.from_yaml(manifest_path)
                skills.append({
                    "name": manifest.name,
                    "version": manifest.version,
                    "description": manifest.description,
                    "permission_level": manifest.permissions.level.value,
                    "actions": len(manifest.actions),
                    "path": skill_dir,
                })
            except Exception:
                skills.append({
                    "name": skill_dir.name,
                    "version": "?",
                    "description": "[invalid manifest]",
                    "permission_level": "?",
                    "actions": 0,
                    "path": skill_dir,
                })

    if not skills:
        console.print("[yellow]No skills installed yet[/yellow]")
        console.print("\n[dim]Install skills with: skillguard install <name>[/dim]")
        return

    table = Table()
    table.add_column("Name", style="cyan")
    table.add_column("Version")
    table.add_column("Description")
    table.add_column("Permissions")
    table.add_column("Actions", justify="right")

    for skill in sorted(skills, key=lambda x: x["name"]):
        table.add_row(
            skill["name"],
            skill["version"],
            skill["description"][:50] + "..." if len(skill["description"]) > 50 else skill["description"],
            skill["permission_level"],
            str(skill["actions"]),
        )

    console.print(table)
    console.print(f"\n[dim]Skills directory: {SKILLS_DIR}[/dim]")
