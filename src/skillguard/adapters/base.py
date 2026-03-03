"""Base adapter interface for framework compatibility."""

from abc import ABC, abstractmethod
from typing import Any

from skillguard.sdk.skill import Skill


class SkillAdapter(ABC):
    """Base class for framework adapters."""

    def __init__(self, skill: Skill) -> None:
        self.skill = skill
        self.manifest = skill.manifest

    @classmethod
    @abstractmethod
    def load(cls, skill_name: str) -> "SkillAdapter":
        """Load a skill by name and wrap it for the target framework."""
        pass

    @abstractmethod
    def as_tool(self) -> Any:
        """Convert the skill to the framework's tool format."""
        pass

    @property
    def name(self) -> str:
        return self.manifest.name

    @property
    def description(self) -> str:
        return self.manifest.description
