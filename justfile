# SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2025-2026 hyperpolymath
#
# justfile - Local task runner for Anvomidav
#
# Per AUTHORITY_STACK: All local operations must be invoked via `just <recipe>`.
# Do not run ad-hoc commands; add recipes here instead.

# Default recipe: show available commands
default:
    @just --list

# === BUILD & TEST ===

# Run all tests
test:
    cargo test

# Build all crates (debug)
build:
    cargo build

# Build all crates (release)
build-release:
    cargo build --release

# Check code without building
check:
    cargo check

# Run clippy lints
lint:
    cargo clippy --all-targets --all-features

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# === CLI OPERATIONS ===

# Check example files with anv-cli
check-examples:
    cargo run -p anv-cli -- check examples/*

# Check conformance valid examples
check-conformance-valid:
    cargo run -p anv-cli -- check conformance/valid/*

# Check that conformance invalid examples produce errors
check-conformance-invalid:
    #!/usr/bin/env bash
    set -e
    echo "Checking invalid conformance examples produce errors..."
    errors=0
    for f in conformance/invalid/*.anv; do
        if cargo run -q -p anv-cli -- check "$f" 2>/dev/null; then
            echo "FAIL: $f should have produced an error"
            errors=$((errors+1))
        else
            echo "OK: $f produced expected error"
        fi
    done
    if [ $errors -gt 0 ]; then
        echo "FAILED: $errors files did not produce expected errors"
        exit 1
    fi
    echo "All invalid examples produced expected errors"

# Run full conformance suite
conformance: check-conformance-valid check-conformance-invalid

# === DEMO ===

# Demo: parse and show AST for a singles program
demo:
    @echo "=== Anvomidav Demo ==="
    @echo ""
    @echo "Parsing conformance/valid/singles_minimal.anv..."
    cargo run -q -p anv-cli -- parse conformance/valid/singles_minimal.anv
    @echo ""
    @echo "Checking all valid conformance examples..."
    cargo run -q -p anv-cli -- check conformance/valid/*
    @echo "All examples valid!"

# === SMOKE TEST (Golden Path) ===

# Run the golden-path smoke test per ANCHOR
smoke-test: test check-examples
    @echo "Smoke test passed!"

# === CLEAN ===

# Clean build artifacts
clean:
    cargo clean

# === INSTALL ===

# Install anv-cli locally
install:
    cargo install --path crates/anv-cli

# === DEVELOPMENT ===

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Run specific crate tests
test-crate crate:
    cargo test -p {{crate}}

# Synchronize A2ML metadata to SCM (Shadow Sync)
sync-metadata:
    #!/usr/bin/env bash
    echo "Synchronizing metadata (A2ML -> SCM)..."
    if [ -f .machine_readable/STATE.a2ml ]; then
        COMPLETION=$(grep "completion-percentage:" .machine_readable/STATE.a2ml | awk '{print $2}')
        sed -i "s/(overall-completion [0-9]\+)/(overall-completion $COMPLETION)/" .machine_readable/STATE.scm
        echo "✓ Metadata synchronized"
    fi
