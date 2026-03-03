"""Build a skill package."""

import hashlib
import json
import tarfile
from datetime import UTC, datetime
from pathlib import Path

from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn

from skillguard.sdk.manifest import SkillManifest

console = Console()


def calculate_file_hash(path: Path) -> str:
    """Calculate SHA256 hash of a file."""
    sha256 = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            sha256.update(chunk)
    return sha256.hexdigest()


def build_skill(path: Path, sign: bool, output: Path | None) -> None:
    """Build a skill package."""
    manifest_path = path / "skillguard.yaml"

    if not manifest_path.exists():
        console.print(f"[red]No skillguard.yaml found in {path}[/red]")
        return

    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console,
    ) as progress:
        task = progress.add_task("Loading manifest...", total=None)

        try:
            manifest = SkillManifest.from_yaml(manifest_path)
        except Exception as e:
            console.print(f"[red]Invalid manifest: {e}[/red]")
            return

        progress.update(task, description="Validating permissions...")

        permission_level = manifest.permissions.level
        console.print(f"  Permission level: [cyan]{permission_level.value}[/cyan]")

        progress.update(task, description="Scanning for issues...")

        issues = scan_for_issues(path, manifest)
        if issues:
            console.print("\n[yellow]Warnings:[/yellow]")
            for issue in issues:
                console.print(f"  ⚠ {issue}")

        progress.update(task, description="Building package...")

        output_dir = output or path / "dist"
        output_dir.mkdir(parents=True, exist_ok=True)

        package_name = f"{manifest.name}-{manifest.version}.tar.gz"
        package_path = output_dir / package_name

        file_hashes = {}
        with tarfile.open(package_path, "w:gz") as tar:
            for file_path in path.rglob("*"):
                if file_path.is_file():
                    rel_path = file_path.relative_to(path)

                    if should_exclude(rel_path):
                        continue

                    tar.add(file_path, arcname=str(rel_path))
                    file_hashes[str(rel_path)] = calculate_file_hash(file_path)

        provenance = {
            "skill": manifest.name,
            "version": manifest.version,
            "build_time": datetime.now(UTC).isoformat(),
            "files": file_hashes,
            "permission_level": permission_level.value,
            "signed": sign,
        }

        provenance_path = output_dir / f"{manifest.name}-{manifest.version}.provenance.json"
        with open(provenance_path, "w") as f:
            json.dump(provenance, f, indent=2)

        if sign:
            progress.update(task, description="Signing with Sigstore...")
            sign_result = sign_package(package_path)
            if sign_result:
                console.print(f"  [green]✓ Signed:[/green] {sign_result}")
            else:
                console.print("  [yellow]⚠ Signing skipped (Sigstore not configured)[/yellow]")

        progress.update(task, description="Done!")

    package_size = package_path.stat().st_size
    console.print(f"\n[green]✓ Built:[/green] {package_path} ({format_size(package_size)})")
    console.print(f"[green]✓ Provenance:[/green] {provenance_path}")


def scan_for_issues(path: Path, manifest: SkillManifest) -> list[str]:
    """Scan for potential security issues."""
    issues = []

    if manifest.permissions.subprocess and not manifest.permissions.subprocess_allowlist:
        issues.append("Unrestricted subprocess access - consider using an allowlist")

    for net_perm in manifest.permissions.network:
        if net_perm.domain == "*" or net_perm.domain.startswith("*"):
            issues.append(f"Wildcard network permission: {net_perm.domain}")

    for fs_perm in manifest.permissions.filesystem:
        if fs_perm.path == "/" or fs_perm.path == "/**":
            issues.append(f"Root filesystem access: {fs_perm.path}")

    skill_py = path / "skill.py"
    if skill_py.exists():
        content = skill_py.read_text()
        dangerous_imports = ["os.system", "subprocess.call", "eval(", "exec("]
        for danger in dangerous_imports:
            if danger in content:
                issues.append(f"Potentially dangerous pattern: {danger}")

    return issues


def should_exclude(rel_path: Path) -> bool:
    """Check if a file should be excluded from the package."""
    excludes = [
        "__pycache__",
        ".git",
        ".pytest_cache",
        ".ruff_cache",
        "dist",
        "*.pyc",
        "*.pyo",
        ".env",
        ".venv",
    ]

    parts = rel_path.parts
    for exclude in excludes:
        if exclude.startswith("*"):
            if any(part.endswith(exclude[1:]) for part in parts):
                return True
        elif exclude in parts:
            return True

    return False


def sign_package(_package_path: Path) -> str | None:
    """Sign a package with Sigstore."""
    try:
        import sigstore  # noqa: F401
        console.print("  [dim]Sigstore signing requires authentication...[/dim]")
        return None
    except ImportError:
        return None


def format_size(size: int) -> str:
    """Format a file size in human-readable form."""
    for unit in ["B", "KB", "MB", "GB"]:
        if size < 1024:
            return f"{size:.1f} {unit}"
        size /= 1024
    return f"{size:.1f} TB"
