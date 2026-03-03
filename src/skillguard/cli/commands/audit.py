"""Audit installed skills for vulnerabilities."""

from pathlib import Path

from rich.console import Console
from rich.table import Table

from skillguard.sdk.manifest import SkillManifest

console = Console()

SKILLS_DIR = Path.home() / ".skillguard" / "skills"


def audit_skills(path: Path, _fix: bool) -> None:
    """Audit installed skills for vulnerabilities."""
    console.print("\n[bold]Security Audit[/bold]\n")

    if path.name == "skillguard.yaml" or (path / "skillguard.yaml").exists():
        audit_single_skill(path if path.is_dir() else path.parent)
    else:
        audit_installed_skills()


def audit_single_skill(skill_path: Path) -> None:
    """Audit a single skill."""
    manifest_path = skill_path / "skillguard.yaml"

    if not manifest_path.exists():
        console.print(f"[red]No skillguard.yaml found in {skill_path}[/red]")
        return

    try:
        manifest = SkillManifest.from_yaml(manifest_path)
    except Exception as e:
        console.print(f"[red]Invalid manifest: {e}[/red]")
        return

    console.print(f"[cyan]Skill:[/cyan] {manifest.name}@{manifest.version}")
    console.print(f"[cyan]Permission Level:[/cyan] {manifest.permissions.level.value}\n")

    issues = []

    if manifest.permissions.subprocess and not manifest.permissions.subprocess_allowlist:
        issues.append({
            "severity": "HIGH",
            "type": "permission",
            "message": "Unrestricted subprocess execution allowed",
            "fix": "Add subprocess_allowlist to restrict commands",
        })

    for net_perm in manifest.permissions.network:
        if net_perm.domain.startswith("*"):
            issues.append({
                "severity": "MEDIUM",
                "type": "permission",
                "message": f"Wildcard network access: {net_perm.domain}",
                "fix": "Specify exact domains instead of wildcards",
            })

    for fs_perm in manifest.permissions.filesystem:
        if fs_perm.path in ["/", "/**", "/*"]:
            issues.append({
                "severity": "CRITICAL",
                "type": "permission",
                "message": f"Root filesystem access: {fs_perm.path}",
                "fix": "Restrict to specific directories like ${WORKSPACE}/**",
            })

    if manifest.security.slsa_level < 2:
        issues.append({
            "severity": "LOW",
            "type": "supply-chain",
            "message": f"SLSA level {manifest.security.slsa_level} provides limited guarantees",
            "fix": "Build with reproducible builds and signing for SLSA level 3",
        })

    skill_py = skill_path / "skill.py"
    if skill_py.exists():
        content = skill_py.read_text()
        dangerous = [
            ("eval(", "CRITICAL", "Use of eval() can execute arbitrary code"),
            ("exec(", "CRITICAL", "Use of exec() can execute arbitrary code"),
            ("os.system(", "HIGH", "os.system() bypasses sandbox"),
            ("subprocess.call(", "MEDIUM", "Direct subprocess call may bypass sandbox"),
            ("__import__", "MEDIUM", "Dynamic imports can load arbitrary modules"),
        ]
        for pattern, severity, message in dangerous:
            if pattern in content:
                issues.append({
                    "severity": severity,
                    "type": "code",
                    "message": message,
                    "fix": f"Remove or replace {pattern}",
                })

    if not issues:
        console.print("[green]✓ No security issues found[/green]")
        return

    table = Table(title="Security Issues")
    table.add_column("Severity", style="bold")
    table.add_column("Type")
    table.add_column("Issue")
    table.add_column("Fix")

    severity_colors = {
        "CRITICAL": "red",
        "HIGH": "red",
        "MEDIUM": "yellow",
        "LOW": "dim",
    }

    for issue in sorted(issues, key=lambda x: ["CRITICAL", "HIGH", "MEDIUM", "LOW"].index(x["severity"])):
        color = severity_colors[issue["severity"]]
        table.add_row(
            f"[{color}]{issue['severity']}[/{color}]",
            issue["type"],
            issue["message"],
            issue["fix"],
        )

    console.print(table)

    critical_count = sum(1 for i in issues if i["severity"] == "CRITICAL")
    high_count = sum(1 for i in issues if i["severity"] == "HIGH")

    if critical_count > 0:
        console.print(f"\n[red]✗ {critical_count} CRITICAL issues found - DO NOT USE this skill[/red]")
    elif high_count > 0:
        console.print(f"\n[yellow]⚠ {high_count} HIGH severity issues found[/yellow]")


def audit_installed_skills() -> None:
    """Audit all installed skills."""
    if not SKILLS_DIR.exists():
        console.print("[yellow]No skills installed yet[/yellow]")
        console.print(f"[dim]Skills directory: {SKILLS_DIR}[/dim]")
        return

    skills = list(SKILLS_DIR.iterdir())

    if not skills:
        console.print("[yellow]No skills installed yet[/yellow]")
        return

    console.print(f"[dim]Auditing {len(skills)} installed skills...[/dim]\n")

    for skill_dir in skills:
        if skill_dir.is_dir():
            console.print(f"[bold]─── {skill_dir.name} ───[/bold]")
            audit_single_skill(skill_dir)
            console.print()
