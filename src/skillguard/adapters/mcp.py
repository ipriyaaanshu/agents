"""MCP (Model Context Protocol) adapter for SkillGuard skills."""

from pathlib import Path
from typing import Any

from skillguard.adapters.base import SkillAdapter
from skillguard.sdk.skill import Skill, SkillContext, SkillStatus

SKILLS_DIR = Path.home() / ".skillguard" / "skills"


def _load_skill_from_path(skill_path: Path) -> Skill:
    """Load a skill instance from a path."""
    import importlib.util
    import sys

    manifest_path = skill_path / "skillguard.yaml"
    skill_py = skill_path / "skill.py"

    if not manifest_path.exists():
        raise FileNotFoundError(f"No skillguard.yaml in {skill_path}")
    if not skill_py.exists():
        raise FileNotFoundError(f"No skill.py in {skill_path}")

    spec = importlib.util.spec_from_file_location("skill", skill_py)
    module = importlib.util.module_from_spec(spec)

    old_path = sys.path.copy()
    sys.path.insert(0, str(skill_path))

    try:
        spec.loader.exec_module(module)
        if hasattr(module, "create_skill"):
            return module.create_skill()
        raise AttributeError("Skill has no create_skill() function")
    finally:
        sys.path = old_path


class MCPAdapter(SkillAdapter):
    """Adapter to expose SkillGuard skills as MCP tools."""

    @classmethod
    def load(cls, skill_name: str) -> "MCPAdapter":
        """Load a skill by name."""
        skill_path = Path(skill_name)
        if not skill_path.exists():
            skill_path = SKILLS_DIR / skill_name

        if not skill_path.exists():
            raise FileNotFoundError(f"Skill not found: {skill_name}")

        skill = _load_skill_from_path(skill_path)
        return cls(skill)

    def as_tool(self) -> dict[str, Any]:
        """Convert to MCP tool definition format."""
        tools = []

        for action in self.manifest.actions:
            properties = {}
            required = []

            for param_name, param_schema in action.parameters.items():
                properties[param_name] = {
                    "type": param_schema.get("type", "string"),
                    "description": param_schema.get("description", ""),
                }
                if param_schema.get("required", False):
                    required.append(param_name)

            tools.append({
                "name": f"{self.name}_{action.name}",
                "description": action.description,
                "inputSchema": {
                    "type": "object",
                    "properties": properties,
                    "required": required,
                },
            })

        return tools

    def handle_call(self, tool_name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        """Handle an MCP tool call.

        Args:
            tool_name: The tool name (format: {skill_name}_{action})
            arguments: The arguments passed to the tool

        Returns:
            MCP-formatted response
        """
        prefix = f"{self.name}_"
        if not tool_name.startswith(prefix):
            return {
                "isError": True,
                "content": [{"type": "text", "text": f"Unknown tool: {tool_name}"}],
            }

        action = tool_name[len(prefix):]

        context = SkillContext(
            workspace=Path.cwd(),
            parameters=arguments,
        )

        result = self.skill.execute(action, context)

        if result.status == SkillStatus.SUCCESS:
            return {
                "content": [{"type": "text", "text": str(result.data)}],
            }
        else:
            return {
                "isError": True,
                "content": [{"type": "text", "text": f"Error: {result.error_message}"}],
            }

    def as_server_config(self) -> dict[str, Any]:
        """Generate MCP server configuration."""
        return {
            "name": f"skillguard-{self.name}",
            "version": self.manifest.version,
            "tools": self.as_tool(),
        }


def as_server(skill_name: str) -> MCPAdapter:
    """Load a skill as an MCP server adapter."""
    return MCPAdapter.load(skill_name)
