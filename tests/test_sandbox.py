"""Tests for sandbox security."""

import tempfile
from pathlib import Path

import pytest

from skillguard.sdk.manifest import (
    FilesystemAccess,
    FilesystemPermission,
    HttpMethod,
    NetworkPermission,
    Permission,
    SkillManifest,
)
from skillguard.sdk.sandbox import Sandbox, SandboxConfig, SandboxViolation


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
        permissions=Permission(
            network=[
                NetworkPermission(
                    domain="api.example.com",
                    methods=[HttpMethod.GET, HttpMethod.POST],
                )
            ],
            filesystem=[
                FilesystemPermission(
                    path="${WORKSPACE}/**",
                    access=[FilesystemAccess.READ, FilesystemAccess.WRITE],
                )
            ],
            subprocess=True,
            subprocess_allowlist=["git", "ls"],
        ),
    )


class TestNetworkAccess:
    """Tests for network access control."""

    def test_allowed_domain_and_method(self, workspace, basic_manifest):
        """Test access to allowed domain with allowed method."""
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        assert sandbox.check_network_access("https://api.example.com/endpoint", HttpMethod.GET)
        assert sandbox.check_network_access("https://api.example.com/endpoint", HttpMethod.POST)

    def test_blocked_domain(self, workspace, basic_manifest):
        """Test access to blocked domain."""
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        assert not sandbox.check_network_access("https://evil.com/steal-data")

    def test_blocked_method(self, workspace, basic_manifest):
        """Test blocked HTTP method."""
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        assert not sandbox.check_network_access("https://api.example.com/endpoint", HttpMethod.DELETE)

    def test_require_network_raises(self, workspace, basic_manifest):
        """Test that require_network raises on violation."""
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        with pytest.raises(SandboxViolation):
            sandbox.require_network("https://evil.com/steal-data")


class TestFilesystemAccess:
    """Tests for filesystem access control."""

    def test_allowed_workspace_path(self, workspace, basic_manifest):
        """Test access to workspace paths."""
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        test_file = workspace / "test.txt"
        assert sandbox.check_filesystem_access(test_file, FilesystemAccess.READ)
        assert sandbox.check_filesystem_access(test_file, FilesystemAccess.WRITE)

    def test_blocked_outside_workspace(self, workspace, basic_manifest):
        """Test access outside workspace is blocked."""
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        assert not sandbox.check_filesystem_access("/etc/passwd", FilesystemAccess.READ)
        assert not sandbox.check_filesystem_access("/root/.ssh/id_rsa", FilesystemAccess.READ)

    def test_require_filesystem_raises(self, workspace, basic_manifest):
        """Test that require_filesystem raises on violation."""
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        with pytest.raises(SandboxViolation):
            sandbox.require_filesystem("/etc/passwd")


class TestSubprocessAccess:
    """Tests for subprocess access control."""

    def test_allowed_command(self, workspace, basic_manifest):
        """Test allowed subprocess command."""
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        assert sandbox.check_subprocess("git")
        assert sandbox.check_subprocess(["git", "status"])
        assert sandbox.check_subprocess("ls")

    def test_blocked_command(self, workspace, basic_manifest):
        """Test blocked subprocess command."""
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        assert not sandbox.check_subprocess("rm")
        assert not sandbox.check_subprocess(["curl", "http://evil.com"])

    def test_subprocess_disabled(self, workspace):
        """Test when subprocess is disabled."""
        manifest = SkillManifest(
            name="test",
            version="1.0.0",
            description="Test",
            author="test",
            permissions=Permission(subprocess=False),
        )
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(manifest, config)

        assert not sandbox.check_subprocess("git")
        assert not sandbox.check_subprocess("ls")


class TestSandboxFileOperations:
    """Tests for sandbox file operations."""

    def test_read_file(self, workspace, basic_manifest):
        """Test reading a file through sandbox."""
        test_file = workspace / "test.txt"
        test_file.write_text("hello world")

        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        content = sandbox.read_file(test_file)
        assert content == "hello world"

    def test_write_file(self, workspace, basic_manifest):
        """Test writing a file through sandbox."""
        test_file = workspace / "output.txt"

        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        sandbox.write_file(test_file, "test content")
        assert test_file.read_text() == "test content"

    def test_read_outside_workspace_blocked(self, workspace, basic_manifest):
        """Test reading outside workspace is blocked."""
        config = SandboxConfig(workspace=workspace)
        sandbox = Sandbox(basic_manifest, config)

        with pytest.raises(SandboxViolation):
            sandbox.read_file("/etc/passwd")
