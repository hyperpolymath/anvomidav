<!--
SPDX-License-Identifier: CC-BY-SA-4.0 OR MPL-2.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->

[![OpenSSF Best Practices](https://img.shields.io/badge/OpenSSF-Best_Practices-green?logo=opensourcesecurity)](https://www.bestpractices.dev/en/projects/new?repo_url=https://github.com/hyperpolymath/anvomidav)

**The first programming language for choreographers of figure skating.**

# Status

> [!IMPORTANT]
> This project is in the **concept phase**. The repository contains
> project infrastructure and governance documents, but no implementation
> yet. See <a href="ROADMAP.adoc" class="adoc">ROADMAP</a> for planned
> development.

# Vision

Anvomidav aims to provide figure skating choreographers with a
domain-specific language (DSL) to:

- **Notate** — Precisely describe skating elements, transitions, and
  sequences

- **Compose** — Build complex programs from reusable choreographic
  patterns

- **Validate** — Check technical compliance with ISU (International
  Skating Union) rules

- **Visualize** — Generate rink diagrams, timing charts, and 3D previews

- **Collaborate** — Share and version-control choreographic works

# Name

*Anvomidav* — etymology and meaning to be documented.

# Technology Stack

Per the [Hyperpolymath Standard](.claude/CLAUDE.md):

| Component            | Technology                    |
|----------------------|-------------------------------|
| Compiler/Interpreter | OCaml or Rust                 |
| Runtime              | Deno (if JS target) or native |
| Editor Integration   | LSP server (Rust)             |
| Visualization        | ReScript + WebGL/Canvas       |
| Mobile Apps          | Tauri 2.0+ or Dioxus          |

# Getting Started

```bash
# Clone the repository
git clone https://github.com/hyperpolymath/anvomidav.git
cd anvomidav

# Development environment (once implemented)
nix develop        # Nix users
# or
guix shell         # Guix users
```

# Documentation

- [Roadmap](ROADMAP.adoc) — Development phases and milestones

- [Contributing](CONTRIBUTING.md) — How to participate

- [Security Policy](SECURITY.md) — Vulnerability reporting

- [Code of Conduct](CODE_OF_CONDUCT.md) — Community standards

# License

Dual-licensed under [MIT OR MPL-2.0](LICENSE.txt). Choose the license
that best fits your use case.

# Contributing

Contributions welcome! This project is in early stages — input on
language design, figure skating domain expertise, and implementation
help are all valuable.

See <a href="CONTRIBUTING.md" class="md">CONTRIBUTING</a> for
guidelines.
