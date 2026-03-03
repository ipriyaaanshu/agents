# SkillGuard

**Chainguard for AI Agent Skills** — Secure, signed, and verified skills for any agent framework.

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Python](https://img.shields.io/badge/python-3.11+-green.svg)](https://python.org)
[![Security](https://img.shields.io/badge/security-first-red.svg)](#security)

---

## The Problem

The AI agent ecosystem has a **supply chain security crisis**:

- **ClawHavoc (Feb 2026)**: 341 malicious skills stealing user data, 283 with critical flaws
- **MalTool Report**: 6,487 malicious tools evading detection across ecosystems
- **82.4%** of LLMs execute malicious commands from peer agents
- Existing marketplaces prioritize growth over security

Meanwhile, skills are **fragmented** across frameworks (OpenClaw, LangChain, CrewAI, AutoGPT) with no universal standard for secure distribution.

## The Solution

**SkillGuard** applies Chainguard's proven model to AI agent skills:

```
┌─────────────────────────────────────────────────────────────────┐
│                        SKILLGUARD                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │   SKILL     │  │   BUILD     │  │   VERIFY    │              │
│  │   SOURCE    │──▶│   FACTORY   │──▶│   & SIGN    │──▶ REGISTRY │
│  │   (Git)     │  │  (Isolated) │  │  (Sigstore) │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│         │                │                │                      │
│         ▼                ▼                ▼                      │
│  ┌─────────────────────────────────────────────────┐            │
│  │              CONTINUOUS SECURITY                 │            │
│  │  • Daily rebuilds from source                   │            │
│  │  • Automated CVE patching                       │            │
│  │  • Dependency graph analysis                    │            │
│  │  • Runtime behavior monitoring                  │            │
│  └─────────────────────────────────────────────────┘            │
└─────────────────────────────────────────────────────────────────┘
```

## Core Principles

### 1. **Secure by Default**
- Skills run in capability-based sandboxes
- Explicit permission manifests (like Android/iOS)
- No network/filesystem access unless declared and approved

### 2. **Cryptographic Trust**
- Every skill signed with Sigstore
- Reproducible builds with full provenance
- Verify before install, always

### 3. **Framework Agnostic**
- Works with OpenClaw, LangChain, CrewAI, AutoGPT, MCP
- Adapters for existing standards (Agent Skills, OSSA, OAF)
- Write once, run anywhere

### 4. **Continuous Security**
- Daily rebuilds from source (not just version bumps)
- Automated vulnerability scanning
- Instant revocation for compromised skills

---

## Architecture

```
skillguard/
├── sdk/                    # Skill Development Kit
│   ├── manifest.py         # Permission manifest schema
│   ├── sandbox.py          # Capability-based sandbox runtime
│   ├── adapters/           # Framework adapters
│   │   ├── openclaw.py
│   │   ├── langchain.py
│   │   ├── crewai.py
│   │   └── mcp.py
│   └── testing.py          # Local testing harness
│
├── factory/                # Build Factory
│   ├── builder.py          # Isolated build environment
│   ├── scanner.py          # Static analysis + CVE scanning
│   ├── signer.py           # Sigstore integration
│   └── provenance.py       # SLSA provenance generation
│
├── registry/               # Skill Registry
│   ├── index.py            # Signed skill index
│   ├── resolver.py         # Dependency resolution (SAT-based)
│   └── mirror.py           # Decentralized mirroring
│
├── cli/                    # Command Line Interface
│   ├── main.py             # Entry point
│   ├── commands/
│   │   ├── init.py         # skillguard init
│   │   ├── build.py        # skillguard build
│   │   ├── verify.py       # skillguard verify
│   │   ├── install.py      # skillguard install
│   │   ├── publish.py      # skillguard publish
│   │   └── audit.py        # skillguard audit
│   └── ui.py               # Rich terminal UI
│
├── runtime/                # Secure Runtime
│   ├── executor.py         # Sandboxed skill execution
│   ├── monitor.py          # Runtime behavior monitoring
│   └── policy.py           # Policy enforcement engine
│
└── skills/                 # Official Verified Skills
    ├── file-ops/
    ├── web-fetch/
    ├── github-ops/
    └── ...
```

---

## Skill Manifest

Every skill declares its permissions explicitly:

```yaml
# skillguard.yaml
name: github-ops
version: 1.0.0
description: GitHub operations for AI agents
author: skillguard-official
license: Apache-2.0

# Explicit capability declarations
permissions:
  network:
    - domain: "api.github.com"
      methods: [GET, POST, PATCH]
    - domain: "github.com"
      methods: [GET]
  filesystem:
    - path: "${WORKSPACE}/**"
      access: [read, write]
  environment:
    - name: "GITHUB_TOKEN"
      required: true
      sensitive: true
  subprocess: false
  
# Framework compatibility
adapters:
  - openclaw: ">=2.0"
  - langchain: ">=0.3"
  - mcp: ">=1.0"

# Build configuration
build:
  reproducible: true
  base: "skillguard/python:3.11-minimal"
  
# Security metadata
security:
  audit: "2026-03-01"
  auditor: "skillguard-security-team"
  slsa-level: 3
```

---

## CLI Usage

```bash
# Install SkillGuard
pip install skillguard

# Initialize a new skill project
skillguard init my-skill --template=basic

# Build with full verification
skillguard build --sign

# Verify a skill before installing
skillguard verify github-ops@1.0.0

# Install a verified skill
skillguard install github-ops

# Audit installed skills for vulnerabilities
skillguard audit

# Run a skill in sandbox (for testing)
skillguard run github-ops --action=list-issues --repo=owner/repo

# Publish to registry (requires identity verification)
skillguard publish --sign
```

---

## Security Model

### Permission Levels

| Level | Network | Filesystem | Subprocess | Use Case |
|-------|---------|------------|------------|----------|
| **Minimal** | None | None | No | Pure computation |
| **Restricted** | Allowlist only | Workspace only | No | Most skills |
| **Standard** | Allowlist only | Workspace + temp | Allowlist | Complex skills |
| **Privileged** | Any | Any | Yes | System tools (rare) |

### Trust Hierarchy

```
┌─────────────────────────────────────────┐
│           SKILLGUARD OFFICIAL           │  ← Maintained by core team
│         (Highest trust, audited)        │
├─────────────────────────────────────────┤
│           VERIFIED PUBLISHERS           │  ← Identity verified, signed
│         (High trust, monitored)         │
├─────────────────────────────────────────┤
│          COMMUNITY PUBLISHERS           │  ← Signed, community reviewed
│         (Medium trust, sandboxed)       │
├─────────────────────────────────────────┤
│             UNVERIFIED                  │  ← Use at own risk
│         (Low trust, max sandbox)        │
└─────────────────────────────────────────┘
```

### Supply Chain Security (SLSA Level 3)

- **Source**: Verified git commits with signed tags
- **Build**: Isolated, reproducible builds
- **Provenance**: Full build attestation
- **Distribution**: Signed artifacts with Sigstore

---

## Framework Adapters

SkillGuard skills work everywhere:

```python
# Use with OpenClaw
from skillguard.adapters import openclaw
skill = openclaw.load("github-ops")

# Use with LangChain
from skillguard.adapters import langchain
tool = langchain.as_tool("github-ops")

# Use with CrewAI
from skillguard.adapters import crewai
crewai_tool = crewai.as_tool("github-ops")

# Use with MCP
from skillguard.adapters import mcp
mcp_server = mcp.as_server("github-ops")
```

---

## Roadmap

### Phase 1: Foundation (Current)
- [ ] SDK with manifest schema and sandbox runtime
- [ ] CLI for init, build, verify, install
- [ ] 5 official verified skills
- [ ] OpenClaw + LangChain adapters
- [ ] Basic Sigstore signing

### Phase 2: Registry & Distribution
- [ ] Decentralized registry (Git-based)
- [ ] Dependency resolution with SAT solver
- [ ] Mirror network for availability
- [ ] CVE database integration

### Phase 3: Continuous Security
- [ ] Daily automated rebuilds
- [ ] Runtime behavior monitoring
- [ ] Anomaly detection
- [ ] Instant revocation system

### Phase 4: Ecosystem
- [ ] Web UI for browsing skills
- [ ] Publisher verification program
- [ ] Security audit partnerships
- [ ] Enterprise features

---

## Comparison

| Feature | ClawHub | Agent Skills | SkillFortify | **SkillGuard** |
|---------|---------|--------------|--------------|----------------|
| Framework agnostic | ❌ | ✅ | ❌ | ✅ |
| Cryptographic signing | ❌ | ❌ | ❌ | ✅ |
| Reproducible builds | ❌ | ❌ | ❌ | ✅ |
| Permission manifests | Basic | ❌ | ✅ | ✅ |
| Capability sandbox | ❌ | ❌ | ✅ | ✅ |
| Continuous rebuilds | ❌ | ❌ | ❌ | ✅ |
| SLSA compliance | ❌ | ❌ | ❌ | ✅ |
| Production ready | ✅ | ✅ | ❌ | 🚧 |

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repo
git clone https://github.com/skillguard/skillguard.git
cd skillguard

# Install dependencies with uv
uv sync

# Run tests
uv run pytest

# Run linting
uv run ruff check .
```

---

## License

Apache 2.0 - See [LICENSE](LICENSE) for details.

---

## Acknowledgments

- [Chainguard](https://chainguard.dev) - Inspiration for supply chain security model
- [Sigstore](https://sigstore.dev) - Cryptographic signing infrastructure
- [SLSA](https://slsa.dev) - Supply chain security framework
- [Agent Skills](https://agentskills.io) - Skill format inspiration
- [OSSA](https://openstandardagents.org) - Agent specification work

---

<p align="center">
  <b>Secure skills for a safer agentic future.</b>
</p>
