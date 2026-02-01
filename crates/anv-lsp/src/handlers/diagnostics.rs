// SPDX-License-Identifier: MIT
// Diagnostics handler for Anvomidav LSP

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub async fn generate_diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check for common syntax errors
    for (line_num, line) in source.lines().enumerate() {
        // Check for unclosed braces
        let open_braces = line.matches('{').count();
        let close_braces = line.matches('}').count();

        if open_braces > 0 && close_braces == 0 {
            // This might be intentional (opening brace), but check if it's ever closed
            let remaining_lines: String = source.lines().skip(line_num + 1).collect();
            if !remaining_lines.contains('}') {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: (line.find('{').unwrap_or(0)) as u32,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: (line.find('{').unwrap_or(0) + 1) as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Unclosed brace".to_string(),
                    source: Some("anvomidav-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        // Check for invalid jump qualifiers
        if line.contains("jump") {
            let invalid_qualifiers = ["penta", "hexa", "sept", "oct"];
            for qualifier in &invalid_qualifiers {
                if line.contains(qualifier) {
                    if let Some(pos) = line.find(qualifier) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: pos as u32,
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: (pos + qualifier.len()) as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!(
                                "Invalid rotation qualifier '{}'. Use single, double, triple, or quad.",
                                qualifier
                            ),
                            source: Some("anvomidav-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Check for ISU rule violations (simplified)
        if line.contains("short") && line.contains("program") {
            // Short programs have strict time limits
            if line.contains("5:00") || line.contains("6:00") || line.contains("7:00") {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: "Short program time limit exceeded. Singles: 2:40, Pairs: 2:40".to_string(),
                    source: Some("anvomidav-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        // Warn about deprecated terms
        if line.contains("ladies") || line.contains("men") {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: line_num as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: line.len() as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::INFORMATION),
                message: "Consider using gender-neutral terms: 'singles' instead of 'ladies'/'men'".to_string(),
                source: Some("anvomidav-lsp".to_string()),
                ..Default::default()
            });
        }
    }

    diagnostics
}
