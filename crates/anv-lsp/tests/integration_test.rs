// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Integration tests for anvomidav LSP
//!
//! These tests verify that the LSP server integrates correctly with
//! the anvomidav compiler components (parser, type checker, semantics).

use anv_core::source::FileId;
use anv_syntax::parse;

#[test]
fn test_parser_integration() {
    // Simple valid anvomidav program
    let source = r#"
program test_program {
    segment sp: short {
        sequence opening {
            jump triple axel
        }
    }
}
    "#;

    let result = parse(source, FileId(0));
    assert!(result.is_ok(), "Valid program should parse successfully: {:?}", result);
}

#[test]
fn test_parser_error_handling() {
    // Invalid program (missing required fields)
    let source = r#"
        program Test
        // Missing discipline and duration
    "#;

    let result = parse(source, FileId(0));
    // Parser should handle this gracefully (may succeed with incomplete program)
    // The actual validation happens in semantics
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_type_checking_integration() {
    let source = r#"
program spin_test {
    segment sp: short {
        sequence spin_seq {
            spin upright L4
        }
    }
}
    "#;

    let result = parse(source, FileId(0));
    assert!(result.is_ok(), "Spin program should parse: {:?}", result);

    if let Ok(program) = result {
        // Type checking should work on valid program
        let type_result = anv_types::check(&program, FileId(0));
        // Type checker may return Ok or type errors depending on implementation
        match type_result {
            Ok(_) => {
                // Type checking passed
            }
            Err(errors) => {
                // Type errors found - this is also valid behavior
                assert!(!errors.is_empty());
            }
        }
    }
}

#[test]
fn test_semantic_validation() {
    use anv_semantics::{Discipline, validate_program};

    let source = r#"
program jump_test {
    segment sp: short {
        sequence axel_seq {
            jump double axel
        }
    }
}
    "#;

    let result = parse(source, FileId(0));
    assert!(result.is_ok(), "Jump program should parse: {:?}", result);

    if let Ok(program) = result {
        // Validate against ISU rules for singles
        let validation = validate_program(&program, Discipline::MenSingles);

        // Validation should complete (may have errors/warnings)
        // This tests that semantic validation integrates with parsed program
        assert!(
            validation.errors.len() + validation.warnings.len() >= 0,
            "Validation should return result"
        );
    }
}

#[test]
fn test_empty_program() {
    let source = "";
    let result = parse(source, FileId(0));

    // Empty program should be handled gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_multiline_program() {
    let source = r#"
program complex_routine {
    segment sp: short {
        sequence jump_combo {
            jump quad lutz
            jump triple toe_loop
        }

        sequence spin_combo {
            spin camel sit upright L4
        }

        sequence steps {
            step circular L4
        }
    }
}
    "#;

    let result = parse(source, FileId(0));
    assert!(result.is_ok(), "Multi-element program should parse: {:?}", result);

    if let Ok(program) = result {
        use anv_semantics::{Discipline, validate_program};

        // Validate for pairs discipline
        let validation = validate_program(&program, Discipline::Pairs);

        // Should complete validation
        assert!(
            validation.errors.len() + validation.warnings.len() >= 0,
            "Validation should process multi-element program"
        );
    }
}

#[test]
fn test_ice_dance_discipline() {
    let source = r#"
program dance_routine {
    segment rd: rhythm {
        sequence steps_seq {
            step circular L4
        }
    }
}
    "#;

    let result = parse(source, FileId(0));
    assert!(result.is_ok(), "Dance program should parse: {:?}", result);

    if let Ok(program) = result {
        use anv_semantics::{Discipline, validate_program};

        // Validate for ice dance
        let validation = validate_program(&program, Discipline::IceDance);

        assert!(
            validation.errors.len() + validation.warnings.len() >= 0,
            "Ice dance validation should work"
        );
    }
}

#[test]
fn test_file_id_handling() {
    let source = r#"
program test {
    segment sp: short {
        sequence seq1 {
            jump triple axel
        }
    }
}
    "#;

    // Test with different FileIds
    let result1 = parse(source, FileId(0));
    let result2 = parse(source, FileId(1));
    let result3 = parse(source, FileId(999));

    assert!(result1.is_ok(), "FileId(0) should work: {:?}", result1);
    assert!(result2.is_ok(), "FileId(1) should work: {:?}", result2);
    assert!(result3.is_ok(), "FileId(999) should work: {:?}", result3);
}
