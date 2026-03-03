"""Skill manifest schema - explicit permission declarations."""

from __future__ import annotations

from enum import StrEnum
from pathlib import Path
from typing import Literal

import yaml
from pydantic import BaseModel, Field, field_validator


class HttpMethod(StrEnum):
    """Allowed HTTP methods."""
    GET = "GET"
    POST = "POST"
    PUT = "PUT"
    PATCH = "PATCH"
    DELETE = "DELETE"
    HEAD = "HEAD"
    OPTIONS = "OPTIONS"


class FilesystemAccess(StrEnum):
    """Filesystem access levels."""
    READ = "read"
    WRITE = "write"
    EXECUTE = "execute"


class PermissionLevel(StrEnum):
    """Overall permission levels for skills."""
    MINIMAL = "minimal"
    RESTRICTED = "restricted"
    STANDARD = "standard"
    PRIVILEGED = "privileged"


class NetworkPermission(BaseModel):
    """Network access permission for a specific domain."""

    domain: str = Field(..., description="Domain or IP to allow access to")
    methods: list[HttpMethod] = Field(
        default=[HttpMethod.GET],
        description="Allowed HTTP methods"
    )
    ports: list[int] = Field(
        default=[443, 80],
        description="Allowed ports"
    )

    @field_validator("domain")
    @classmethod
    def validate_domain(cls, v: str) -> str:
        if not v or v.isspace():
            raise ValueError("Domain cannot be empty")
        return v.lower().strip()


class FilesystemPermission(BaseModel):
    """Filesystem access permission for a specific path pattern."""

    path: str = Field(..., description="Path pattern (supports ${WORKSPACE} variable)")
    access: list[FilesystemAccess] = Field(
        default=[FilesystemAccess.READ],
        description="Allowed access types"
    )

    @field_validator("path")
    @classmethod
    def validate_path(cls, v: str) -> str:
        if not v or v.isspace():
            raise ValueError("Path cannot be empty")
        return v.strip()


class EnvironmentPermission(BaseModel):
    """Environment variable access permission."""

    name: str = Field(..., description="Environment variable name")
    required: bool = Field(default=False, description="Whether the variable is required")
    sensitive: bool = Field(default=False, description="Whether the variable contains sensitive data")


class Permission(BaseModel):
    """Complete permission manifest for a skill."""

    network: list[NetworkPermission] = Field(
        default_factory=list,
        description="Network access permissions"
    )
    filesystem: list[FilesystemPermission] = Field(
        default_factory=list,
        description="Filesystem access permissions"
    )
    environment: list[EnvironmentPermission] = Field(
        default_factory=list,
        description="Environment variable permissions"
    )
    subprocess: bool = Field(
        default=False,
        description="Whether subprocess execution is allowed"
    )
    subprocess_allowlist: list[str] = Field(
        default_factory=list,
        description="Allowed subprocess commands (if subprocess is True)"
    )

    @property
    def level(self) -> PermissionLevel:
        """Compute the effective permission level."""
        if not self.network and not self.filesystem and not self.subprocess:
            return PermissionLevel.MINIMAL
        if self.subprocess and not self.subprocess_allowlist:
            return PermissionLevel.PRIVILEGED
        if self.subprocess:
            return PermissionLevel.STANDARD
        return PermissionLevel.RESTRICTED


class AdapterConfig(BaseModel):
    """Framework adapter compatibility configuration."""

    openclaw: str | None = Field(default=None, description="OpenClaw version constraint")
    langchain: str | None = Field(default=None, description="LangChain version constraint")
    crewai: str | None = Field(default=None, description="CrewAI version constraint")
    mcp: str | None = Field(default=None, description="MCP version constraint")
    autogpt: str | None = Field(default=None, description="AutoGPT version constraint")


