"""Search for skills in the registry."""

from typing import Any

from rich.console import Console
from rich.table import Table

console = Console()

MOCK_SKILLS: list[dict[str, Any]] = [
    {
        "name": "github-ops",
        "version": "1.2.0",
        "description": "GitHub operations for AI agents",
        "author": "skillguard-official",
        "downloads": 15420,
        "verified": True,
    },
    {
        "name": "file-ops",
        "version": "1.0.0",
        "description": "File operations with sandboxing",
        "author": "skillguard-official",
        "downloads": 12850,
        "verified": True,
    },
    {
        "name": "web-fetch",
        "version": "0.9.0",
        "description": "Fetch and parse web content",
        "author": "skillguard-official",
        "downloads": 9340,
        "verified": True,
    },
    {
        "name": "slack-notify",
        "version": "1.1.0",
        "description": "Send Slack notifications",
        "author": "community",
        "downloads": 5620,
        "verified": False,
    },
    {
        "name": "data-transform",
        "version": "0.8.0",
        "description": "Transform JSON, CSV, and text data",
        "author": "community",
        "downloads": 4210,
        "verified": False,
    },
]


def search_skills(query: str, limit: int) -> None:
    """Search for skills in the registry."""
    console.print(f"\n[bold]Searching for: {query}[/bold]\n")

    query_lower = query.lower()
    results = [
        s for s in MOCK_SKILLS
        if query_lower in s["name"].lower() or query_lower in s["description"].lower()
    ][:limit]

    if not results:
        console.print("[yellow]No skills found matching your query[/yellow]")
        console.print("\n[dim]Note: Registry search is using mock data. Full registry coming soon.[/dim]")
        return

    table = Table(title=f"Search Results ({len(results)} found)")
    table.add_column("Name", style="cyan")
    table.add_column("Version")
    table.add_column("Description")
    table.add_column("Author")
    table.add_column("Downloads", justify="right")
    table.add_column("Verified")

    for skill in results:
        verified = "[green]✓[/green]" if skill["verified"] else "[dim]-[/dim]"
        table.add_row(
            skill["name"],
            skill["version"],
            skill["description"],
            skill["author"],
            f"{skill['downloads']:,}",
            verified,
        )

    console.print(table)
    console.print("\n[dim]Install with: skillguard install <name>[/dim]")
    console.print("[dim]Note: Using mock data. Full registry coming soon.[/dim]")
