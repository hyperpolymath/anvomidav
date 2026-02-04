;; SPDX-License-Identifier: PMPL-1.0-or-later
;; SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
;; ECOSYSTEM.scm - Project relationship mapping

(ecosystem
  (version "1.0")
  (name "anvomidav")
  (type "domain-specific-language")
  (purpose "Figure skating choreography language for Olympic-level routine design and analysis, created for the Russian Olympic figure skating team (Betina Popova)")

  (position-in-ecosystem
    (category "Domain-Specific Languages")
    (subcategory "Sports Choreography / Figure Skating")
    (unique-value
      ("First formal language for Olympic-level figure skating choreography"
       "Precise notation for jumps (axel, salchow, lutz, toe loop)"
       "Grade of Execution (GoE) and base value calculation"
       "Element level tracking and sequence validation")))

  (related-projects
    ((robodog-ecm
      (relationship "explicitly-separate")
      (description "MUST remain separate - military ECM project incompatible with Russian collaboration due to export control"))
     (robot-vacuum-cleaner
      (relationship "inspiration")
      (description "Formation control concepts inspired choreographed movements, but no code sharing"))))

  (what-this-is
    ("Figure skating choreography notation language"
     "Olympic routine design and validation tool"
     "Performance analysis system for GoE/base value calculation"
     "Civilian sports technology (export-safe)"))

  (what-this-is-not
    ("NOT a military application"
     "NOT related to drone/avionics (despite algorithm similarity)"
     "NOT a dual-use technology"
     "NOT subject to export control restrictions")))
