;; AUTHORITY_STACK.mustfile-nickel.scm
;; SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
;; SPDX-FileCopyrightText: 2026 hyperpolymath
;;
;; Shared drop for hyperpolymath repos: defines task routing + config authority.

(define authority-stack
  '((schema . "hyperpolymath.authority-stack/1")
    (intent
      . ("Stop agentic drift and toolchain creep."
         "Make the repo executable via a single blessed interface."
         "Prevent the LLM from inventing commands, tools, or files."))

    (operational-authority
      . ((local-tasks . "just")
         (deployment-transitions . "must")
         (config-manifests . "nickel")
         (container-engine . "podman-first")))

    (hard-rules
      . ("Makefiles are forbidden."
         "All operations must be invoked via `just <recipe>` (local) or `must <transition>` (deployment)."
         "If a recipe/transition does not exist, the correct action is to ADD it (and document it), not to run ad-hoc commands."
         "Nickel manifests are the single source of truth for config; do not hand-edit generated outputs."
         "No network-required runtime paths for demos/tests unless explicitly permitted in ANCHOR."))

    (workflow
      . ((first-run
           . ("Read ANCHOR*.scm"
              "Read STATE.scm"
              "Run: just --list"
              "Run: just test"
              "Run: just demo (if defined)"))
         (adding-new-capability
           . ("Update SPEC/ROADMAP first"
              "Add a `just` recipe (and tests) that implements the capability"
              "Only then edit code"))))

    (tooling-contract
      . ((mustfile-notes
           . ("Mustfile is the deployment contract (physical state transitions)."
              "must is the supervisor/enforcer for must-spec; it routes through just where appropriate."))
         (nickel-notes
           . ("Nickel provides validated, type-safe manifests."
              "Prefer .ncl for machine-truth; render docs from it via your conversion pipeline."))
         (shell-entrypoints
           . ("Shell wrappers may exist; all must route to just/must without inventing extra logic."))))))
