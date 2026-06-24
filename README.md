[![Sponsor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-pink?logo=github)](https://github.com/sponsors/hyperpolymath)

// SPDX-License-Identifier: CC-BY-SA-4.0 OR MPL-2.0
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

= Anvomidav

image:https://img.shields.io/badge/OpenSSF-Best_Practices-green?logo=opensourcesecurity[OpenSSF Best Practices, link="https://www.bestpractices.dev/en/projects/new?repo_url=https://github.com/hyperpolymath/anvomidav"]

**The first programming language for choreographers of figure skating.**

== Status

[IMPORTANT]
====
This project is in the *concept phase*. The repository contains project infrastructure and governance documents, but no implementation yet. See link:ROADMAP.adoc[ROADMAP.adoc] for planned development.
====

== Vision

Anvomidav aims to provide figure skating choreographers with a domain-specific language (DSL) to:

* **Notate** — Precisely describe skating elements, transitions, and sequences
* **Compose** — Build complex programs from reusable choreographic patterns
* **Validate** — Check technical compliance with ISU (International Skating Union) rules
* **Visualize** — Generate rink diagrams, timing charts, and 3D previews
* **Collaborate** — Share and version-control choreographic works

== Name

_Anvomidav_ — etymology and meaning to be documented.

== Technology Stack

Per the link:.claude/CLAUDE.md[Hyperpolymath Standard]:

[cols="1,2"]
|===
| Component | Technology

| Compiler/Interpreter
| OCaml or Rust

| Runtime
| Deno (if JS target) or native

| Editor Integration
| LSP server (Rust)

| Visualization
| ReScript + WebGL/Canvas

| Mobile Apps
| Tauri 2.0+ or Dioxus
|===

== Getting Started

[source,bash]
----
# Clone the repository
git clone https://github.com/hyperpolymath/anvomidav.git
cd anvomidav

# Development environment (once implemented)
nix develop        # Nix users
# or
guix shell         # Guix users
----

== Documentation

* link:ROADMAP.adoc[Roadmap] — Development phases and milestones
* link:CONTRIBUTING.md[Contributing] — How to participate
* link:SECURITY.md[Security Policy] — Vulnerability reporting
* link:CODE_OF_CONDUCT.md[Code of Conduct] — Community standards

== License

Dual-licensed under link:LICENSE.txt[MIT OR MPL-2.0]. Choose the license that best fits your use case.

== Contributing

Contributions welcome! This project is in early stages — input on language design, figure skating domain expertise, and implementation help are all valuable.

See link:CONTRIBUTING.md[CONTRIBUTING.md] for guidelines.
