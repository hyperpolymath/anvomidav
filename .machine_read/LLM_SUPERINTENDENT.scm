;; SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
;; SPDX-FileCopyrightText: 2025-2026 hyperpolymath
;;
;; LLM_SUPERINTENDENT.scm - Instructions for AI/LLM agents operating on this repository
;;
;; This file provides machine-readable directives for LLM code assistants.

(define superintendent
  '((schema . "hyperpolymath.superintendent/1")
    (updated . "2026-01-01")

    ;; Project identity - DO NOT DEVIATE
    (identity
      . ((name . "Anvomidav")
         (kind . "domain-specific language + CLI toolchain")
         (domain . "figure skating choreography")
         (one-sentence . "A DSL and toolchain for expressing skating choreography and validating ISU rule constraints.")))

    ;; Implementation constraints
    (implementation
      . ((language . "Rust")
         (workspace . "Cargo.toml defines workspace members")
         (cli-binary . "anv-cli")
         (reference-behavior . "CLI output is authoritative")))

    ;; What to protect
    (invariants
      . (("Core semantics must match SPEC.core.scm")
         ("ISU rule validation must pass for valid examples in conformance/valid/")
         ("Invalid examples in conformance/invalid/ must produce stable diagnostics")
         ("All 90+ tests must pass after any change")))

    ;; What is allowed
    (allowed
      . (("Bug fixes in existing crates")
         ("Extending ISU rule coverage")
         ("Adding new conformance tests")
         ("Improving error messages")
         ("Documentation improvements")))

    ;; What is forbidden
    (forbidden
      . (("Changing project identity or domain")
         ("Adding non-Rust implementation languages")
         ("Adding external service dependencies")
         ("Rewriting the parser without migration plan")
         ("Breaking existing conformance tests")))

    ;; Before making changes, verify
    (verification
      . ((smoke-test . "cargo test && cargo run -p anv-cli -- check examples/*")
         (conformance . "cargo run -p anv-cli -- check conformance/valid/*")))

    ;; Key files to understand the system
    (key-files
      . (("README.adoc" . "Project overview and quick start")
         ("ROADMAP.adoc" . "Development phases and milestones")
         ("Cargo.toml" . "Workspace configuration")
         ("crates/anv-syntax/src/lib.rs" . "Lexer and parser entry")
         ("crates/anv-semantics/src/lib.rs" . "ISU rules validation")
         ("crates/anv-cli/src/main.rs" . "CLI entry point")
         (".machine_read/SPEC.core.scm" . "Core language specification")))

    ;; Current phase and scope limits
    (phase
      . ((current . "f0")
         (focus . "Make repo unambiguously runnable and establish conformance baseline")
         (scope-arrest . #t)
         (no-new-features . #t)))))
