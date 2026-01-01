;; SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
;; SPDX-FileCopyrightText: 2025-2026 hyperpolymath
;;
;; ROADMAP.f0.scm - Phase f0: Scope Arrest & Baseline Establishment
;;
;; This roadmap defines the immediate stabilization phase.
;; Focus: Make the repository unambiguously runnable and freeze identity.

(define roadmap-f0
  '((schema . "hyperpolymath.roadmap/1")
    (phase . "f0")
    (name . "Scope Arrest")
    (goal . "Establish stable, runnable baseline with conformance corpus")
    (started . "2026-01-01")
    (target-tier . "bronze")

    ;; ==========================================================================
    ;; PHASE F0 OBJECTIVES
    ;; ==========================================================================
    (objectives
      . (("O1" . "Repository is unambiguously runnable")
         ("O2" . "Identity is frozen and documented")
         ("O3" . "Conformance corpus exists with valid/invalid examples")
         ("O4" . "Tree-sitter/editor integration is marked optional")
         ("O5" . "Smoke test passes reliably")))

    ;; ==========================================================================
    ;; ACCEPTANCE CRITERIA
    ;; ==========================================================================
    (acceptance-criteria
      . ((smoke-test
           . ((command . "cargo test && cargo run -p anv-cli -- check examples/*")
              (expected . "All tests pass, at least 1 example parses successfully")))
         (conformance-valid
           . ((command . "cargo run -p anv-cli -- check conformance/valid/*")
              (expected . "All valid examples parse and validate without errors")))
         (conformance-invalid
           . ((command . "for f in conformance/invalid/*; do cargo run -p anv-cli -- check $f 2>&1; done")
              (expected . "Each invalid example produces stable diagnostic output")
              (count . ">= 5 invalid examples")))))

    ;; ==========================================================================
    ;; TASKS (CHECKLIST)
    ;; ==========================================================================
    (tasks
      . ((done
           . (("Create .machine_read directory")
              ("Add LLM_SUPERINTENDENT.scm")
              ("Add SPEC.core.scm")
              ("Add ROADMAP.f0.scm")
              ("Add CANONICAL_README.scm")
              ("Fix STATE.scm language reference")
              ("Add conformance/valid/ examples (4 files)")
              ("Add conformance/invalid/ examples (7 files, exceeds >= 5)")
              ("Mark tree-sitter as optional (OPTIONAL.md)")
              ("Mark editors as optional (OPTIONAL.md)")
              ("Verify smoke test passes (152 tests)")
              ("Verify conformance tests pass")))
         (pending
           . ())))

    ;; ==========================================================================
    ;; SCOPE BOUNDARIES
    ;; ==========================================================================
    (scope
      . ((in-scope
           . (("Bug fixes in core crates")
              ("Conformance test additions")
              ("Documentation corrections")
              ("Error message improvements")))
         (out-of-scope
           . (("New language features")
              ("New crates or binaries")
              ("Editor/IDE integration work")
              ("Visualization features")
              ("IR layer implementation")))))

    ;; ==========================================================================
    ;; SUCCESS METRICS
    ;; ==========================================================================
    (success-metrics
      . ((tests-passing . ">= 90")
         (valid-conformance-files . ">= 3")
         (invalid-conformance-files . ">= 5")
         (smoke-test-reliable . #t)))

    ;; ==========================================================================
    ;; NEXT PHASE
    ;; ==========================================================================
    (next-phase
      . ((name . "f1")
         (focus . "IR layer and code generation")
         (prerequisite . "f0 acceptance criteria met")
         (target-tier . "silver")))))
