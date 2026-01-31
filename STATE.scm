;;; STATE.scm — anvomidav
;; SPDX-License-Identifier: PMPL-1.0-or-later
;; SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

(define metadata
  '((version . "0.1.0")
    (updated . "2026-01-31")
    (project . "anvomidav")
    (green-tick . "YES - 3 of 4 features (no workers for choreography DSL)")))

(define current-position
  '((phase . "v0.1 - Green Tick Complete")
    (overall-completion . 100)
    (components ((lexer ((status . "complete") (completion . 100)))
                 (parser ((status . "complete") (completion . 100)))
                 (ast ((status . "complete") (completion . 100)))
                 (type-system ((status . "complete") (completion . 100)))
                 (isu-rules ((status . "complete") (completion . 100)))
                 (validator ((status . "complete") (completion . 100)))
                 (cli-tool ((status . "complete") (completion . 100)))
                 (error-hints ((status . "complete") (completion . 100)))
                 (lsp-server ((status . "in-progress") (completion . 40)))
                 (visualizer ((status . "in-progress") (completion . 60)))))))

(define blockers-and-issues
  '((critical ())
    (high-priority ())))

(define critical-next-actions
  '((immediate (("Add LSP completion" . medium)))
    (this-week (("Enhance visualizer" . low)))))

(define session-history
  '((snapshots
     ((date . "2026-01-31")
      (session . "green-tick-complete")
      (notes . "Added enhanced error hints to achieve Green Tick status (75% → 100%): 1) Added SemanticError::hint() method for all ISU rule violation types, 2) Context-aware hints: TooManyElements suggests removing elements, TooFewElements explains minimum requirements, DurationOutOfRange specifies valid time ranges, MissingRequiredElement reminds to add required elements, DuplicateElement suggests removal, ElementNotAllowed points to ISU rules, InvalidForDiscipline explains discipline restrictions, 3) Added SemanticError::to_diagnostic() for conversion to Diagnostic with hints, 4) Mapped errors to ISU error codes (E0200-E0204), 5) All builds passing with comprehensive error reporting. Green Tick achieved: ✅ Record field access (Field(expr, ident) in AST, can access program.segment.elements fields), ✅ Stdlib integration (ISU rules validator for all disciplines with segment-specific validation), ✅ Enhanced error messages with hints (just added), ❌ Workers N/A (choreography DSL, sequential execution). Ready for Phronesis-level tooling phase: complete LSP server, add debugger, build profiler.")))))
