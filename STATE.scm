;; SPDX-License-Identifier: PMPL-1.0-or-later
;; STATE.scm - Current project state

(define state
  '((metadata
     (version "1.0")
     (schema-version "1.0")
     (created "2026-02-04")
     (updated "2026-02-06T2")
     (project "anvomidav")
     (repo "hyperpolymath/anvomidav"))

    (project-context
     (name "Anvomidav")
     (tagline "Domain-specific language for Olympic-level figure skating choreography")
     (tech-stack ("rust")))

    (current-position
     (phase "developer-experience")
     (overall-completion 62)
     (components
       (("anv-core" "Core skating types (Edge, JumpKind, SpinPosition, Level)" 100)
        ("anv-syntax" "Lexer (logos) + Parser (chumsky) + AST" 100)
        ("anv-types" "Type checking and inference engine" 100)
        ("anv-semantics" "ISU rules validation engine" 100)
        ("anv-ir" "Choreography IR: timeline, placed elements, transitions, paths, phases" 80)
        ("anv-viz" "Visualization framework" 10)
        ("anv-cli" "CLI tool (1092 LOC)" 100)
        ("anv-lsp" "LSP server (10 files)" 100)
        ("documentation" "Tutorials and guides" 20)
        ("visualization" "2D rink diagrams, timing charts, 3D preview" 0)
        ("ecosystem" "Package manager, collaboration tools" 0)))
     (working-features
       ("Full lexer: all skating keywords, rotation prefixes, level designations, edge notation, lift groups"
        "Complete parser with error recovery"
        "Type checking and validation"
        "ISU rules semantic validation engine"
        "  - Discipline-specific rules (Singles, Pairs, Ice Dance)"
        "  - Element count validation (max jumps, required spins, step sequences)"
        "  - Segment-specific rules (short program vs free skate limits)"
        "  - Enhanced error hints for ISU rule violations"
        "LSP server: completion, diagnostics, hover with ISU codes, go-to-definition, formatting"
        "CLI: check, parse, lex, fmt, new (with singles/pairs/ice-dance templates)"
        "Choreography IR: two-pass lowering (AST->Timeline->Choreography)"
        "  - PlacedElement with spatial/temporal placement and phase decomposition"
        "  - Transition generation with ice paths (straight, arc, serpentine)"
        "  - Element phase breakdown (entry/execution/exit) for all element types"
        "  - Music synchronization map (BPM, tempo changes, cue points)"
        "  - Discipline inference, multi-skater support"
        "  - IcePath with waypoints, distance calculation, position interpolation"
        "  - Step sequence path generation per pattern type"
        "180+ tests, all passing (core:16, syntax:84, types:21, semantics:11, ir:38, lsp-integ:8, cli:2, viz:4)"
        "5 example programs: mens_short, ladies_free, pairs_short, ice_dance_rhythm, exhibition"
        "8-crate Rust workspace (~15,200 LOC across 40 files)")))

    (route-to-mvp
     (milestones
      ((milestone-id "m1")
       (name "Core Language")
       (status "complete")
       (completion 100)
       (items ("Core skating type definitions"
               "Lexer with all skating notation"
               "Parser with error recovery"
               "AST representation")))

      ((milestone-id "m2")
       (name "Validation Engine")
       (status "complete")
       (completion 100)
       (items ("Type checking engine"
               "ISU rules validation"
               "Discipline-specific constraints"
               "Segment-specific limits"
               "Enhanced error hints")))

      ((milestone-id "m3")
       (name "Developer Experience")
       (status "in-progress")
       (completion 60)
       (items ("LSP server (done)"
               "CLI tool (done)"
               "Example programs (done)"
               "Documentation and tutorials (TODO)"
               "VS Code extension packaging (TODO)")))

      ((milestone-id "m4")
       (name "Visualization")
       (status "not-started")
       (completion 0)
       (items ("2D rink diagrams with element placement"
               "Timing charts for program structure"
               "3D preview of choreography"
               "Animation export")))

      ((milestone-id "m5")
       (name "Ecosystem")
       (status "not-started")
       (completion 0)
       (items ("Package manager for choreography libraries"
               "Collaboration tools for coaches and choreographers"
               "Competition result integration"
               "Music synchronization")))))

    (blockers-and-issues
     (critical ())
     (high
       ("Visualization framework is scaffold only - needs to consume new Choreography IR"))
     (medium
       ("No documentation/tutorials for end users"
        "VS Code extension not packaged for marketplace"))
     (low
       ("Some license headers inconsistent (mix of PMPL/MIT/AGPL)")))

    (critical-next-actions
     (immediate
       ("Write user documentation and getting-started guide"
        "Package VS Code extension for distribution"))
     (this-week
       ("Connect anv-viz to new Choreography IR (rink diagrams with paths)"
        "Begin 2D rink visualization using IcePath data"))
     (this-month
       ("Complete visualization pipeline (rink diagrams, timing charts)"
        "Add music synchronization support"
        "Publish v1.0 with documentation")))

    (session-history
     ((date "2026-02-06")
      (accomplishments
        ("Updated STATE.scm with accurate project status from code audit")))
     ((date "2026-02-06T2")
      (accomplishments
        ("Implemented full Choreography IR layer (3 new modules: choreo, path, choreo_lower)"
         "choreo.rs: Choreography, PlacedElement, ElementPhase, Transition, MusicMap types"
         "path.rs: IcePath with waypoints, straight/arc/serpentine generators, interpolation"
         "choreo_lower.rs: Timeline->Choreography lowering with phase generation and transition paths"
         "22 new tests, all passing (38 total in anv-ir, 184 total workspace)"
         "Updated SPDX headers to PMPL-1.0-or-later"
         "anv-ir completion: 10% -> 80%"))))))
