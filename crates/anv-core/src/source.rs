// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Source location tracking.
//!
//! This module provides types for tracking positions in source code,
//! essential for error reporting and IDE features.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Range;

/// Unique identifier for a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(pub u32);

impl FileId {
    /// Dummy file ID for generated/synthetic code.
    pub const DUMMY: FileId = FileId(u32::MAX);
}

/// A span in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Start byte offset (inclusive).
    pub start: u32,
    /// End byte offset (exclusive).
    pub end: u32,
    /// File containing this span.
    pub file_id: FileId,
}

impl Span {
    /// Create a new span.
    pub fn new(start: u32, end: u32, file_id: FileId) -> Self {
        debug_assert!(start <= end);
        Span { start, end, file_id }
    }

    /// Create a dummy span for synthetic nodes.
    pub const fn dummy() -> Self {
        Span {
            start: 0,
            end: 0,
            file_id: FileId::DUMMY,
        }
    }

    /// Length of the span in bytes.
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Is this span empty?
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Convert to a byte range.
    pub fn to_range(&self) -> Range<usize> {
        (self.start as usize)..(self.end as usize)
    }

    /// Merge two spans (taking the union).
    pub fn merge(&self, other: &Span) -> Self {
        debug_assert_eq!(self.file_id, other.file_id);
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            file_id: self.file_id,
        }
    }

    /// Create a zero-width span at the start of this span.
    pub fn start_span(&self) -> Self {
        Span {
            start: self.start,
            end: self.start,
            file_id: self.file_id,
        }
    }

    /// Create a zero-width span at the end of this span.
    pub fn end_span(&self) -> Self {
        Span {
            start: self.end,
            end: self.end,
            file_id: self.file_id,
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Span::dummy()
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A value with an associated source span.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Create a new spanned value.
    pub fn new(node: T, span: Span) -> Self {
        Spanned { node, span }
    }

    /// Create a spanned value with a dummy span.
    pub fn dummy(node: T) -> Self {
        Spanned {
            node,
            span: Span::dummy(),
        }
    }

    /// Map the inner value.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }

    /// Get a reference to the inner value.
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            node: &self.node,
            span: self.span,
        }
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

/// Line and column location (1-indexed, for display).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineCol {
    /// Line number (1-indexed).
    pub line: u32,
    /// Column number (1-indexed, in UTF-8 code units).
    pub col: u32,
}

impl LineCol {
    pub fn new(line: u32, col: u32) -> Self {
        LineCol { line, col }
    }
}

impl fmt::Display for LineCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// Source file with contents and line index.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// File ID.
    pub id: FileId,
    /// File name/path.
    pub name: String,
    /// File contents.
    pub contents: String,
    /// Byte offsets of line starts.
    line_starts: Vec<u32>,
}

impl SourceFile {
    /// Create a new source file.
    pub fn new(id: FileId, name: String, contents: String) -> Self {
        let line_starts = std::iter::once(0)
            .chain(
                contents
                    .bytes()
                    .enumerate()
                    .filter(|(_, b)| *b == b'\n')
                    .map(|(i, _)| (i + 1) as u32),
            )
            .collect();

        SourceFile {
            id,
            name,
            contents,
            line_starts,
        }
    }

    /// Get the line/column for a byte offset.
    pub fn line_col(&self, offset: u32) -> LineCol {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line - 1,
        };
        let col = offset - self.line_starts[line];
        LineCol::new(line as u32 + 1, col + 1)
    }

    /// Get the byte offset for a line/column.
    pub fn offset(&self, line_col: LineCol) -> Option<u32> {
        let line_idx = (line_col.line - 1) as usize;
        if line_idx >= self.line_starts.len() {
            return None;
        }
        let line_start = self.line_starts[line_idx];
        Some(line_start + line_col.col - 1)
    }

    /// Get the source text for a span.
    pub fn slice(&self, span: &Span) -> &str {
        &self.contents[span.to_range()]
    }

    /// Get a specific line (1-indexed).
    pub fn line(&self, line_num: u32) -> Option<&str> {
        let idx = (line_num - 1) as usize;
        if idx >= self.line_starts.len() {
            return None;
        }
        let start = self.line_starts[idx] as usize;
        let end = self
            .line_starts
            .get(idx + 1)
            .map(|&e| e as usize)
            .unwrap_or(self.contents.len());
        Some(self.contents[start..end].trim_end_matches('\n'))
    }

    /// Number of lines in the file.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}

/// Database of source files.
#[derive(Debug, Default)]
pub struct SourceDb {
    files: Vec<SourceFile>,
}

impl SourceDb {
    pub fn new() -> Self {
        SourceDb { files: Vec::new() }
    }

    /// Add a source file.
    pub fn add(&mut self, name: String, contents: String) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile::new(id, name, contents));
        id
    }

    /// Get a source file by ID.
    pub fn get(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.0 as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_merge() {
        let file = FileId(0);
        let s1 = Span::new(10, 20, file);
        let s2 = Span::new(15, 30, file);
        let merged = s1.merge(&s2);
        assert_eq!(merged.start, 10);
        assert_eq!(merged.end, 30);
    }

    #[test]
    fn test_source_file_line_col() {
        let file = SourceFile::new(
            FileId(0),
            "test.anv".to_string(),
            "line1\nline2\nline3".to_string(),
        );

        assert_eq!(file.line_col(0), LineCol::new(1, 1));
        assert_eq!(file.line_col(5), LineCol::new(1, 6)); // newline
        assert_eq!(file.line_col(6), LineCol::new(2, 1)); // start of line2
    }

    #[test]
    fn test_source_file_line() {
        let file = SourceFile::new(
            FileId(0),
            "test.anv".to_string(),
            "line1\nline2\nline3".to_string(),
        );

        assert_eq!(file.line(1), Some("line1"));
        assert_eq!(file.line(2), Some("line2"));
        assert_eq!(file.line(3), Some("line3"));
        assert_eq!(file.line(4), None);
    }
}
