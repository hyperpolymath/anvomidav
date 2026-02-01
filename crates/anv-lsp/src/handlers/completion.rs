// SPDX-License-Identifier: MIT
// Completion handler for Anvomidav LSP

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Position, Url};

/// Keywords for Anvomidav choreography DSL
const KEYWORDS: &[(&str, &str)] = &[
    ("program", "Define a choreography program"),
    ("segment", "Define a program segment"),
    ("sequence", "Define a sequence of elements"),
    ("transition", "Define a transition between elements"),
    ("choreography", "Define choreography structure"),
    ("spiral", "Define a spiral sequence"),
    ("step", "Define a step sequence"),
    ("turn", "Define a turn sequence"),
    ("jump", "Define a jump element"),
    ("spin", "Define a spin element"),
    ("lift", "Define a lift element (pairs)"),
    ("throw", "Define a throw element (pairs)"),
    ("twist", "Define a twist element (pairs)"),
    ("death_spiral", "Define a death spiral (pairs)"),
    ("pattern", "Define an ice dance pattern"),
    ("timing", "Specify element timing"),
    ("music", "Music reference"),
    ("tempo", "Tempo specification"),
    ("short", "Short program segment"),
    ("free", "Free program segment"),
    ("rhythm", "Rhythm dance segment"),
];

/// Jump types
const JUMPS: &[(&str, &str)] = &[
    ("axel", "Axel jump (1.5 rotations forward takeoff)"),
    ("lutz", "Lutz jump (toe-assisted, outside edge)"),
    ("flip", "Flip jump (toe-assisted, inside edge)"),
    ("loop", "Loop jump (edge jump)"),
    ("salchow", "Salchow jump (edge jump, inside edge)"),
    ("toe_loop", "Toe loop jump (toe-assisted)"),
];

/// Spin types
const SPINS: &[(&str, &str)] = &[
    ("camel", "Camel spin (arabesque position)"),
    ("sit", "Sit spin (sitting position)"),
    ("upright", "Upright spin"),
    ("layback", "Layback spin"),
    ("biellmann", "Biellmann spin (leg held overhead)"),
    ("combination", "Combination spin"),
    ("flying", "Flying spin (jump entry)"),
];

/// Rotation qualifiers
const ROTATIONS: &[(&str, &str)] = &[
    ("single", "Single rotation"),
    ("double", "Double rotation"),
    ("triple", "Triple rotation"),
    ("quad", "Quadruple rotation"),
];

/// Edge qualifiers
const EDGES: &[(&str, &str)] = &[
    ("inside", "Inside edge"),
    ("outside", "Outside edge"),
    ("forward", "Forward direction"),
    ("backward", "Backward direction"),
];

pub async fn handle_completion(
    _uri: &Url,
    source: &str,
    params: &CompletionParams,
) -> Option<CompletionResponse> {
    let position = params.text_document_position.position;
    let context = get_context_at_position(source, position);

    let mut items = Vec::new();

    match context.as_str() {
        "jump" => {
            // Suggest jump types
            for (name, detail) in JUMPS {
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                    detail: Some(detail.to_string()),
                    ..Default::default()
                });
            }
            // Add rotation qualifiers
            for (name, detail) in ROTATIONS {
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some(detail.to_string()),
                    ..Default::default()
                });
            }
        }
        "spin" => {
            // Suggest spin types
            for (name, detail) in SPINS {
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                    detail: Some(detail.to_string()),
                    ..Default::default()
                });
            }
        }
        _ => {
            // Default: suggest all keywords
            for (name, detail) in KEYWORDS {
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some(detail.to_string()),
                    ..Default::default()
                });
            }
        }
    }

    Some(CompletionResponse::Array(items))
}

fn get_context_at_position(source: &str, position: Position) -> String {
    let lines: Vec<&str> = source.lines().collect();
    if position.line as usize >= lines.len() {
        return String::new();
    }

    let line = lines[position.line as usize];
    let before_cursor = &line[..position.character.min(line.len() as u32) as usize];

    // Detect if we're after "jump" or "spin" keywords
    if before_cursor.contains("jump") {
        "jump".to_string()
    } else if before_cursor.contains("spin") {
        "spin".to_string()
    } else {
        String::new()
    }
}
