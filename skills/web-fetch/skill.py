"""
web-fetch - Fetch and parse web content securely

A SkillGuard official skill for fetching web content with
sandboxed network access.
"""

from pathlib import Path

from skillguard.sdk import Skill, SkillContext, SkillManifest, SkillResult


class WebFetchSkill(Skill):
    """Secure web fetching skill."""

    def execute(self, action: str, context: SkillContext) -> SkillResult:
        """Execute a web fetch action."""
        actions = {
            "fetch": self._fetch,
            "fetch_json": self._fetch_json,
            "extract_text": self._extract_text,
        }

        handler = actions.get(action)
        if handler is None:
            return SkillResult.error(f"Unknown action: {action}")

        return handler(context)

    def _fetch(self, context: SkillContext) -> SkillResult:
        """Fetch content from a URL."""
        try:
            import httpx
        except ImportError:
            return SkillResult.error("httpx not installed")

        url = context.parameters.get("url")
        if not url:
            return SkillResult.error("Missing required parameter: url")

        headers = context.parameters.get("headers", {})

        if context.dry_run:
            return SkillResult.success(
                {"would_fetch": url},
                dry_run=True
            )

        try:
            with httpx.Client(timeout=context.timeout_seconds) as client:
                response = client.get(url, headers=headers)

                return SkillResult.success({
                    "status_code": response.status_code,
                    "headers": dict(response.headers),
                    "content": response.text[:100000],
                    "url": str(response.url),
                })
        except httpx.TimeoutException:
            return SkillResult.timeout(context.timeout_seconds)
        except Exception as e:
            return SkillResult.error(f"Fetch failed: {e}")

    def _fetch_json(self, context: SkillContext) -> SkillResult:
        """Fetch and parse JSON from a URL."""
        try:
            import httpx
        except ImportError:
            return SkillResult.error("httpx not installed")

        url = context.parameters.get("url")
        if not url:
            return SkillResult.error("Missing required parameter: url")

        if context.dry_run:
            return SkillResult.success({"would_fetch": url}, dry_run=True)

        try:
            with httpx.Client(timeout=context.timeout_seconds) as client:
                response = client.get(url)
                response.raise_for_status()

                data = response.json()
                return SkillResult.success(data)
        except httpx.TimeoutException:
            return SkillResult.timeout(context.timeout_seconds)
        except ValueError as e:
            return SkillResult.error(f"Invalid JSON response: {e}")
        except Exception as e:
            return SkillResult.error(f"Fetch failed: {e}")

    def _extract_text(self, context: SkillContext) -> SkillResult:
        """Fetch URL and extract readable text."""
        try:
            import httpx
            from bs4 import BeautifulSoup
        except ImportError as e:
            return SkillResult.error(f"Missing dependency: {e}")

        url = context.parameters.get("url")
        if not url:
            return SkillResult.error("Missing required parameter: url")

        selector = context.parameters.get("selector")

        if context.dry_run:
            return SkillResult.success({"would_fetch": url}, dry_run=True)

        try:
            with httpx.Client(timeout=context.timeout_seconds) as client:
                response = client.get(url)
                response.raise_for_status()

                soup = BeautifulSoup(response.text, "html.parser")

                for tag in soup(["script", "style", "nav", "footer", "header"]):
                    tag.decompose()

                if selector:
                    elements = soup.select(selector)
                    text = "\n\n".join(el.get_text(strip=True) for el in elements)
                else:
                    text = soup.get_text(separator="\n", strip=True)

                lines = [line.strip() for line in text.split("\n") if line.strip()]
                text = "\n".join(lines)

                return SkillResult.success(text[:50000], chars=len(text))
        except httpx.TimeoutException:
            return SkillResult.timeout(context.timeout_seconds)
        except Exception as e:
            return SkillResult.error(f"Extract failed: {e}")


def create_skill() -> WebFetchSkill:
    """Factory function to create the skill instance."""
    manifest = SkillManifest.from_yaml(Path(__file__).parent / "skillguard.yaml")
    return WebFetchSkill(manifest)
