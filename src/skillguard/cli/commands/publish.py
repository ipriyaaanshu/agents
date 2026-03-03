"""Publish a skill to the registry."""

from pathlib import Path

from rich.console import Console
from rich.panel import Panel

from skillguard.sdk.manifest import SkillManifest

console = Console()


def publish_skill(path: Path, _sign: bool) -> None:
    """Publish a skill to the registry."""
    manifest_path = path / "skillguard.yaml"

    if not manifest_path.exists():
        console.print(f"[red]No skillguard.yaml found in {path}[/red]")
        return

    try:
        manifest = SkillManifest.from_yaml(manifest_path)
    except Exception as e:
        console.print(f"[red]Invalid manifest: {e}[/red]")
        return

    console.print(f"\n[bold]Publishing: {manifest.name}@{manifest.version}[/bold]\n")

    console.print("[cyan]Pre-publish checks:[/cyan]")

    checks = [
        ("Manifest valid", True),
        ("Version format", True),
        ("Author specified", bool(manifest.author and manifest.author != "your-name")),
        ("Description provided", bool(manifest.description)),
        ("At least one action", len(manifest.actions) > 0),
    ]

    all_passed = True
    for check_name, passed in checks:
        status = "[green]✓[/green]" if passed else "[red]✗[/red]"
        console.print(f"  {status} {check_name}")
        if not passed:
            all_passed = False

    if not all_passed:
        console.print("\n[red]Pre-publish checks failed. Please fix the issues above.[/red]")
        return

    console.print(Panel(
        """[yellow]Registry publishing is not yet implemented.[/yellow]

To share your skill now, you can:

1. [bold]Push to GitHub[/bold]
   Users can install directly from your repo

2. [bold]Create a release[/bold]
   Tag your version and create a GitHub release

3. [bold]Submit for inclusion[/bold]
   Open a PR to skillguard/registry to be listed

[dim]Full registry support coming in Phase 2[/dim]""",
        title="Publishing",
        border_style="yellow",
    ))