class BuildConfig(BaseModel):
    """Build configuration for reproducible builds."""

    reproducible: bool = Field(default=True, description="Whether builds should be reproducible")
    base: str = Field(
        default="skillguard/python:3.11-minimal",
        description="Base image for building"
    )
    python_version: str = Field(default="3.11", description="Python version to use")
    dependencies: list[str] = Field(
        default_factory=list,
        description="Additional pip dependencies"
    )


class SecurityMetadata(BaseModel):
    """Security audit metadata."""

    audit_date: str | None = Field(default=None, description="Date of last security audit")
    auditor: str | None = Field(default=None, description="Entity that performed the audit")
    slsa_level: Literal[0, 1, 2, 3, 4] = Field(
        default=0,
        description="SLSA compliance level"
    )
    cve_scan_date: str | None = Field(default=None, description="Date of last CVE scan")
    known_vulnerabilities: list[str] = Field(
        default_factory=list,
        description="Known CVEs (should be empty for verified skills)"
    )


class SkillAction(BaseModel):
    """A single action that the skill can perform."""

    name: str = Field(..., description="Action name")
    description: str = Field(..., description="Human-readable description")
    parameters: dict[str, dict] = Field(
        default_factory=dict,
        description="JSON Schema for action parameters"
    )
    returns: dict = Field(
        default_factory=dict,
        description="JSON Schema for return value"
    )


class SkillManifest(BaseModel):
    """Complete skill manifest - the security contract for a skill."""

    name: str = Field(..., description="Skill name (lowercase, alphanumeric, hyphens)")
    version: str = Field(..., description="Semantic version")
    description: str = Field(..., description="Human-readable description")
    author: str = Field(..., description="Author or organization")
    license: str = Field(default="Apache-2.0", description="SPDX license identifier")
    homepage: str | None = Field(default=None, description="Project homepage URL")
    repository: str | None = Field(default=None, description="Source repository URL")

    permissions: Permission = Field(
        default_factory=Permission,
        description="Permission manifest"
    )
    adapters: AdapterConfig = Field(
        default_factory=AdapterConfig,
        description="Framework adapter compatibility"
    )
    build: BuildConfig = Field(
        default_factory=BuildConfig,
        description="Build configuration"
    )
    security: SecurityMetadata = Field(
        default_factory=SecurityMetadata,
        description="Security metadata"
    )

    actions: list[SkillAction] = Field(
        default_factory=list,
        description="Actions this skill can perform"
    )

    keywords: list[str] = Field(
        default_factory=list,
        description="Keywords for discovery"
    )

    @field_validator("name")
    @classmethod
    def validate_name(cls, v: str) -> str:
        import re
        if not re.match(r"^[a-z][a-z0-9-]*[a-z0-9]$", v) and len(v) > 1 and not re.match(r"^[a-z]$", v):
            raise ValueError(
                "Name must be lowercase, start with a letter, "
                "and contain only letters, numbers, and hyphens"
            )
        return v

    @field_validator("version")
    @classmethod
    def validate_version(cls, v: str) -> str:
        from packaging.version import Version
        try:
            Version(v)
        except Exception as e:
            raise ValueError(f"Invalid semantic version: {v}") from e
        return v

    @classmethod
    def from_yaml(cls, path: Path | str) -> SkillManifest:
        """Load a manifest from a YAML file."""
        path = Path(path)
        if not path.exists():
            raise FileNotFoundError(f"Manifest not found: {path}")

        with open(path) as f:
            data = yaml.safe_load(f)

        return cls.model_validate(data)

    def to_yaml(self, path: Path | str) -> None:
        """Write the manifest to a YAML file."""
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)

        with open(path, "w") as f:
            yaml.dump(
                self.model_dump(exclude_none=True, exclude_defaults=True),
                f,
                default_flow_style=False,
                sort_keys=False,
            )

    def to_yaml_string(self) -> str:
        """Serialize the manifest to a YAML string."""
        return yaml.dump(
            self.model_dump(exclude_none=True),
            default_flow_style=False,
            sort_keys=False,
        )
