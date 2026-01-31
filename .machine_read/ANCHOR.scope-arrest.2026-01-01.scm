;; ANCHOR.scope-arrest.2026-01-01.Jewell.scm  (anvomidav)
;; SPDX-License-Identifier: PMPL-1.0-or-later
;; SPDX-FileCopyrightText: 2026 hyperpolymath
;;
;; This anchor file defines the scope arrest for phase f0.
;; It freezes the project identity and establishes the conformance baseline.

(define anchor
  '((schema . "hyperpolymath.anchor/1")
    (repo . "hyperpolymath/anvomidav")
    (date . "2026-01-01")
    (authority . "repo-superintendent")
    (purpose . ("Scope arrest + make repo unambiguously runnable + freeze identity."))
    (identity
      . ((project . "Anvomidav")
         (kind . "domain-language + CLI toolchain")
         (domain . "figure-skating choreography")
         (one-sentence . "A DSL and toolchain for expressing skating choreography and validating rule constraints.")))

    (semantic-anchor
      . ((policy . "dual")
         (reference-impl . ("Rust workspace" "CLI is authoritative behavior"))
         (formal-spec . ("SPEC.core.scm defines syntax/semantics; conformance corpus is binding"))))

    (allowed-implementation-languages
      . ("Rust")) ;; keep narrow in f0; editors/tooling optional later
    (forbidden
      . ("Expanding into unrelated domains"
         "Adding new backends"
         "Rewriting as a different project"))

    (golden-path
      . ((smoke-test-command . "cargo test && cargo run -p anv-cli -- check examples/*")
         (success-criteria . ("parse+check succeeds on at least 1 example"
                              "at least 5 invalid examples produce stable diagnostics"))))

    (mandatory-files
      . ("./.machine_read/LLM_SUPERINTENDENT.scm"
         "./.machine_read/SPEC.core.scm"
         "./.machine_read/ROADMAP.f0.scm"
         "./conformance/"))

    (first-pass-directives
      . ("Ensure README(s) do not contradict this identity."
         "If multiple READMEs exist, create one canonical machine README pointer (no prose needed)."
         "Quarantine tree-sitter/editor integration as optional: it must not block core build."
         "Add conformance corpus representing core grammar + rule checks."))

    (rsr
      . ((target-tier . "bronze-now") (upgrade-path . "silver-after-f1")))))
