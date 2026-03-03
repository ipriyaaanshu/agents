"""Capability-based sandbox for secure skill execution."""

from __future__ import annotations

import os
import subprocess
import tempfile
from dataclasses import dataclass, field
from fnmatch import fnmatch
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

from loguru import logger

from skillguard.sdk.manifest import (
    FilesystemAccess,
    HttpMethod,
    SkillManifest,
)


class SandboxViolation(Exception):
    """Raised when a skill violates sandbox rules."""
    pass


@dataclass
class SandboxConfig:
    """Configuration for the sandbox environment."""

    workspace: Path
    """The workspace directory."""

    temp_dir: Path | None = None
    """Temporary directory for skill use."""

    timeout_seconds: int = 30
    """Maximum execution time."""

    max_memory_mb: int = 512
    """Maximum memory usage."""

    max_output_bytes: int = 1024 * 1024
    """Maximum output size (1MB default)."""

    allowed_env_vars: set[str] = field(default_factory=set)
    """Environment variables to pass through."""


class Sandbox:
    """Capability-based sandbox for skill execution."""

    def __init__(self, manifest: SkillManifest, config: SandboxConfig) -> None:
        self.manifest = manifest
        self.config = config
        self.permissions = manifest.permissions
        self._logger = logger.bind(skill=manifest.name, sandbox=True)

        if config.temp_dir is None:
            self._temp_dir = Path(tempfile.mkdtemp(prefix=f"skillguard-{manifest.name}-"))
            self._owns_temp = True
        else:
            self._temp_dir = config.temp_dir
            self._owns_temp = False

    def check_network_access(self, url: str, method: HttpMethod = HttpMethod.GET) -> bool:
        """Check if network access to a URL is allowed."""
        parsed = urlparse(url)
        domain = parsed.netloc.lower()

        if ":" in domain:
            domain, port_str = domain.rsplit(":", 1)
            try:
                port = int(port_str)
            except ValueError:
                port = 443 if parsed.scheme == "https" else 80
        else:
            port = 443 if parsed.scheme == "https" else 80

        for perm in self.permissions.network:
            if self._match_domain(domain, perm.domain) and method in perm.methods and port in perm.ports:
                return True

        return False

    def _match_domain(self, actual: str, pattern: str) -> bool:
        """Match a domain against a pattern (supports wildcards)."""
        if pattern.startswith("*."):
            suffix = pattern[1:]
            return actual.endswith(suffix) or actual == pattern[2:]
        return actual == pattern

    def check_filesystem_access(
        self,
        path: Path | str,
        access: FilesystemAccess = FilesystemAccess.READ
    ) -> bool:
        """Check if filesystem access to a path is allowed."""
        path = Path(path).resolve()

        for perm in self.permissions.filesystem:
            if access not in perm.access:
                continue

            pattern = perm.path.replace("${WORKSPACE}", str(self.config.workspace))
            pattern = pattern.replace("${TEMP}", str(self._temp_dir))

            if self._match_path(path, pattern):
                return True

        return False

    def _match_path(self, path: Path, pattern: str) -> bool:
        """Match a path against a pattern (supports globs)."""
        pattern_path = Path(pattern)

        if "**" in pattern or "*" in pattern:
            return fnmatch(str(path), pattern)

        try:
            path.relative_to(pattern_path)
            return True
        except ValueError:
            return path == pattern_path

    def check_subprocess(self, command: str | list[str]) -> bool:
        """Check if subprocess execution is allowed."""
        if not self.permissions.subprocess:
            return False

        if not self.permissions.subprocess_allowlist:
            return True

        if isinstance(command, list):
            cmd = command[0] if command else ""
        else:
            cmd = command.split()[0] if command else ""

        return cmd in self.permissions.subprocess_allowlist

    def check_environment(self, var_name: str) -> bool:
        """Check if access to an environment variable is allowed."""
        return any(perm.name == var_name for perm in self.permissions.environment)

    def get_environment(self) -> dict[str, str]:
        """Get the filtered environment for skill execution."""
        env = {}
        for perm in self.permissions.environment:
            value = os.environ.get(perm.name)
            if value is not None:
                env[perm.name] = value
            elif perm.required:
                raise SandboxViolation(
                    f"Required environment variable not set: {perm.name}"
                )
        return env

    def require_network(self, url: str, method: HttpMethod = HttpMethod.GET) -> None:
        """Require network access, raising if denied."""
        if not self.check_network_access(url, method):
            raise SandboxViolation(
                f"Network access denied: {method.value} {url}"
            )

    def require_filesystem(
        self,
        path: Path | str,
        access: FilesystemAccess = FilesystemAccess.READ
    ) -> None:
        """Require filesystem access, raising if denied."""
        if not self.check_filesystem_access(path, access):
            raise SandboxViolation(
                f"Filesystem access denied: {access.value} {path}"
            )

    def require_subprocess(self, command: str | list[str]) -> None:
        """Require subprocess access, raising if denied."""
        if not self.check_subprocess(command):
            raise SandboxViolation(f"Subprocess execution denied: {command}")

    def run_subprocess(
        self,
        command: list[str],
        capture_output: bool = True,
        timeout: int | None = None,
    ) -> subprocess.CompletedProcess:
        """Run a subprocess in the sandbox."""
        self.require_subprocess(command)

        timeout = timeout or self.config.timeout_seconds
        env = self.get_environment()

        self._logger.debug(f"Running subprocess: {' '.join(command)}")

        return subprocess.run(
            command,
            capture_output=capture_output,
            timeout=timeout,
            env=env,
            cwd=self.config.workspace,
        )

    def read_file(self, path: Path | str) -> str:
        """Read a file in the sandbox."""
        path = Path(path).resolve()
        self.require_filesystem(path, FilesystemAccess.READ)

        with open(path) as f:
            content = f.read(self.config.max_output_bytes)

        return content

    def write_file(self, path: Path | str, content: str) -> None:
        """Write a file in the sandbox."""
        path = Path(path).resolve()
        self.require_filesystem(path, FilesystemAccess.WRITE)

        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "w") as f:
            f.write(content)

    @property
    def temp_dir(self) -> Path:
        """Get the temporary directory."""
        return self._temp_dir

    def cleanup(self) -> None:
        """Clean up sandbox resources."""
        if self._owns_temp and self._temp_dir.exists():
            import shutil
            shutil.rmtree(self._temp_dir, ignore_errors=True)

    def __enter__(self) -> Sandbox:
        return self

    def __exit__(self, *args: Any) -> None:
        self.cleanup()
