"""Base skill class and execution context."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import StrEnum
from pathlib import Path
from typing import Any, Generic, TypeVar

from loguru import logger

from skillguard.sdk.manifest import SkillManifest


class SkillStatus(StrEnum):
    """Skill execution status."""
    SUCCESS = "success"
    ERROR = "error"
    DENIED = "denied"
    TIMEOUT = "timeout"


@dataclass
class SkillContext:
    """Execution context passed to skills."""

    workspace: Path
    """The workspace directory the skill can access."""

    environment: dict[str, str] = field(default_factory=dict)
    """Environment variables available to the skill."""

    parameters: dict[str, Any] = field(default_factory=dict)
    """Parameters passed to the skill action."""

    timeout_seconds: int = 30
    """Maximum execution time."""

    dry_run: bool = False
    """If True, skill should not make actual changes."""

    trace_id: str | None = None
    """Trace ID for logging and monitoring."""


@dataclass
class SkillResult:
    """Result from skill execution."""

    status: SkillStatus
    """Execution status."""

    data: Any = None
    """Result data (if successful)."""

    error_message: str | None = None
    """Error message (if failed)."""

    metadata: dict[str, Any] = field(default_factory=dict)
    """Additional metadata (timing, resource usage, etc.)."""

    @classmethod
    def success(cls, data: Any = None, **metadata: Any) -> SkillResult:
        """Create a successful result."""
        return cls(status=SkillStatus.SUCCESS, data=data, metadata=metadata)

    @classmethod
    def error(cls, message: str, **metadata: Any) -> SkillResult:
        """Create an error result."""
        return cls(status=SkillStatus.ERROR, error_message=message, metadata=metadata)

    @classmethod
    def denied(cls, reason: str) -> SkillResult:
        """Create a permission denied result."""
        return cls(status=SkillStatus.DENIED, error_message=f"Permission denied: {reason}")

    @classmethod
    def timeout(cls, seconds: int) -> SkillResult:
        """Create a timeout result."""
        return cls(status=SkillStatus.TIMEOUT, error_message=f"Execution timed out after {seconds}s")


T = TypeVar("T")


class Skill(ABC, Generic[T]):
    """Base class for all SkillGuard skills."""

    def __init__(self, manifest: SkillManifest) -> None:
        self.manifest = manifest
        self._logger = logger.bind(skill=manifest.name)

    @property
    def name(self) -> str:
        """Skill name."""
        return self.manifest.name

    @property
    def version(self) -> str:
        """Skill version."""
        return self.manifest.version

    @abstractmethod
    def execute(self, action: str, context: SkillContext) -> SkillResult:
        """Execute a skill action.

        Args:
            action: The action to execute (must be defined in manifest.actions)
            context: Execution context with parameters and environment

        Returns:
            SkillResult with status and data/error
        """
        pass

    def get_actions(self) -> list[str]:
        """Get list of available actions."""
        return [a.name for a in self.manifest.actions]

    def validate_action(self, action: str) -> bool:
        """Check if an action is valid for this skill."""
        return action in self.get_actions()

    def __repr__(self) -> str:
        return f"Skill({self.name}@{self.version})"


class FunctionSkill(Skill[T]):
    """A skill implemented as a collection of Python functions."""

    def __init__(self, manifest: SkillManifest) -> None:
        super().__init__(manifest)
        self._actions: dict[str, callable] = {}

    def register_action(self, name: str):
        """Decorator to register an action handler."""
        def decorator(func: callable) -> callable:
            self._actions[name] = func
            return func
        return decorator

    def execute(self, action: str, context: SkillContext) -> SkillResult:
        """Execute a registered action."""
        if action not in self._actions:
            return SkillResult.error(f"Unknown action: {action}")

        try:
            self._logger.info(f"Executing action: {action}", trace_id=context.trace_id)
            result = self._actions[action](context)

            if isinstance(result, SkillResult):
                return result
            return SkillResult.success(result)

        except PermissionError as e:
            self._logger.warning(f"Permission denied: {e}")
            return SkillResult.denied(str(e))
        except TimeoutError as e:
            self._logger.warning(f"Timeout: {e}")
            return SkillResult.timeout(context.timeout_seconds)
        except Exception as e:
            self._logger.exception(f"Action failed: {e}")
            return SkillResult.error(str(e))
