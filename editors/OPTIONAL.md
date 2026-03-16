<!-- SPDX-License-Identifier: MIT OR PMPL-1.0-or-later -->
<!-- SPDX-FileCopyrightText: 2025-2026 hyperpolymath -->

# Editor Integrations (OPTIONAL)

**Status: Optional - Does NOT block core build**

This directory contains editor-specific configurations for Anvomidav support.
These are provided for developer convenience but are **not required** for the
core language toolchain to function.

## Included Editors

| Directory | Editor | Status |
|-----------|--------|--------|
| `neovim/` | Neovim | Optional |
| `helix/` | Helix | Optional |
| `vscode/` | VS Code / VSCodium | Optional |
| `zed/` | Zed | Optional |

## Core Build Independence

The core Anvomidav toolchain (`cargo build` / `cargo test`) does not depend on
any editor configuration. You can:

- Build and run `anv-cli` without editor plugins
- Run all conformance tests without editor configuration
- Develop new language features using any text editor

## Maintenance Priority

During scope-arrest phases (f0), editor integration maintenance is deferred.
Focus is on core language semantics and ISU rule validation.

The LSP server (`anv-lsp`) is part of the Cargo workspace but is not required
for core functionality.

## See Also

- `tree-sitter-anvomidav/` - Tree-sitter grammar for syntax highlighting
- `crates/anv-lsp/` - Language Server Protocol implementation
- `crates/anv-cli/` - Core CLI tool (authoritative)
