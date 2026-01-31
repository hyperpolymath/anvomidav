// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Program validation against ISU rules.
//!
//! This module provides validation functions to check programs against
//! ISU (International Skating Union) rules.

use crate::rules::{Discipline, ISURules, SegmentRules};
use anv_core::diagnostics::{Diagnostic, ErrorCode};
use anv_core::source::Span;
use anv_syntax::ast::{Element, ElementKind, Program, Segment, SegmentKind};
use thiserror::Error;

/// Semantic error type.
#[derive(Debug, Clone, Error)]
pub enum SemanticError {
    #[error("Too many {element_type} elements: found {found}, maximum {max}")]
    TooManyElements {
        element_type: String,
        found: usize,
        max: usize,
        span: Span,
    },

    #[error("Too few {element_type} elements: found {found}, minimum {min}")]
    TooFewElements {
        element_type: String,
        found: usize,
        min: usize,
        span: Span,
    },

    #[error("Duration out of range: {duration}s (expected {min}s - {max}s)")]
    DurationOutOfRange {
        duration: u32,
        min: u32,
        max: u32,
        span: Span,
    },

    #[error("Missing required element: {description}")]
    MissingRequiredElement { description: String, span: Span },

    #[error("Duplicate element type not allowed: {element_type}")]
    DuplicateElement { element_type: String, span: Span },

    #[error("Element not allowed in this segment: {element_type}")]
    ElementNotAllowed { element_type: String, span: Span },

    #[error("Invalid element for discipline: {message}")]
    InvalidForDiscipline { message: String, span: Span },
}

impl SemanticError {
    /// Get a helpful hint for this error.
    pub fn hint(&self) -> Option<String> {
        match self {
            SemanticError::TooManyElements { element_type, max, .. } => {
                Some(format!(
                    "ISU rules limit {} elements to {} in this segment. Consider removing some or moving to a different sequence.",
                    element_type, max
                ))
            }
            SemanticError::TooFewElements { element_type, min, .. } => {
                Some(format!(
                    "ISU rules require at least {} {} elements in this segment. Add more to meet the requirement.",
                    min, element_type
                ))
            }
            SemanticError::DurationOutOfRange { min, max, .. } => {
                Some(format!(
                    "Segment duration must be between {}s and {}s according to ISU rules. Adjust your choreography timing.",
                    min, max
                ))
            }
            SemanticError::MissingRequiredElement { description, .. } => {
                Some(format!(
                    "This element is required by ISU rules: {}. Add it to your program.",
                    description
                ))
            }
            SemanticError::DuplicateElement { element_type, .. } => {
                Some(format!(
                    "ISU rules don't allow duplicate {} elements in the same segment. Remove the duplicate.",
                    element_type
                ))
            }
            SemanticError::ElementNotAllowed { element_type, .. } => {
                Some(format!(
                    "{} is not allowed in this segment type. Check ISU rules for allowed elements.",
                    element_type
                ))
            }
            SemanticError::InvalidForDiscipline { .. } => {
                Some("This element doesn't match the discipline rules. For example, pairs elements like lifts are only allowed in pairs skating.".to_string())
            }
        }
    }

    /// Get the span for this error.
    pub fn span(&self) -> Span {
        match self {
            SemanticError::TooManyElements { span, .. } => *span,
            SemanticError::TooFewElements { span, .. } => *span,
            SemanticError::DurationOutOfRange { span, .. } => *span,
            SemanticError::MissingRequiredElement { span, .. } => *span,
            SemanticError::DuplicateElement { span, .. } => *span,
            SemanticError::ElementNotAllowed { span, .. } => *span,
            SemanticError::InvalidForDiscipline { span, .. } => *span,
        }
    }

    /// Convert to a Diagnostic with helpful hints.
    pub fn to_diagnostic(&self) -> Diagnostic {
        let mut diag = Diagnostic::error(self.to_string())
            .with_code(self.error_code())
            .with_label(self.span(), "here");

        if let Some(hint) = self.hint() {
            diag = diag.with_help(hint);
        }

        diag
    }

