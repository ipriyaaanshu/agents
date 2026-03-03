"""LangChain adapter for SkillGuard skills."""

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
    if spec is None or spec.loader is None:
        raise ImportError(f"Failed to load skill module: {skill_py}")

    module = importlib.util.module_from_spec(spec)

    old_path = sys.path.copy()
    sys.path.insert(0, str(skill_path))

    try:
        spec.loader.exec_module(module)
        if hasattr(module, "create_skill"):
            skill = module.create_skill()
            return skill  # type: ignore[no-any-return]
        raise AttributeError("Skill has no create_skill() function")
    finally:
        sys.path = old_path


class LangChainAdapter(SkillAdapter):
    """Adapter to use SkillGuard skills with LangChain."""

    @classmethod
    def load(cls, skill_name: str) -> "LangChainAdapter":
        """Load a skill by name."""
        skill_path = Path(skill_name)
        if not skill_path.exists():
            skill_path = SKILLS_DIR / skill_name

        if not skill_path.exists():
            raise FileNotFoundError(f"Skill not found: {skill_name}")

        skill = _load_skill_from_path(skill_path)
        return cls(skill)

    def as_tool(self, action: str | None = None) -> Any:
        """Convert to a LangChain Tool.

        Args:
            action: Specific action to wrap. If None, uses first action.

        Returns:
            A LangChain Tool instance.
        """
        try:
            from langchain_core.tools import Tool
        except ImportError as e:
            raise ImportError(
                "LangChain not installed. Install with: pip install langchain-core"
            ) from e

        if action is None:
            if not self.manifest.actions:
                raise ValueError(f"Skill {self.name} has no actions")
            action = self.manifest.actions[0].name

        action_def = next(
            (a for a in self.manifest.actions if a.name == action),
            None
        )
        if action_def is None:
            raise ValueError(f"Action {action} not found in skill {self.name}")

        skill = self.skill

        def run_skill(**kwargs: Any) -> str:
            context = SkillContext(
                workspace=Path.cwd(),
                parameters=kwargs,
            )
            result = skill.execute(action, context)

            if result.status == SkillStatus.SUCCESS:
                return str(result.data)
            return f"Error: {result.error_message}"

        return Tool(
            name=f"{self.name}_{action}",
            description=action_def.description,
            func=run_skill,
        )

    def as_tools(self) -> list[Any]:
        """Convert all actions to LangChain Tools."""
        return [
            self.as_tool(action.name)
            for action in self.manifest.actions
        ]


def as_tool(skill_name: str, action: str | None = None) -> Any:
    """Convenience function to load a skill as a LangChain tool."""
    adapter = LangChainAdapter.load(skill_name)
    return adapter.as_tool(action)


def as_tools(skill_name: str) -> list[Any]:
    """Convenience function to load all skill actions as LangChain tools."""
    adapter = LangChainAdapter.load(skill_name)
    return adapter.as_tools()
