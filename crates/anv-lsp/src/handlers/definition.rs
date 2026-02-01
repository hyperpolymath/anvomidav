// SPDX-License-Identifier: MIT
// Go-to-definition handler for Anvomidav LSP

use tower_lsp::lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Position, Range, Url};

pub async fn handle_definition(
    uri: &Url,
    source: &str,
    params: &GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let position = params.text_document_position_params.position;
    let word = get_word_at_position(source, position)?;

    // Search for definition
    let definition_location = find_definition(source, &word, uri)?;

    Some(GotoDefinitionResponse::Scalar(definition_location))
}

fn get_word_at_position(source: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    if char_pos >= line.len() {
        return None;
    }

    // Find word boundaries
    let start = line[..char_pos]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);

    let end = line[char_pos..]
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| char_pos + i)
        .unwrap_or(line.len());

    Some(line[start..end].to_string())
}

fn find_definition(source: &str, target: &str, uri: &Url) -> Option<Location> {
    // Search for definitions of segments, sequences, etc.
    for (line_num, line) in source.lines().enumerate() {
        // Match: "segment <name>:", "sequence <name> {", "program <name> {"
        if let Some(pos) = line.find(target) {
            // Check if this is a definition (has "segment", "sequence", "program" before it)
            let before = &line[..pos];
            if before.contains("segment ")
                || before.contains("sequence ")
                || before.contains("program ")
                || before.contains("transition ")
            {
                return Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: pos as u32,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: (pos + target.len()) as u32,
                        },
                    },
                });
            }
        }
    }

    None
}