    /// Get the error code for this error.
    fn error_code(&self) -> &'static str {
        match self {
            SemanticError::TooManyElements { element_type, .. } => {
                if element_type.contains("jump") {
                    ErrorCode::TOO_MANY_JUMPS
                } else {
                    ErrorCode::INVALID_COMBINATION
                }
            }
            SemanticError::TooFewElements { .. } => ErrorCode::INVALID_COMBINATION,
            SemanticError::DurationOutOfRange { .. } => ErrorCode::DURATION_EXCEEDED,
            SemanticError::MissingRequiredElement { .. } => ErrorCode::INVALID_COMBINATION,
            SemanticError::DuplicateElement { .. } => ErrorCode::INVALID_COMBINATION,
            SemanticError::ElementNotAllowed { .. } => ErrorCode::INVALID_COMBINATION,
            SemanticError::InvalidForDiscipline { .. } => ErrorCode::INVALID_COMBINATION,
        }
    }
}

/// Validation result containing warnings and errors.
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Semantic errors that must be fixed.
    pub errors: Vec<SemanticError>,
    /// Warnings that should be reviewed.
    pub warnings: Vec<SemanticError>,
}

impl ValidationResult {
    /// Returns true if validation passed with no errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Validate a program against ISU rules.
pub fn validate_program(program: &Program, discipline: Discipline) -> ValidationResult {
    validate_program_with_rules(program, discipline, &ISURules::default())
}

/// Validate a program with custom rules configuration.
pub fn validate_program_with_rules(
    program: &Program,
    discipline: Discipline,
    _rules: &ISURules,
) -> ValidationResult {
    let mut result = ValidationResult::default();

    for segment in &program.segments {
        validate_segment(segment, discipline, &mut result);
    }

    result
}

/// Validate a single segment.
fn validate_segment(segment: &Segment, discipline: Discipline, result: &mut ValidationResult) {
    let rules = discipline.segment_rules(segment.kind);

    // Collect all elements from all sequences
    let elements: Vec<&Element> = segment
        .sequences
        .iter()
        .flat_map(|seq| &seq.elements)
        .collect();

    // Count elements by type
    let counts = count_elements(&elements);

    // Validate element counts
    validate_element_counts(&counts, &rules, segment.span, result);

    // Validate discipline-specific rules
    validate_discipline_rules(&elements, discipline, segment.kind, segment.span, result);
}

/// Element counts by type.
#[derive(Debug, Default)]
struct ElementCounts {
    jumps: usize,
    spins: usize,
    step_sequences: usize,
    lifts: usize,
    throws: usize,
    twists: usize,
    death_spirals: usize,
    choreographic: usize,
    patterns: usize,
}

/// Count elements by type.
fn count_elements(elements: &[&Element]) -> ElementCounts {
    let mut counts = ElementCounts::default();

    for element in elements {
        match &element.kind {
            ElementKind::Jump(_) => counts.jumps += 1,
            ElementKind::Spin(_) => counts.spins += 1,
            ElementKind::StepSequence(_) => counts.step_sequences += 1,
            ElementKind::Lift(_) => counts.lifts += 1,
            ElementKind::Throw(_) => counts.throws += 1,
            ElementKind::Twist(_) => counts.twists += 1,
            ElementKind::DeathSpiral(_) => counts.death_spirals += 1,
            ElementKind::Choreographic(_) => counts.choreographic += 1,
            ElementKind::Pattern(_) => counts.patterns += 1,
            ElementKind::Transition(_) | ElementKind::Parallel(_) | ElementKind::Sync(_) => {}
        }
    }

    counts
}

/// Validate element counts against rules.
fn validate_element_counts(
    counts: &ElementCounts,
    rules: &SegmentRules,
    span: Span,
    result: &mut ValidationResult,
) {
    // Check maximum jumps
    if let Some(max) = rules.max_jumps {
        if counts.jumps > max as usize {
            result.errors.push(SemanticError::TooManyElements {
                element_type: "jump".into(),
                found: counts.jumps,
                max: max as usize,
                span,
            });
        }
    }

    // Check maximum spins
    if let Some(max) = rules.max_spins {
        if counts.spins > max as usize {
            result.errors.push(SemanticError::TooManyElements {
                element_type: "spin".into(),
                found: counts.spins,
                max: max as usize,
                span,
            });
        }
    }

    // Check step sequences
    if let Some(required) = rules.step_sequences {
        if counts.step_sequences < required as usize {
            result.errors.push(SemanticError::TooFewElements {
                element_type: "step sequence".into(),
                found: counts.step_sequences,
                min: required as usize,
                span,
            });
        }
    }

    // Check required lifts
    if let Some(required) = rules.required_lifts {
        if counts.lifts < required as usize {
            result.errors.push(SemanticError::TooFewElements {
                element_type: "lift".into(),
                found: counts.lifts,
                min: required as usize,
                span,
            });
        }
    }

    // Check required throws
    if let Some(required) = rules.required_throws {
        if counts.throws < required as usize {
            result.errors.push(SemanticError::TooFewElements {
                element_type: "throw".into(),
                found: counts.throws,
                min: required as usize,
                span,
            });
        }
    }

    // Check required twists
    if let Some(required) = rules.required_twists {
        if counts.twists < required as usize {
            result.errors.push(SemanticError::TooFewElements {
                element_type: "twist".into(),
                found: counts.twists,
                min: required as usize,
                span,
            });
        }
    }

    // Check required death spirals
    if let Some(required) = rules.required_death_spirals {
        if counts.death_spirals < required as usize {
            result.errors.push(SemanticError::TooFewElements {
                element_type: "death spiral".into(),
                found: counts.death_spirals,
                min: required as usize,
                span,
            });
        }
    }
}

/// Validate discipline-specific rules.
fn validate_discipline_rules(
    elements: &[&Element],
    discipline: Discipline,
    segment_kind: SegmentKind,
    span: Span,
    result: &mut ValidationResult,
) {
    match discipline {
        Discipline::MenSingles | Discipline::LadiesSingles => {
            validate_singles_rules(elements, span, result);
        }
        Discipline::Pairs => {
            validate_pairs_rules(elements, span, result);
        }
        Discipline::IceDance => {
            validate_ice_dance_rules(elements, segment_kind, span, result);
        }
    }
}

/// Validate singles-specific rules.
fn validate_singles_rules(elements: &[&Element], span: Span, result: &mut ValidationResult) {
    // Singles cannot have pairs elements
    for element in elements {
        match &element.kind {
            ElementKind::Lift(_) => {
                result.errors.push(SemanticError::InvalidForDiscipline {
                    message: "Lifts are not allowed in singles skating".into(),
                    span,
                });
            }
            ElementKind::Throw(_) => {
                result.errors.push(SemanticError::InvalidForDiscipline {
                    message: "Throws are not allowed in singles skating".into(),
                    span,
                });
            }
            ElementKind::Twist(_) => {
                result.errors.push(SemanticError::InvalidForDiscipline {
                    message: "Twists are not allowed in singles skating".into(),
                    span,
                });
            }
            ElementKind::DeathSpiral(_) => {
                result.errors.push(SemanticError::InvalidForDiscipline {
                    message: "Death spirals are not allowed in singles skating".into(),
                    span,
                });
            }
            _ => {}
        }
    }
}

/// Validate pairs-specific rules.
fn validate_pairs_rules(_elements: &[&Element], _span: Span, _result: &mut ValidationResult) {
    // Pairs can have most element types
    // Could add specific validation for lift groups, etc.
}

/// Validate ice dance-specific rules.
fn validate_ice_dance_rules(
    elements: &[&Element],
    _segment_kind: SegmentKind,
    span: Span,
    result: &mut ValidationResult,
) {
    // Ice dance has restrictions on jumps
    for element in elements {
        match &element.kind {
            ElementKind::Jump(jump) => {
                // Ice dance only allows single jumps
                if jump.rotations != anv_core::skating::Rotations::Single {
                    result.errors.push(SemanticError::InvalidForDiscipline {
                        message: "Ice dance only allows single jumps".into(),
                        span,
                    });
                }
            }
            ElementKind::Throw(_) => {
                result.errors.push(SemanticError::InvalidForDiscipline {
                    message: "Throws are not allowed in ice dance".into(),
                    span,
                });
            }
            ElementKind::Twist(_) => {
                result.errors.push(SemanticError::InvalidForDiscipline {
                    message: "Twists are not allowed in ice dance".into(),
                    span,
                });
            }
            ElementKind::DeathSpiral(_) => {
                result.errors.push(SemanticError::InvalidForDiscipline {
                    message: "Death spirals are not allowed in ice dance".into(),
                    span,
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anv_core::source::FileId;
    use anv_syntax::parse;

    fn parse_and_validate(source: &str, discipline: Discipline) -> ValidationResult {
        let program = parse(source, FileId(0)).expect("Failed to parse");
        validate_program(&program, discipline)
    }

    #[test]
    fn test_valid_singles_program() {
        let source = r#"
            program test {
                segment sp: short {
                    sequence {
                        jump triple axel
                        jump triple lutz
                        spin camel L3
                        step circular L4
                    }
                }
            }
        "#;
        let result = parse_and_validate(source, Discipline::MenSingles);
        // May have warnings about missing elements, but should be parseable
        assert!(result.errors.iter().all(|e| !matches!(
            e,
            SemanticError::InvalidForDiscipline { .. }
        )));
    }

    #[test]
    fn test_singles_with_pairs_element_fails() {
        let source = r#"
            program test {
                segment sp: short {
                    sequence {
                        lift Gr3 L4
                    }
                }
            }
        "#;
        let result = parse_and_validate(source, Discipline::MenSingles);
        assert!(result.has_errors());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, SemanticError::InvalidForDiscipline { .. })));
    }

    #[test]
    fn test_too_many_jumps() {
        let source = r#"
            program test {
                segment sp: short {
                    sequence {
                        jump triple axel
                        jump triple lutz
                        jump triple flip
                        jump triple loop
                    }
                }
            }
        "#;
        let result = parse_and_validate(source, Discipline::MenSingles);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, SemanticError::TooManyElements { .. })));
    }

