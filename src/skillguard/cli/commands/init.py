"""Initialize a new skill project."""

from pathlib import Path
from typing import Any

from rich.console import Console
from rich.panel import Panel

from skillguard.sdk.manifest import (
    AdapterConfig,
    BuildConfig,
    FilesystemAccess,
    FilesystemPermission,
    Permission,
    SecurityMetadata,
    SkillAction,
    SkillManifest,
)

console = Console()

TEMPLATES: dict[str, dict[str, Any]] = {
    "basic": {
        "description": "A basic skill template",
        "permissions": Permission(),
        "actions": [
            SkillAction(
                name="execute",
                description="Execute the skill",
                parameters={"input": {"type": "string", "description": "Input text"}},
                returns={"type": "string", "description": "Output text"},
            )
        ],
    },
    "api": {
        "description": "A skill that calls external APIs",
        "permissions": Permission(
            network=[],
            environment=[],
        ),
        "actions": [
            SkillAction(
                name="fetch",
                description="Fetch data from API",
                parameters={"endpoint": {"type": "string", "description": "API endpoint"}},
                returns={"type": "object", "description": "API response"},
            )
        ],
    },
    "file-ops": {
        "description": "A skill for file operations",
        "permissions": Permission(
            filesystem=[
                FilesystemPermission(
                    path="${WORKSPACE}/**",
                    access=[FilesystemAccess.READ, FilesystemAccess.WRITE],
                )
            ]
        ),
        "actions": [
            SkillAction(
                name="read",
                description="Read a file",
                parameters={"path": {"type": "string", "description": "File path"}},
                returns={"type": "string", "description": "File contents"},
            ),
            SkillAction(
                name="write",
                description="Write a file",
                parameters={
                    "path": {"type": "string", "description": "File path"},
                    "content": {"type": "string", "description": "Content to write"},
                },
                returns={"type": "boolean", "description": "Success status"},
            ),
        ],
    },
}


SKILL_PY_TEMPLATE = '''"""
{name} - {description}

A SkillGuard skill.
"""

from skillguard.sdk import Skill, SkillContext, SkillResult, SkillManifest


class {class_name}(Skill):
    """Implementation of {name} skill."""

    def execute(self, action: str, context: SkillContext) -> SkillResult:
        """Execute a skill action."""
        if action == "execute":
            return self._execute(context)
        return SkillResult.error(f"Unknown action: {{action}}")

    def _execute(self, context: SkillContext) -> SkillResult:
        """Main execution logic."""
        input_text = context.parameters.get("input", "")

        # TODO: Implement your skill logic here
        result = f"Processed: {{input_text}}"

        return SkillResult.success(result)


def create_skill() -> {class_name}:
    """Factory function to create the skill instance."""
    manifest = SkillManifest.from_yaml("skillguard.yaml")
    return {class_name}(manifest)
'''


def init_skill(name: str, path: Path | None, template: str) -> None:
    """Initialize a new skill project."""
    if template not in TEMPLATES:
        console.print(f"[red]Unknown template: {template}[/red]")
        console.print(f"Available templates: {', '.join(TEMPLATES.keys())}")
        return

    target_dir = (path or Path.cwd()) / name

    if target_dir.exists():
        console.print(f"[red]Directory already exists: {target_dir}[/red]")
        return

    target_dir.mkdir(parents=True)

    tmpl = TEMPLATES[template]

    manifest = SkillManifest(
        name=name,
        version="0.1.0",
        description=tmpl["description"],
        author="your-name",
        permissions=tmpl["permissions"],
        actions=tmpl["actions"],
        adapters=AdapterConfig(
            openclaw=">=2.0",
            langchain=">=0.3",
            mcp=">=1.0",
        ),
        build=BuildConfig(),
        security=SecurityMetadata(),
        keywords=[template],
    )

    manifest.to_yaml(target_dir / "skillguard.yaml")

    class_name = "".join(word.capitalize() for word in name.split("-")) + "Skill"
    skill_code = SKILL_PY_TEMPLATE.format(
        name=name,
        description=tmpl["description"],
        class_name=class_name,
    )

    (target_dir / "skill.py").write_text(skill_code)

    (target_dir / "__init__.py").write_text(f'"""Skill: {name}"""\n\nfrom .skill import create_skill\n')

    (target_dir / "tests").mkdir()
    (target_dir / "tests" / "__init__.py").write_text("")
    (target_dir / "tests" / "test_skill.py").write_text(f'''"""Tests for {name} skill."""

import pytest
from skillguard.sdk import SkillContext, SkillResult, SkillStatus
from pathlib import Path

from {name.replace("-", "_")}.skill import create_skill


@pytest.fixture
def skill():
    """Create skill instance for testing."""
    return create_skill()


@pytest.fixture
def context(tmp_path):
    """Create test context."""
    return SkillContext(
        workspace=tmp_path,
        parameters={{"input": "test"}},
    )


def test_execute_action(skill, context):
    """Test the execute action."""
    result = skill.execute("execute", context)
    assert result.status == SkillStatus.SUCCESS


def test_unknown_action(skill, context):
    """Test handling of unknown actions."""
    result = skill.execute("unknown", context)
    assert result.status == SkillStatus.ERROR
''')

    console.print(
        Panel(
            f"""[green]✓ Created skill: {name}[/green]

[bold]Directory structure:[/bold]
{target_dir}/
├── skillguard.yaml    # Manifest with permissions
├── skill.py           # Skill implementation
├── __init__.py
└── tests/
    └── test_skill.py

[bold]Next steps:[/bold]
1. Edit [cyan]skillguard.yaml[/cyan] to configure permissions
2. Implement your logic in [cyan]skill.py[/cyan]
3. Run [cyan]skillguard build[/cyan] to build
4. Run [cyan]skillguard verify {name}[/cyan] to verify""",
            title="Skill Created",
            border_style="green",
        )
    )
