// SPDX-License-Identifier: MIT
// Formatting handler for Anvomidav LSP

use tower_lsp::lsp_types::{Position, Range, TextEdit};

pub async fn format_document(source: &str) -> Vec<TextEdit> {
    // For now, return empty - formatting is complex for a DSL
    // Future: implement proper formatting based on anvomidav grammar

    // Basic formatting: ensure consistent indentation
    let formatted = format_indentation(source);

    if formatted == source {
        vec![]
    } else {
        vec![TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: source.lines().count() as u32,
                    character: 0,
                },
            },
            new_text: formatted,
        }]
    }
}

fn format_indentation(source: &str) -> String {
    let mut result = String::new();
    let mut indent_level: usize = 0;
    const INDENT: &str = "    "; // 4 spaces

    for line in source.lines() {
        let trimmed = line.trim();

        // Decrease indent for closing braces
        if trimmed.starts_with('}') {
            indent_level = indent_level.saturating_sub(1);
        }

        // Add indentation
        if !trimmed.is_empty() {
            result.push_str(&INDENT.repeat(indent_level));
            result.push_str(trimmed);
        }
        result.push('\n');

        // Increase indent for opening braces
        if trimmed.ends_with('{') {
            indent_level += 1;
        }
    }

    result
}