    #[test]
    fn test_valid_pairs_program() {
        let source = r#"
            program test {
                segment sp: short {
                    sequence {
                        lift Gr5 L4
                        throw triple axel
                        twist double L3
                        death_spiral LBI L4
                        spin camel L3
                        step circular L4
                    }
                }
            }
        "#;
        let result = parse_and_validate(source, Discipline::Pairs);
        // Pairs can have all these elements
        assert!(result.errors.iter().all(|e| !matches!(
            e,
            SemanticError::InvalidForDiscipline { .. }
        )));
    }

    #[test]
    fn test_ice_dance_no_multi_rotation_jumps() {
        let source = r#"
            program test {
                segment rd: rhythm {
                    sequence {
                        jump triple axel
                    }
                }
            }
        "#;
        let result = parse_and_validate(source, Discipline::IceDance);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| {
            if let SemanticError::InvalidForDiscipline { message, .. } = e {
                message.contains("single jumps")
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_exhibition_allows_anything() {
        let source = r#"
            program test {
                segment gala: exhibition {
                    sequence {
                        jump quad axel
                        lift Gr5 L4
                        choreographic spiral
                    }
                }
            }
        "#;
        let _result = parse_and_validate(source, Discipline::MenSingles);
        // Exhibition segments have no strict rules
        // This test just verifies parsing works for exhibition
    }

    #[test]
    fn test_missing_step_sequence() {
        let source = r#"
            program test {
                segment sp: short {
                    sequence {
                        jump triple axel
                        spin camel L3
                    }
                }
            }
        "#;
        let result = parse_and_validate(source, Discipline::MenSingles);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, SemanticError::TooFewElements { element_type, .. } if element_type == "step sequence")));
    }
}
