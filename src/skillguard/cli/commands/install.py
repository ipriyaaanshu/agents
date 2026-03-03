"""Install a verified skill."""

from pathlib import Path

from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn

console = Console()

SKILLS_DIR = Path.home() / ".skillguard" / "skills"


def install_skill(skill: str, force: bool, skip_verify: bool) -> None:
    """Install a verified skill."""
    if "@" in skill:
        name, version = skill.split("@", 1)
    else:
        name = skill
        version = "latest"

    console.print(f"\n[bold]Installing skill: {name}@{version}[/bold]\n")

    if skip_verify:
        console.print("[yellow]⚠ Skipping verification - this is NOT recommended![/yellow]\n")

    skill_dir = SKILLS_DIR / name

    if skill_dir.exists() and not force:
        console.print(f"[yellow]Skill already installed: {skill_dir}[/yellow]")
        console.print("Use --force to reinstall")
        return

    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console,
    ) as progress:
        task = progress.add_task("Resolving skill...", total=None)

        progress.update(task, description="Fetching from registry...")

        console.print("\n[yellow]Registry not yet implemented[/yellow]")
        console.print("To install a local skill, use:")
        console.print(f"  [cyan]cp -r /path/to/skill {SKILLS_DIR}/{name}[/cyan]")

        return

        if not skip_verify:
            progress.update(task, description="Verifying signature...")

        progress.update(task, description="Installing...")

        SKILLS_DIR.mkdir(parents=True, exist_ok=True)

        progress.update(task, description="Done!")

    console.print(f"\n[green]✓ Installed:[/green] {name}@{version}")
    console.print(f"[dim]Location: {skill_dir}[/dim]")
