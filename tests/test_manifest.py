"""Tests for skill manifest."""

import tempfile
from pathlib import Path

import pytest

from skillguard.sdk.manifest import (
    FilesystemAccess,
    FilesystemPermission,
    HttpMethod,
    NetworkPermission,
    Permission,
    PermissionLevel,
    SkillAction,
    SkillManifest,
)


class TestPermission:
    """Tests for Permission model."""

    def test_minimal_level(self):
        """Test minimal permission level detection."""
        perm = Permission()
        assert perm.level == PermissionLevel.MINIMAL

    def test_restricted_level_network(self):
        """Test restricted level with network permissions."""
        perm = Permission(
            network=[NetworkPermission(domain="api.example.com")]
        )
        assert perm.level == PermissionLevel.RESTRICTED

    def test_restricted_level_filesystem(self):
        """Test restricted level with filesystem permissions."""
        perm = Permission(
            filesystem=[FilesystemPermission(path="${WORKSPACE}/**")]
        )
        assert perm.level == PermissionLevel.RESTRICTED

    def test_standard_level_subprocess_allowlist(self):
        """Test standard level with subprocess allowlist."""
        perm = Permission(
            subprocess=True,
            subprocess_allowlist=["git", "ls"]
        )
        assert perm.level == PermissionLevel.STANDARD

    def test_privileged_level_unrestricted_subprocess(self):
        """Test privileged level with unrestricted subprocess."""
        perm = Permission(subprocess=True)
        assert perm.level == PermissionLevel.PRIVILEGED


class TestNetworkPermission:
    """Tests for NetworkPermission model."""

    def test_defaults(self):
        """Test default values."""
        perm = NetworkPermission(domain="api.example.com")
        assert perm.methods == [HttpMethod.GET]
        assert perm.ports == [443, 80]

    def test_domain_validation(self):
        """Test domain normalization."""
        perm = NetworkPermission(domain="API.EXAMPLE.COM  ")
        assert perm.domain == "api.example.com"

    def test_empty_domain_rejected(self):
        """Test that empty domains are rejected."""
        with pytest.raises(ValueError):
            NetworkPermission(domain="  ")


class TestFilesystemPermission:
    """Tests for FilesystemPermission model."""

    def test_defaults(self):
        """Test default values."""
        perm = FilesystemPermission(path="${WORKSPACE}/**")
        assert perm.access == [FilesystemAccess.READ]

    def test_empty_path_rejected(self):
        """Test that empty paths are rejected."""
        with pytest.raises(ValueError):
            FilesystemPermission(path="  ")


class TestSkillManifest:
    """Tests for SkillManifest model."""

    def test_valid_manifest(self):
        """Test creating a valid manifest."""
        manifest = SkillManifest(
            name="test-skill",
            version="1.0.0",
            description="A test skill",
            author="test-author",
        )
        assert manifest.name == "test-skill"
        assert manifest.version == "1.0.0"

    def test_invalid_name_rejected(self):
        """Test that invalid names are rejected."""
        with pytest.raises(ValueError):
            SkillManifest(
                name="Invalid Name!",
                version="1.0.0",
                description="Test",
                author="test",
            )

    def test_invalid_version_rejected(self):
        """Test that invalid versions are rejected."""
        with pytest.raises(ValueError):
            SkillManifest(
                name="test",
                version="not-a-version",
                description="Test",
                author="test",
            )

    def test_yaml_roundtrip(self):
        """Test YAML serialization roundtrip."""
        manifest = SkillManifest(
            name="test-skill",
            version="1.0.0",
            description="A test skill",
            author="test-author",
            permissions=Permission(
                network=[NetworkPermission(domain="api.example.com")],
                filesystem=[FilesystemPermission(path="${WORKSPACE}/**")],
            ),
            actions=[
                SkillAction(
                    name="test",
                    description="Test action",
                    parameters={"input": {"type": "string"}},
                )
            ],
        )

        with tempfile.TemporaryDirectory() as tmpdir:
            yaml_path = Path(tmpdir) / "skillguard.yaml"
            manifest.to_yaml(yaml_path)

            loaded = SkillManifest.from_yaml(yaml_path)

            assert loaded.name == manifest.name
            assert loaded.version == manifest.version
            assert len(loaded.permissions.network) == 1
            assert len(loaded.actions) == 1
