"""
file-ops - Secure file operations for AI agents

A SkillGuard official skill for reading, writing, and searching files
within the workspace sandbox.
"""

import re
from fnmatch import fnmatch
from pathlib import Path

from skillguard.sdk import Skill, SkillContext, SkillManifest, SkillResult


class FileOpsSkill(Skill):
    """Secure file operations skill."""

    def execute(self, action: str, context: SkillContext) -> SkillResult:
        """Execute a file operation."""
        actions = {
            "read": self._read,
            "write": self._write,
            "list": self._list,
            "search": self._search,
        }

        handler = actions.get(action)
        if handler is None:
            return SkillResult.error(f"Unknown action: {action}")

        return handler(context)

    def _read(self, context: SkillContext) -> SkillResult:
        """Read a file."""
        rel_path = context.parameters.get("path")
        if not rel_path:
            return SkillResult.error("Missing required parameter: path")

        file_path = (context.workspace / rel_path).resolve()

        if not self._is_within_workspace(file_path, context.workspace):
            return SkillResult.denied(f"Path outside workspace: {rel_path}")

        if not file_path.exists():
            return SkillResult.error(f"File not found: {rel_path}")

        if not file_path.is_file():
            return SkillResult.error(f"Not a file: {rel_path}")

        try:
            content = file_path.read_text()
            return SkillResult.success(content, bytes=len(content.encode()))
        except Exception as e:
            return SkillResult.error(f"Failed to read file: {e}")

    def _write(self, context: SkillContext) -> SkillResult:
        """Write to a file."""
        rel_path = context.parameters.get("path")
        content = context.parameters.get("content")

        if not rel_path:
            return SkillResult.error("Missing required parameter: path")
        if content is None:
            return SkillResult.error("Missing required parameter: content")

        file_path = (context.workspace / rel_path).resolve()

        if not self._is_within_workspace(file_path, context.workspace):
            return SkillResult.denied(f"Path outside workspace: {rel_path}")

        if context.dry_run:
            return SkillResult.success(True, dry_run=True, would_write=len(content))

        try:
            file_path.parent.mkdir(parents=True, exist_ok=True)
            file_path.write_text(content)
            return SkillResult.success(True, bytes_written=len(content.encode()))
        except Exception as e:
            return SkillResult.error(f"Failed to write file: {e}")

    def _list(self, context: SkillContext) -> SkillResult:
        """List files in a directory."""
        rel_path = context.parameters.get("path", ".")
        pattern = context.parameters.get("pattern", "*")

        dir_path = (context.workspace / rel_path).resolve()

        if not self._is_within_workspace(dir_path, context.workspace):
            return SkillResult.denied(f"Path outside workspace: {rel_path}")

        if not dir_path.exists():
            return SkillResult.error(f"Directory not found: {rel_path}")

        if not dir_path.is_dir():
            return SkillResult.error(f"Not a directory: {rel_path}")

        try:
            files = []
            for file_path in dir_path.rglob("*"):
                if file_path.is_file() and fnmatch(file_path.name, pattern):
                    rel = file_path.relative_to(context.workspace)
                    files.append(str(rel))

            return SkillResult.success(sorted(files), count=len(files))
        except Exception as e:
            return SkillResult.error(f"Failed to list files: {e}")

    def _search(self, context: SkillContext) -> SkillResult:
        """Search for text in files."""
        pattern = context.parameters.get("pattern")
        rel_path = context.parameters.get("path", ".")

        if not pattern:
            return SkillResult.error("Missing required parameter: pattern")

        dir_path = (context.workspace / rel_path).resolve()

        if not self._is_within_workspace(dir_path, context.workspace):
            return SkillResult.denied(f"Path outside workspace: {rel_path}")

        try:
            regex = re.compile(pattern)
        except re.error as e:
            return SkillResult.error(f"Invalid regex pattern: {e}")

        matches = []
        try:
            for file_path in dir_path.rglob("*"):
                if not file_path.is_file():
                    continue

                try:
                    content = file_path.read_text()
                except (UnicodeDecodeError, PermissionError):
                    continue

                for line_num, line in enumerate(content.split("\n"), 1):
                    if regex.search(line):
                        rel = file_path.relative_to(context.workspace)
                        matches.append({
                            "file": str(rel),
                            "line": line_num,
                            "content": line.strip()[:200],
                        })

            return SkillResult.success(matches, count=len(matches))
        except Exception as e:
            return SkillResult.error(f"Search failed: {e}")

    def _is_within_workspace(self, path: Path, workspace: Path) -> bool:
        """Check if a path is within the workspace."""
        try:
            path.relative_to(workspace.resolve())
            return True
        except ValueError:
            return False


def create_skill() -> FileOpsSkill:
    """Factory function to create the skill instance."""
    manifest = SkillManifest.from_yaml(Path(__file__).parent / "skillguard.yaml")
    return FileOpsSkill(manifest)
