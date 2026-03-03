"""Run a skill action in sandbox."""

import json
from pathlib import Path

from rich.console import Console
from rich.panel import Panel
from rich.syntax import Syntax

from skillguard.sdk.manifest import SkillManifest
from skillguard.sdk.sandbox import Sandbox, SandboxConfig
from skillguard.sdk.skill import SkillContext, SkillStatus

console = Console()

SKILLS_DIR = Path.home() / ".skillguard" / "skills"


def run_skill(skill: str, action: str, params: str | None, dry_run: bool) -> None:
    """Run a skill action in sandbox."""
    skill_path = Path(skill)
    if not skill_path.exists():
        skill_path = SKILLS_DIR / skill

    if not skill_path.exists():
        console.print(f"[red]Skill not found: {skill}[/red]")
        console.print(f"[dim]Looked in: {skill_path}[/dim]")
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

    available_actions = [a.name for a in manifest.actions]
    if action not in available_actions:
        console.print(f"[red]Unknown action: {action}[/red]")
        console.print(f"[dim]Available actions: {', '.join(available_actions)}[/dim]")
        return

    parameters = {}
    if params:
        try:
            parameters = json.loads(params)
        except json.JSONDecodeError as e:
            console.print(f"[red]Invalid JSON parameters: {e}[/red]")
            return

    console.print(f"\n[bold]Running skill: {manifest.name}@{manifest.version}[/bold]")
    console.print(f"[cyan]Action:[/cyan] {action}")
    console.print(f"[cyan]Permission level:[/cyan] {manifest.permissions.level.value}")
    if dry_run:
        console.print("[yellow]Mode: DRY RUN[/yellow]")
    console.print()

    sandbox_config = SandboxConfig(
        workspace=Path.cwd(),
        timeout_seconds=30,
    )

    context = SkillContext(
        workspace=Path.cwd(),
        parameters=parameters,
        dry_run=dry_run,
    )

    with Sandbox(manifest, sandbox_config):
        try:
            import importlib.util
            skill_py = skill_path / "skill.py"

            if not skill_py.exists():
                console.print(f"[red]No skill.py found in {skill_path}[/red]")
                return

            spec = importlib.util.spec_from_file_location("skill", skill_py)
            module = importlib.util.module_from_spec(spec)

            import sys
            old_path = sys.path.copy()
            sys.path.insert(0, str(skill_path))

            try:
                spec.loader.exec_module(module)

                if hasattr(module, "create_skill"):
                    skill_instance = module.create_skill()
                else:
                    console.print("[red]Skill has no create_skill() function[/red]")
                    return

                result = skill_instance.execute(action, context)

            finally:
                sys.path = old_path

            if result.status == SkillStatus.SUCCESS:
                console.print(Panel(
                    f"[green]✓ Success[/green]\n\n{format_result(result.data)}",
                    title="Result",
                    border_style="green",
                ))
            elif result.status == SkillStatus.DENIED:
                console.print(Panel(
                    f"[red]✗ Permission Denied[/red]\n\n{result.error_message}",
                    title="Result",
                    border_style="red",
                ))
            else:
                console.print(Panel(
                    f"[red]✗ Error[/red]\n\n{result.error_message}",
                    title="Result",
                    border_style="red",
                ))

        except Exception as e:
            console.print(f"[red]Execution failed: {e}[/red]")
            raise


def format_result(data) -> str:
    """Format result data for display."""
    if data is None:
        return "[dim]No data returned[/dim]"

    if isinstance(data, (dict, list)):
        return Syntax(
            json.dumps(data, indent=2),
            "json",
            theme="monokai",
            word_wrap=True,
        ).__rich__()

    return str(data)
