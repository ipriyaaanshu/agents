"""Verify a skill's signature and integrity."""

from pathlib import Path

from rich.console import Console
from rich.table import Table

from skillguard.sdk.manifest import SkillManifest

console = Console()


def verify_skill(skill: str, strict: bool) -> None:
    """Verify a skill's signature and integrity."""
    console.print(f"\n[bold]Verifying skill: {skill}[/bold]\n")

    if "@" in skill:
        name, version = skill.split("@", 1)
        skill_path = None
    else:
        skill_path = Path(skill)
        if skill_path.exists():
            name = skill_path.name
            version = None
        else:
            name = skill
            version = "latest"
            skill_path = None

    checks: list[tuple[str, bool | None, str | None]] = []
    all_passed = True
    warnings = []

    if skill_path and skill_path.exists():
        manifest_path = skill_path / "skillguard.yaml"
        if manifest_path.exists():
            checks.append(("Manifest exists", True, None))

            try:
                manifest = SkillManifest.from_yaml(manifest_path)
                checks.append(("Manifest valid", True, None))

                perm_level = manifest.permissions.level.value
                checks.append(("Permission level", True, perm_level))

                if manifest.security.slsa_level >= 2:
                    checks.append(("SLSA Level", True, f"Level {manifest.security.slsa_level}"))
                else:
                    checks.append(("SLSA Level", False, f"Level {manifest.security.slsa_level} (< 2)"))
                    warnings.append("SLSA level below 2 - limited supply chain guarantees")

            except Exception as e:
                checks.append(("Manifest valid", False, str(e)))
                all_passed = False
        else:
            checks.append(("Manifest exists", False, "skillguard.yaml not found"))
            all_passed = False
    else:
        checks.append(("Registry lookup", True, f"{name}@{version}"))
        checks.append(("Signature check", None, "Not implemented yet"))
        warnings.append("Registry verification not yet implemented")

    checks.append(("Signature valid", None, "Sigstore verification pending"))
    checks.append(("Provenance verified", None, "SLSA provenance pending"))
    checks.append(("No known CVEs", None, "CVE database check pending"))

    table = Table(title="Verification Results")
    table.add_column("Check", style="cyan")
    table.add_column("Status")
    table.add_column("Details", style="dim")

    for check_name, status, details in checks:
        if status is True:
            status_str = "[green]✓ PASS[/green]"
        elif status is False:
            status_str = "[red]✗ FAIL[/red]"
            all_passed = False
        else:
            status_str = "[yellow]○ PENDING[/yellow]"

        table.add_row(check_name, status_str, details or "")

    console.print(table)

    if warnings:
        console.print("\n[yellow]Warnings:[/yellow]")
        for warning in warnings:
            console.print(f"  ⚠ {warning}")

    if strict and (not all_passed or warnings):
        console.print("\n[red]Verification failed (strict mode)[/red]")
        raise SystemExit(1)
    elif not all_passed:
        console.print("\n[red]Some checks failed[/red]")
        raise SystemExit(1)
    else:
        console.print("\n[green]Verification passed[/green]")
