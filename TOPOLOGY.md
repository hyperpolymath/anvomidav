<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-02-19 -->

# Anvomidav — Project Topology

## System Architecture

```
                        ┌─────────────────────────────────────────┐
                        │              CHOREOGRAPHER              │
                        │        (.anv Source Files / CLI)        │
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │           ANV-CLI (RUST)                │
                        │    (check, parse, lex, fmt, new)        │
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │           COMPILER PIPELINE             │
                        │                                         │
                        │  ┌───────────┐  ┌───────────────────┐  │
                        │  │anv-syntax │  │    anv-types      │  │
                        │  │ (Logos/   │  │ (Type Checking)   │  │
                        │  │ Chumsky)  │  │                   │  │
                        │  └─────┬─────┘  └────────┬──────────┘  │
                        │        │                 │              │
                        │        └────────┬────────┘              │
                        │                 ▼                       │
                        │        ┌────────────────┐               │
                        │        │ anv-semantics  │               │
                        │        │ (ISU Rules     │               │
                        │        │  Validation)   │               │
                        │        └────────┬────────┘              │
                        └─────────────────│───────────────────────┘
                                          │
                                          ▼
                        ┌─────────────────────────────────────────┐
                        │             ANV-CORE                    │
                        │    (Skating Types, Diagnostics)         │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          REPO INFRASTRUCTURE            │
                        │  .machine_readable/ (STATE.a2ml)        │
                        │  Cargo Workspace (Monorepo)             │
                        │  Test Suite (90+ Passing)               │
                        └─────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
COMPILER PIPELINE
  anv-syntax (Lexer/Parser)         ██████████ 100%    Logos/Chumsky stable
  anv-types (Type Checker)          ██████████ 100%    Domain types verified
  anv-semantics (ISU Rules)         ██████████ 100%    Singles/Pairs/Dance rules active
  anv-cli (clap 4)                  ██████████ 100%    Full command set verified

CORE & DISCIPLINES
  anv-core                          ██████████ 100%    Core skating types stable
  Singles Discipline                ██████████ 100%    Short & Free support
  Pairs Discipline                  ██████████ 100%    Lifts, twists, spirals active
  Ice Dance Discipline              ██████████ 100%    Pattern & Rhythm dance stable

REPO INFRASTRUCTURE
  miette Diagnostics                ██████████ 100%    Fancy error hints verified
  Justfile                          ██████████ 100%    Standard build automation
  .machine_readable/                ██████████ 100%    STATE.a2ml tracking

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            ██████████ 100%    Core language features complete
```

## Key Dependencies

```
anv-syntax ──────► anv-types ──────► anv-semantics ──────► anv-cli
     │               │                   │
     └───────────────┴────────┬──────────┴───────────────┐
                              ▼
                           anv-core
```

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
