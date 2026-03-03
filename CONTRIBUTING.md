# Contributing to SkillGuard

Thank you for your interest in contributing to SkillGuard! This document provides guidelines for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- Python 3.11+ (for the thin Python SDK, optional)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/ipriyaaanshu/agents.git
cd agents

# Build the Rust workspace
cd rust
cargo build

# Run tests
cargo test --workspace

# Run linting
cargo clippy --workspace --all-targets -- -D warnings

# Check formatting
cargo fmt --all -- --check
```

### Python SDK (optional)

```bash
cd rust/sdk/python
pip install -e ".[all]"
```

## Project Structure

```
agents/
├── rust/                          # Rust workspace (primary implementation)
│   ├── crates/
│   │   ├── skillguard-core/       # Types, manifest, validation
│   │   ├── skillguard-sandbox/    # Wasmtime WASI sandbox
│   │   ├── skillguard-signing/    # Sigstore signing + SLSA provenance
│   │   ├── skillguard-registry/   # Git-based registry client
│   │   └── skillguard-cli/        # CLI binary (clap)
│   └── sdk/python/                # Thin Python SDK (delegates to CLI)
├── skills/                        # Example skills
└── .github/workflows/             # CI/CD
```

## Types of Contributions

### 1. Bug Reports

When filing a bug report, please include:

- A clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Your environment (OS, Rust version, etc.)

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

For changes to the Rust crates, CLI, or Python SDK:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test --workspace`)
6. Ensure clippy passes (`cargo clippy --workspace --all-targets -- -D warnings`)
7. Ensure formatting passes (`cargo fmt --all -- --check`)
8. Commit your changes (`git commit -m 'Add amazing feature'`)
9. Push to the branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

## Code Style

### Rust

- `cargo fmt` for formatting
- `cargo clippy` with `-D warnings` for linting
- Doc comments on all public items

### Python SDK

- Type hints on all functions
- Keep the SDK thin — delegate to the CLI binary

## Security

Security is core to SkillGuard. When contributing:

- **Never bypass sandbox checks** in skill implementations
- **Minimize permissions** in skill manifests
- **Report security issues privately** via GitHub Security Advisories
- **Don't commit secrets** or credentials

### Security Review Process

All skill contributions undergo security review:

1. Automated static analysis (`skillguard audit`)
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
- [ ] `cargo clippy` passes with `-D warnings`
- [ ] `cargo fmt` passes
- [ ] `cargo test --workspace` passes

## Skill Publishing

To publish a skill to the official registry:

1. Ensure your skill follows all guidelines
2. Run `skillguard audit` and resolve all issues
3. Build with `skillguard build --sign`
4. Publish with `skillguard publish`

### Skill Guidelines

- **Clear purpose** - One skill, one responsibility
- **Minimal permissions** - Least privilege principle
- **Good documentation** - Clear action descriptions and parameter schemas
- **Comprehensive tests** - Edge cases covered
- **Semantic versioning** - Follow semver

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.
