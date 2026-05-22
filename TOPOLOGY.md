<!-- SPDX-License-Identifier: MIT OR MPL-2.0 -->
<!-- Copyright (c) 2024-2025 hyperpolymath -->

# TOPOLOGY.md — Anvomidav

## Purpose

The first domain-specific language for figure skating choreography. Enables choreographers to compose movement sequences, timing, and formations without learning general-purpose programming. Project is in concept phase with infrastructure and governance documents; no compiler implementation yet.

## Module Map

```
anvomidav/
├── ROADMAP.adoc               # Development phases, milestones, design decisions
├── spec/
│   └── ... (Language specification sketches)
├── docs/
│   └── ... (Choreography DSL semantics)
└── .github/workflows/
    └── ... (Standards enforcement, Scorecard, security)
```

## Data Flow

```
[Figure Skating Notation] ──► [Anvomidav Compiler (planned)] ──► [Timing/Movement Events]
                                                                      ↓
                                                            [Animation System Integration]
```

## Key Invariants

- DSL semantics grounded in ice skating physics and choreographic conventions
- Compiler not yet implemented; specification phase ongoing
- Target: make choreography as expressive and accessible as text-based programming
