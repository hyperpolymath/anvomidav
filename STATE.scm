;; STATE.scm - RSR State File
;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Copyright (C) 2025 Jonathan D.A. Jewell
;;
;; This file tracks the current state of the project using S-expressions.
;; It is machine-readable and used by RSR tooling for validation.

(state
  (version . "0.1.0")
  (phase . "alpha")
  (updated . "2025-12-27T00:00:00Z")

  (project
    (name . "anvomidav")
    (tier . "infrastructure")
    (license . "MIT OR AGPL-3.0-or-later")
    (language . "rust"))

  (compliance
    (rsr . #t)
    (security-hardened . #t)
    (ci-cd . #t)
    (guix-primary . #f)
    (nix-fallback . #f))

  (artifacts
    (binary . "bin/anvomidav")
    (container . "ghcr.io/hyperpolymath/anvomidav:latest"))

  (dependencies
    (build
      ("gnat" . ">=12")
      ("gprbuild" . ">=22"))
    (runtime))

  (milestones
    (v0.1.0
      (status . "released")
      (date . "2025-12-27")
      (features
        "Task runner"
        "Template engine"
        "Requirements enforcer"
        "Deploy command"))
    (v0.2.0
      (status . "planned")
      (features
        "Mustache partials"
        "Content requirement checks"
        "TOML variable loading"))))
