"""MCP (Model Context Protocol) adapter for SkillGuard skills."""

from __future__ import annotations

import json
from typing import Any

from skillguard.client import SkillGuardClient


def handle_call(
    client: SkillGuardClient,
    skill: str,
    action: str,
    params: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Handle an MCP tool call by delegating to SkillGuard.

    Args:
        client: SkillGuard client instance.
        skill: Skill name or path.
        action: Action name.
        params: Tool call parameters.

    Returns:
        MCP-compatible response dict with content array.
    """
    result = client.run(skill, action, params)

    if result.status == "success":
        content = json.dumps(result.data) if result.data else "Success"
        return {
            "content": [{"type": "text", "text": content}],
            "isError": False,
        }
    else:
        return {
            "content": [
                {
                    "type": "text",
                    "text": f"Error: {result.error_message or 'Unknown error'}",
                }
            ],
            "isError": True,
        }


def skill_to_mcp_tool(
    client: SkillGuardClient,
    skill: str,
    action: str,
) -> dict[str, Any]:
    """Generate an MCP tool definition from a SkillGuard skill action.

    Args:
        client: SkillGuard client instance.
        skill: Skill name.
        action: Action name.

    Returns:
        MCP tool definition dict.
    """
    info = client.info(skill)
    action_info = None
    for act in info.get("actions", []):
        if act.get("name") == action:
            action_info = act
            break

    if not action_info:
        raise ValueError(f"Action '{action}' not found in skill '{skill}'")

    return {
        "name": f"{skill}.{action}",
        "description": action_info.get("description", ""),
        "inputSchema": {
            "type": "object",
            "properties": action_info.get("parameters", {}),
        },
    }
