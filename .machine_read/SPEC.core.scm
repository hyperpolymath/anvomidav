;; SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
;; SPDX-FileCopyrightText: 2025-2026 hyperpolymath
;;
;; SPEC.core.scm - Core Language Specification (Binding)
;;
;; This specification defines the authoritative syntax and semantics for Anvomidav.
;; The CLI implementation in Rust is the reference behavior.
;; Conformance tests in conformance/ are binding test cases.

(define core-spec
  '((schema . "hyperpolymath.spec/1")
    (version . "0.1.0")
    (status . "draft-binding")

    ;; ==========================================================================
    ;; 1. LEXICAL STRUCTURE
    ;; ==========================================================================
    (lexical
      . ((keywords
           . ((program-kw . "program")
              (segment-kw . "segment")
              (sequence-kw . "sequence")
              (jump-kw . "jump")
              (spin-kw . "spin")
              (step-kw . "step")
              (lift-kw . "lift")
              (throw-kw . "throw")
              (twist-kw . "twist")
              (death-spiral-kw . "death_spiral")
              (choreographic-kw . "choreographic")
              (pattern-kw . "pattern")
              (sync-kw . "sync")
              (parallel-kw . "parallel")))

         (rotations
           . ((single . "single")
              (double . "double")
              (triple . "triple")
              (quad . "quad")))

         (jump-kinds
           . ((axel . "axel")
              (lutz . "lutz")
              (flip . "flip")
              (loop . "loop")
              (salchow . "salchow")
              (toe-loop . "toe_loop")
              (euler . "euler")))

         (spin-positions
           . ((upright . "upright")
              (sit . "sit")
              (camel . "camel")
              (layback . "layback")
              (biellmann . "biellmann")))

         (step-patterns
           . ((straight . "straight")
              (circular . "circular")
              (serpentine . "serpentine")))

         (levels
           . ((base . "B")
              (level-1 . "L1")
              (level-2 . "L2")
              (level-3 . "L3")
              (level-4 . "L4")))

         (lift-groups
           . ((group-1 . "Gr1")
              (group-2 . "Gr2")
              (group-3 . "Gr3")
              (group-4 . "Gr4")
              (group-5 . "Gr5")))

         (edges
           . ((lfo . "LFO") (lfi . "LFI") (lbo . "LBO") (lbi . "LBI")
              (rfo . "RFO") (rfi . "RFI") (rbo . "RBO") (rbi . "RBI")))

         (segment-types
           . ((short . "short")
              (free . "free")
              (rhythm . "rhythm")
              (pattern-dance . "pattern")
              (exhibition . "exhibition")))

         (choreographic-kinds
           . ((spiral . "spiral")
              (spread . "spread")
              (ina . "ina")
              (hydroblading . "hydroblading")
              (pivot . "pivot")))))

    ;; ==========================================================================
    ;; 2. GRAMMAR (EBNF-like)
    ;; ==========================================================================
    (grammar
      . ((program     . "program IDENT { segment+ }")
         (segment     . "segment IDENT : SEGMENT_TYPE { sequence+ }")
         (sequence    . "sequence IDENT { element+ }")
         (element     . "jump-elem | spin-elem | step-elem | lift-elem | throw-elem | twist-elem | death-spiral-elem | choreographic-elem | pattern-elem | sync-block")
         (jump-elem   . "jump ROTATION JUMP_KIND")
         (spin-elem   . "spin SPIN_POSITION+ LEVEL")
         (step-elem   . "step STEP_PATTERN LEVEL")
         (lift-elem   . "lift LIFT_GROUP LEVEL")
         (throw-elem  . "throw ROTATION JUMP_KIND")
         (twist-elem  . "twist ROTATION LEVEL")
         (death-spiral-elem . "death_spiral EDGE LEVEL")
         (choreographic-elem . "choreographic CHOREO_KIND")
         (pattern-elem . "pattern DANCE_NAME")
         (sync-block  . "sync { element+ }")))

    ;; ==========================================================================
    ;; 3. DISCIPLINES AND CONSTRAINTS
    ;; ==========================================================================
    (disciplines
      . ((singles
           . ((description . "Men's and Ladies' singles skating")
              (allowed-elements . (jump spin step choreographic))
              (forbidden-elements . (lift throw twist death-spiral))))

         (pairs
           . ((description . "Pairs skating")
              (allowed-elements . (jump spin step lift throw twist death-spiral choreographic sync))))

         (ice-dance
           . ((description . "Ice dance")
              (allowed-elements . (spin step choreographic pattern))
              (jump-constraint . "only single jumps allowed")
              (forbidden-elements . (throw twist death-spiral))))))

    ;; ==========================================================================
    ;; 4. SEGMENT LIMITS (ISU Technical Rules)
    ;; ==========================================================================
    (segment-limits
      . ((singles-short
           . ((max-jumps . 3)
              (max-spins . 3)
              (max-steps . 1)))
         (singles-free
           . ((max-jumps . 7)
              (max-spins . 3)
              (max-steps . 1)))
         (pairs-short
           . ((required-lift . 1)
              (required-throw . 1)
              (required-twist . 1)
              (required-death-spiral . 1)))
         (ice-dance-rhythm
           . ((max-jumps . 1)
              (jump-constraint . "single only")))))

    ;; ==========================================================================
    ;; 5. SEMANTIC INVARIANTS
    ;; ==========================================================================
    (invariants
      . (("Each program must have at least one segment")
         ("Each segment must have at least one sequence")
         ("Each sequence must have at least one element")
         ("Jump rotations must be single, double, triple, or quad")
         ("Spin levels must be B, L1, L2, L3, or L4")
         ("Death spiral edges must be valid (LFO, LBI, RFO, RBI, etc.)")
         ("Discipline constraints are enforced at semantic analysis")))))
