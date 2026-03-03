"""Tests for skill execution."""

import tempfile
from pathlib import Path

import pytest

from skillguard.sdk.manifest import SkillAction, SkillManifest
from skillguard.sdk.skill import (
    FunctionSkill,
    SkillContext,
    SkillResult,
    SkillStatus,
)


@pytest.fixture
def workspace():
    """Create a temporary workspace."""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def basic_manifest():
    """Create a basic manifest for testing."""
    return SkillManifest(
        name="test-skill",
        version="1.0.0",
        description="Test skill",
        author="test",
        actions=[
            SkillAction(
                name="greet",
                description="Greet someone",
                parameters={"name": {"type": "string"}},
            ),
            SkillAction(
                name="add",
                description="Add two numbers",
                parameters={
                    "a": {"type": "number"},
                    "b": {"type": "number"},
                },
            ),
        ],
    )


class TestSkillResult:
    """Tests for SkillResult."""

    def test_success_result(self):
        """Test creating a success result."""
        result = SkillResult.success("hello", extra="metadata")

        assert result.status == SkillStatus.SUCCESS
        assert result.data == "hello"
        assert result.error_message is None
        assert result.metadata["extra"] == "metadata"

    def test_error_result(self):
        """Test creating an error result."""
        result = SkillResult.error("something went wrong")

        assert result.status == SkillStatus.ERROR
        assert result.data is None
        assert result.error_message == "something went wrong"

    def test_denied_result(self):
        """Test creating a denied result."""
        result = SkillResult.denied("no access")

        assert result.status == SkillStatus.DENIED
        assert "Permission denied" in result.error_message

    def test_timeout_result(self):
        """Test creating a timeout result."""
        result = SkillResult.timeout(30)

        assert result.status == SkillStatus.TIMEOUT
        assert "30s" in result.error_message


class TestFunctionSkill:
    """Tests for FunctionSkill."""

    def test_register_and_execute_action(self, workspace, basic_manifest):
        """Test registering and executing an action."""
        skill = FunctionSkill(basic_manifest)

        @skill.register_action("greet")
        def greet(context: SkillContext) -> str:
            name = context.parameters.get("name", "World")
            return f"Hello, {name}!"

        context = SkillContext(
            workspace=workspace,
            parameters={"name": "Alice"},
        )

        result = skill.execute("greet", context)

        assert result.status == SkillStatus.SUCCESS
        assert result.data == "Hello, Alice!"

    def test_unknown_action(self, workspace, basic_manifest):
        """Test executing an unknown action."""
        skill = FunctionSkill(basic_manifest)

        context = SkillContext(workspace=workspace)
        result = skill.execute("unknown", context)

        assert result.status == SkillStatus.ERROR
        assert "Unknown action" in result.error_message

    def test_action_returning_skill_result(self, workspace, basic_manifest):
        """Test action that returns SkillResult directly."""
        skill = FunctionSkill(basic_manifest)

        @skill.register_action("add")
        def add(context: SkillContext) -> SkillResult:
            a = context.parameters.get("a", 0)
            b = context.parameters.get("b", 0)
            return SkillResult.success(a + b, operation="add")

        context = SkillContext(
            workspace=workspace,
            parameters={"a": 5, "b": 3},
        )

        result = skill.execute("add", context)

        assert result.status == SkillStatus.SUCCESS
        assert result.data == 8
        assert result.metadata["operation"] == "add"

    def test_action_exception_handling(self, workspace, basic_manifest):
        """Test that exceptions in actions are handled."""
        skill = FunctionSkill(basic_manifest)

        @skill.register_action("greet")
        def greet(_context: SkillContext):
            raise ValueError("Something went wrong")

        context = SkillContext(workspace=workspace)
        result = skill.execute("greet", context)

        assert result.status == SkillStatus.ERROR
        assert "Something went wrong" in result.error_message

    def test_get_actions(self, basic_manifest):
        """Test getting available actions."""
        skill = FunctionSkill(basic_manifest)

        actions = skill.get_actions()

        assert "greet" in actions
        assert "add" in actions

    def test_validate_action(self, basic_manifest):
        """Test action validation."""
        skill = FunctionSkill(basic_manifest)

        assert skill.validate_action("greet") is True
        assert skill.validate_action("unknown") is False


class TestSkillContext:
    """Tests for SkillContext."""

    def test_default_values(self, workspace):
        """Test default context values."""
        context = SkillContext(workspace=workspace)

        assert context.workspace == workspace
        assert context.parameters == {}
        assert context.environment == {}
        assert context.timeout_seconds == 30
        assert context.dry_run is False

    def test_custom_values(self, workspace):
        """Test custom context values."""
        context = SkillContext(
            workspace=workspace,
            parameters={"key": "value"},
            environment={"VAR": "test"},
            timeout_seconds=60,
            dry_run=True,
            trace_id="abc123",
        )

        assert context.parameters["key"] == "value"
        assert context.environment["VAR"] == "test"
        assert context.timeout_seconds == 60
        assert context.dry_run is True
        assert context.trace_id == "abc123"
