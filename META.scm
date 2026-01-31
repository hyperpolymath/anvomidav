;; SPDX-License-Identifier: PMPL-1.0-or-later
;; META.scm - Project metadata and architectural decisions

(define project-meta
  `((version . "1.0.0")
    (architecture-decisions
      ((adr-001
         ((status . "accepted")
          (date . "2026-01-30")
          (title . "Use Rust for compiler implementation")
          (context . "Need safe, performant systems language with strong parsing libraries")
          (decision . "Use Rust with logos (lexer) and chumsky (parser)")
          (consequences . "Memory safety, excellent error handling, but steeper learning curve for contributors")))
       (adr-002
         ((status . "accepted")
          (date . "2026-01-30")
          (title . "ISU rules as standard library")
          (context . "Choreographers need validation against official competition rules")
          (decision . "Implement ISU Technical Panel guidelines as language stdlib with semantic analysis")
          (consequences . "Language is domain-specific but highly valuable to target audience")))
       (adr-003
         ((status . "accepted")
          (date . "2026-01-31")
          (title . "Enhanced error messages with ISU-specific hints")
          (context . "Users may not know all ISU rules, need educational error messages")
          (decision . "Add hint() method to SemanticError with rule explanations and suggestions")
          (consequences . "Better UX for beginners, language becomes teaching tool")))
       (adr-004
         ((status . "proposed")
          (date . "2026-01-31")
          (title . "No workers/concurrency for choreography DSL")
          (context . "Choreography is inherently sequential, elements happen in time order")
          (decision . "Mark workers as N/A for Green Tick, focus on sequential execution model")
          (consequences . "Simpler mental model, but no parallelism for future batch processing")))))
    (development-practices
      ((code-style . "rust-fmt")
       (security . "openssf-scorecard")
       (testing . "unit-and-integration")
       (versioning . "semver")
       (documentation . "asciidoc")
       (branching . "trunk-based")
       (parser-library . "chumsky-0.9")
       (lexer-library . "logos-0.14")
       (error-reporting . "miette-7")
       (cli-framework . "clap-4")))
    (design-rationale
      ((why-rust
         "Memory safety critical for user-facing tools, excellent parser combinator ecosystem")
       (why-chumsky
         "Parser combinator approach matches language structure naturally, excellent error recovery")
       (why-logos
         "Fast regex-based tokenization, compile-time optimization")
       (why-miette
         "Beautiful diagnostic output with source spans and hints")
       (why-domain-specific
         "Figure skating has formal notation needs unmet by general languages")
       (why-isu-stdlib
         "Rules validation is core value proposition, not an afterthought")
       (why-five-crates
         "Separation of concerns: core types, syntax, type checking, semantics, CLI")))))
