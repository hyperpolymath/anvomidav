// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Diagnostic reporting for errors, warnings, and hints.

use crate::source::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Informational hint.
    Hint,
    /// Warning (does not prevent compilation).
    Warning,
    /// Error (prevents successful compilation).
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Hint => write!(f, "hint"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// Error code for categorizing diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ErrorCode(pub String);

impl ErrorCode {
    // Syntax errors
    pub const UNEXPECTED_TOKEN: &'static str = "E0001";
    pub const UNTERMINATED_STRING: &'static str = "E0002";
    pub const INVALID_NUMBER: &'static str = "E0003";
    pub const INVALID_TIME: &'static str = "E0004";

    // Type errors
    pub const TYPE_MISMATCH: &'static str = "E0100";
    pub const UNDEFINED_VARIABLE: &'static str = "E0101";
    pub const UNDEFINED_ELEMENT: &'static str = "E0102";
    pub const WRONG_ARITY: &'static str = "E0103";

    // ISU rule errors
    pub const INVALID_COMBINATION: &'static str = "E0200";
    pub const TOO_MANY_JUMPS: &'static str = "E0201";
    pub const ZAYAK_VIOLATION: &'static str = "E0202";
    pub const WRONG_EDGE: &'static str = "E0203";
    pub const DURATION_EXCEEDED: &'static str = "E0204";

    // Temporal errors
    pub const TEMPORAL_OVERLAP: &'static str = "E0300";
    pub const UNREACHABLE_TIME: &'static str = "E0301";
    pub const DURATION_MISMATCH: &'static str = "E0302";

    // Spatial errors
    pub const OUT_OF_BOUNDS: &'static str = "E0400";
    pub const COLLISION: &'static str = "E0401";
    pub const UNREACHABLE_POSITION: &'static str = "E0402";

    pub fn new(code: impl Into<String>) -> Self {
        ErrorCode(code.into())
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A secondary label attached to a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    /// Span of the label.
    pub span: Span,
    /// Message for the label.
    pub message: String,
    /// Is this the primary label?
    pub primary: bool,
}

impl Label {
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Label {
            span,
            message: message.into(),
            primary: true,
        }
    }

    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Label {
            span,
            message: message.into(),
            primary: false,
        }
    }
}

/// A note or help message attached to a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Note {
    /// A note providing additional context.
    Note(String),
    /// A help message suggesting a fix.
    Help(String),
}

/// A diagnostic message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Severity level.
    pub severity: Severity,
    /// Error code (optional).
    pub code: Option<ErrorCode>,
    /// Primary message.
    pub message: String,
    /// Labels pointing to source locations.
    pub labels: Vec<Label>,
    /// Additional notes and help.
    pub notes: Vec<Note>,
}

impl Diagnostic {
    /// Create a new error diagnostic.
    pub fn error(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            code: None,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Create a new warning diagnostic.
    pub fn warning(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            code: None,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Create a new hint diagnostic.
    pub fn hint(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Hint,
            code: None,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Set the error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(ErrorCode::new(code));
        self
    }

    /// Add a primary label.
    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label::primary(span, message));
        self
    }

    /// Add a secondary label.
    pub fn with_secondary_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label::secondary(span, message));
        self
    }

    /// Add a note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(Note::Note(note.into()));
        self
    }

    /// Add a help message.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.notes.push(Note::Help(help.into()));
        self
    }

    /// Is this an error?
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Get the primary span (first primary label).
    pub fn primary_span(&self) -> Option<Span> {
        self.labels.iter().find(|l| l.primary).map(|l| l.span)
    }
}

/// Collection of diagnostics.
#[derive(Debug, Default, Clone)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics {
            diagnostics: Vec::new(),
        }
    }

    /// Add a diagnostic.
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Add an error.
    pub fn error(&mut self, message: impl Into<String>, span: Span) {
        self.add(Diagnostic::error(message).with_label(span, "here"));
    }

    /// Add a warning.
    pub fn warning(&mut self, message: impl Into<String>, span: Span) {
        self.add(Diagnostic::warning(message).with_label(span, "here"));
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    /// Get error count.
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.is_error()).count()
    }

    /// Get warning count.
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    /// Iterate over all diagnostics.
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    /// Take all diagnostics.
    pub fn take(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Merge another diagnostics collection.
    pub fn merge(&mut self, other: Diagnostics) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Number of diagnostics.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}

impl Extend<Diagnostic> for Diagnostics {
    fn extend<T: IntoIterator<Item = Diagnostic>>(&mut self, iter: T) {
        self.diagnostics.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::FileId;

    #[test]
    fn test_diagnostic_builder() {
        let span = Span::new(10, 20, FileId(0));
        let diag = Diagnostic::error("Type mismatch")
            .with_code(ErrorCode::TYPE_MISMATCH)
            .with_label(span, "expected int, found string")
            .with_help("Try converting the string to an integer");

        assert!(diag.is_error());
        assert_eq!(diag.code, Some(ErrorCode::new(ErrorCode::TYPE_MISMATCH)));
        assert_eq!(diag.labels.len(), 1);
        assert_eq!(diag.notes.len(), 1);
    }

    #[test]
    fn test_diagnostics_collection() {
        let mut diags = Diagnostics::new();
        let span = Span::new(0, 10, FileId(0));

        diags.error("Error 1", span);
        diags.warning("Warning 1", span);
        diags.error("Error 2", span);

        assert!(diags.has_errors());
        assert_eq!(diags.error_count(), 2);
        assert_eq!(diags.warning_count(), 1);
    }
}
