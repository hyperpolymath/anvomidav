<!-- SPDX-License-Identifier: MIT OR AGPL-3.0-or-later -->
<!-- SPDX-FileCopyrightText: 2025-2026 hyperpolymath -->

# Conformance Test Corpus

This directory contains the **binding** conformance test corpus for Anvomidav.
These tests define authoritative behavior for the language.

## Structure

```
conformance/
├── valid/      # Programs that MUST parse and validate without errors
└── invalid/    # Programs that MUST produce specific diagnostics
```

## Usage

### Validate All Valid Examples
```bash
cargo run -p anv-cli -- check conformance/valid/*
```
All files in `valid/` must succeed without errors.

### Validate Invalid Examples
```bash
for f in conformance/invalid/*.anv; do
  echo "=== $f ==="
  cargo run -p anv-cli -- check "$f" 2>&1
done
```
Each file in `invalid/` must produce a diagnostic. The diagnostics should be
stable across runs (same error message structure).

## Adding New Tests

### Valid Examples
- Add to `valid/` directory
- Must represent legal Anvomidav programs
- Should cover different disciplines and element types
- Name descriptively: `singles_minimal.anv`, `pairs_lift_types.anv`

### Invalid Examples
- Add to `invalid/` directory
- Each file should trigger ONE specific error type
- Name should indicate the error: `err_missing_program.anv`
- Document the expected error in a comment at the top

## Stability Requirement

Conformance tests are **binding**. Once added:
- Valid tests must remain valid across all future versions
- Invalid tests must continue to produce diagnostics
- Error messages should remain structurally stable

Breaking changes require version bump and migration documentation.
