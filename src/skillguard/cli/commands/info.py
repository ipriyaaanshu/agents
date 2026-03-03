"""Show detailed information about a skill."""

from pathlib import Path

from rich.console import Console
from rich.panel import Panel
from rich.table import Table

from skillguard.sdk.manifest import SkillManifest

console = Console()

SKILLS_DIR = Path.home() / ".skillguard" / "skills"


def show_skill_info(skill: str) -> None:
    """Show detailed information about a skill."""
    skill_path = Path(skill)
    if not skill_path.exists():
        skill_path = SKILLS_DIR / skill

    if not skill_path.exists():
        console.print(f"[red]Skill not found: {skill}[/red]")
        return

    manifest_path = skill_path / "skillguard.yaml"
    if not manifest_path.exists():
        console.print(f"[red]No skillguard.yaml found in {skill_path}[/red]")
        return

    try:
        manifest = SkillManifest.from_yaml(manifest_path)
    except Exception as e:
        console.print(f"[red]Invalid manifest: {e}[/red]")
        return

    console.print(Panel(
        f"""[bold cyan]{manifest.name}[/bold cyan] [dim]v{manifest.version}[/dim]

{manifest.description}

[bold]Author:[/bold] {manifest.author}
[bold]License:[/bold] {manifest.license}
[bold]Permission Level:[/bold] {manifest.permissions.level.value}""",
        title="Skill Information",
    ))

    if manifest.actions:
        actions_table = Table(title="Actions")
        actions_table.add_column("Name", style="cyan")
        actions_table.add_column("Description")

        for action in manifest.actions:
            actions_table.add_row(action.name, action.description)

        console.print(actions_table)

    console.print("\n[bold]Permissions[/bold]")

    if manifest.permissions.network:
        console.print("\n[cyan]Network:[/cyan]")
        for net_perm in manifest.permissions.network:
            methods = ", ".join(m.value for m in net_perm.methods)
            console.print(f"  • {net_perm.domain} [{methods}]")
    else:
        console.print("\n[cyan]Network:[/cyan] [dim]None[/dim]")

    if manifest.permissions.filesystem:
        console.print("\n[cyan]Filesystem:[/cyan]")
        for fs_perm in manifest.permissions.filesystem:
            access = ", ".join(a.value for a in fs_perm.access)
            console.print(f"  • {fs_perm.path} [{access}]")
    else:
        console.print("\n[cyan]Filesystem:[/cyan] [dim]None[/dim]")

    if manifest.permissions.environment:
        console.print("\n[cyan]Environment:[/cyan]")
        for env_perm in manifest.permissions.environment:
            flags: list[str] = []
            if env_perm.required:
                flags.append("required")
            if env_perm.sensitive:
                flags.append("sensitive")
            flag_str = f" ({', '.join(flags)})" if flags else ""
            console.print(f"  • {env_perm.name}{flag_str}")
    else:
        console.print("\n[cyan]Environment:[/cyan] [dim]None[/dim]")

    subprocess_status = "Allowed" if manifest.permissions.subprocess else "Denied"
    if manifest.permissions.subprocess and manifest.permissions.subprocess_allowlist:
        subprocess_status = f"Restricted to: {', '.join(manifest.permissions.subprocess_allowlist)}"
    console.print(f"\n[cyan]Subprocess:[/cyan] {subprocess_status}")

    if manifest.adapters:
        console.print("\n[bold]Framework Compatibility[/bold]")
        adapters = manifest.adapters.model_dump(exclude_none=True)
        for framework, version in adapters.items():
            console.print(f"  • {framework}: {version}")

    if manifest.security.slsa_level > 0:
        console.print("\n[bold]Security[/bold]")
        console.print(f"  SLSA Level: {manifest.security.slsa_level}")
        if manifest.security.audit_date:
            console.print(f"  Last Audit: {manifest.security.audit_date}")
        if manifest.security.auditor:
            console.print(f"  Auditor: {manifest.security.auditor}")

    console.print(f"\n[dim]Location: {skill_path}[/dim]")
