;; SPDX-License-Identifier: PMPL-1.0-or-later
;; SPDX-FileCopyrightText: 2025-2026 hyperpolymath
;;
;; CANONICAL_README.scm - Pointer to Canonical Documentation
;;
;; This file establishes the documentation hierarchy for the repository.
;; Multiple READMEs exist for different purposes; this file clarifies authority.

(define canonical-docs
  '((schema . "hyperpolymath.canonical-docs/1")
    (updated . "2026-01-01")

    ;; ==========================================================================
    ;; PRIMARY DOCUMENTATION
    ;; ==========================================================================
    (primary
      . ((readme
           . ((path . "./README.adoc")
              (purpose . "Main project documentation, quick start, syntax examples")
              (authority . "canonical")))
         (roadmap
           . ((path . "./ROADMAP.adoc")
              (purpose . "Development phases and milestones")
              (authority . "canonical")))
         (contributing
           . ((path . "./CONTRIBUTING.adoc")
              (purpose . "How to contribute")
              (authority . "canonical")))
         (security
           . ((path . "./SECURITY.md")
              (purpose . "Security policy and vulnerability reporting")
              (authority . "canonical")))))

    ;; ==========================================================================
    ;; MACHINE-READABLE DOCUMENTATION
    ;; ==========================================================================
    (machine-readable
      . ((superintendent
           . ((path . "./.machine_read/LLM_SUPERINTENDENT.scm")
              (purpose . "Instructions for AI/LLM agents")))
         (spec
           . ((path . "./.machine_read/SPEC.core.scm")
              (purpose . "Core language specification (binding)")))
         (roadmap-f0
           . ((path . "./.machine_read/ROADMAP.f0.scm")
              (purpose . "Phase f0 scope arrest roadmap")))
         (state
           . ((path . "./STATE.scm")
              (purpose . "Current project state")))))

    ;; ==========================================================================
    ;; SUPPLEMENTARY DOCUMENTATION (Non-authoritative for Core)
    ;; ==========================================================================
    (supplementary
      . ((academic
           . ((path . "./docs/academic/README.adoc")
              (purpose . "Academic papers and formal proofs")
              (status . "optional")))
         (implementation
           . ((path . "./docs/IMPLEMENTATION.adoc")
              (purpose . "Implementation details")
              (status . "optional")))))

    ;; ==========================================================================
    ;; EDITOR/TOOLING DOCUMENTATION (OPTIONAL - Does not affect core)
    ;; ==========================================================================
    (optional-tooling
      . ((neovim
           . ((path . "./editors/neovim/README.md")
              (purpose . "Neovim integration")
              (status . "optional - does not block core build")))
         (helix
           . ((path . "./editors/helix/README.md")
              (purpose . "Helix editor integration")
              (status . "optional - does not block core build")))
         (tree-sitter
           . ((path . "./tree-sitter-anvomidav/")
              (purpose . "Tree-sitter grammar for syntax highlighting")
              (status . "optional - does not block core build")))))

    ;; ==========================================================================
    ;; READING ORDER FOR NEW CONTRIBUTORS
    ;; ==========================================================================
    (reading-order
      . (("README.adoc" . "Start here for project overview")
         ("ROADMAP.adoc" . "Understand development phases")
         ("CONTRIBUTING.adoc" . "How to participate")
         (".machine_read/SPEC.core.scm" . "Language specification")
         ("crates/anv-cli/src/main.rs" . "CLI entry point")))))
