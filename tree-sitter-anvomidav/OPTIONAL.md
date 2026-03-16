<!-- SPDX-License-Identifier: MIT OR PMPL-1.0-or-later -->
<!-- SPDX-FileCopyrightText: 2025-2026 hyperpolymath -->

# Tree-sitter Grammar (OPTIONAL)

**Status: Optional - Does NOT block core build**

This directory contains the tree-sitter grammar for Anvomidav syntax highlighting.
It is provided for editor integration convenience but is **not required** for the
core language toolchain to function.

## Core Build Independence

The core Anvomidav toolchain (`cargo build` / `cargo test`) does not depend on
tree-sitter. You can:

- Build and run `anv-cli` without tree-sitter
- Run all conformance tests without tree-sitter
- Develop new language features without tree-sitter

## When to Use

Use the tree-sitter grammar when you want:

- Syntax highlighting in editors (Neovim, Helix, VS Code, etc.)
- Fast incremental parsing for editor features
- Structural code navigation

## Maintenance Priority

During scope-arrest phases (f0), tree-sitter maintenance is deferred.
Focus is on core language semantics and ISU rule validation.

## See Also

- `editors/` - Editor-specific configurations
- `crates/anv-syntax/` - Authoritative lexer/parser (Rust)
