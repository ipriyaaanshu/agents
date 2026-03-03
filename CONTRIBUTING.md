# Contributing to SkillGuard

Thank you for your interest in contributing to SkillGuard! This document provides guidelines for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## Getting Started

### Prerequisites

- Python 3.11+
- [uv](https://github.com/astral-sh/uv) for package management

### Development Setup

```bash
# Clone the repository
git clone https://github.com/skillguard/skillguard.git
cd skillguard

# Install dependencies
uv sync --all-extras

# Run tests
uv run pytest

# Run linting
uv run ruff check .

# Run type checking
uv run mypy src/
```

## Types of Contributions

### 1. Bug Reports

When filing a bug report, please include:

- A clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Your environment (Python version, OS, etc.)

### 2. Feature Requests

We welcome feature requests! Please:

- Check existing issues first
- Describe the use case
- Explain why this would benefit the project

### 3. Code Contributions

#### Skill Contributions

Contributing new skills is one of the best ways to help! When creating a skill:

1. **Follow the manifest schema** - Use `skillguard init` to scaffold
2. **Minimize permissions** - Request only what you need
3. **Write tests** - Cover happy paths and edge cases
4. **Document actions** - Clear descriptions and parameter schemas

#### Core Contributions

For changes to the SDK, CLI, or other core components:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`uv run pytest`)
6. Ensure linting passes (`uv run ruff check .`)
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## Code Style

We use:

- **ruff** for linting and formatting
- **mypy** for type checking
- **Google-style docstrings**

### Example

```python
def fetch_skill(name: str, version: str | None = None) -> Skill:
    """Fetch a skill from the registry.
    
    Args:
        name: The skill name.
        version: Optional version constraint. Defaults to latest.
        
    Returns:
        The loaded Skill instance.
        
    Raises:
        SkillNotFoundError: If the skill doesn't exist.
        VerificationError: If signature verification fails.
    """
    ...
```

## Security

Security is core to SkillGuard. When contributing:

- **Never bypass sandbox checks** in skill implementations
- **Minimize permissions** in skill manifests
- **Report security issues privately** to security@skillguard.dev
- **Don't commit secrets** or credentials

### Security Review Process

All skill contributions undergo security review:

1. Automated static analysis
2. Permission level assessment
3. Manual code review for high-permission skills

## Pull Request Process

1. Update documentation if needed
2. Add tests for new features
3. Ensure CI passes
4. Request review from maintainers
5. Address feedback promptly

### PR Checklist

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Linting passes
- [ ] Type checking passes
- [ ] Changelog entry added (if applicable)

## Skill Publishing

To publish a skill to the official registry:

1. Ensure your skill follows all guidelines
2. Submit a PR to `skillguard/registry`
3. Pass security review
4. Sign with your verified identity

### Skill Guidelines

- **Clear purpose** - One skill, one responsibility
- **Minimal permissions** - Least privilege principle
- **Good documentation** - README with examples
- **Comprehensive tests** - Edge cases covered
- **Semantic versioning** - Follow semver

## Questions?

- Open a [Discussion](https://github.com/skillguard/skillguard/discussions)
- Join our [Discord](https://discord.gg/skillguard)
- Email: hello@skillguard.dev

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.
