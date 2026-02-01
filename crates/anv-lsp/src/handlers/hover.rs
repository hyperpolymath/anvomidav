// SPDX-License-Identifier: MIT
// Hover handler for Anvomidav LSP

use tower_lsp::lsp_types::{Hover, HoverContents, HoverParams, MarkedString, Position, Url};

pub async fn handle_hover(_uri: &Url, source: &str, params: &HoverParams) -> Option<Hover> {
    let position = params.text_document_position_params.position;
    let word = get_word_at_position(source, position)?;

    let documentation = get_keyword_documentation(&word)?;

    Some(Hover {
        contents: HoverContents::Scalar(MarkedString::String(documentation)),
        range: None,
    })
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

fn get_keyword_documentation(word: &str) -> Option<String> {
    match word {
        // Core structure
        "program" => Some("**program** - Define a complete choreography program\n\nSyntax: `program <name> { ... }`".to_string()),
        "segment" => Some("**segment** - Define a program segment (short program, free skate, rhythm dance)\n\nSyntax: `segment <id>: <type> { ... }`".to_string()),
        "sequence" => Some("**sequence** - Define a sequence of choreographic elements\n\nSyntax: `sequence <name> { ... }`".to_string()),
        "transition" => Some("**transition** - Define a transition between elements\n\nSyntax: `transition <from> -> <to> { ... }`".to_string()),

        // Jump elements
        "jump" => Some("**jump** - Define a jump element\n\nTypes: axel, lutz, flip, loop, salchow, toe_loop\nQualifiers: single, double, triple, quad".to_string()),
        "axel" => Some("**axel** - Axel jump (forward takeoff, 1.5 rotations)\n\nOnly jump with forward takeoff. Named after Axel Paulsen.".to_string()),
        "lutz" => Some("**lutz** - Lutz jump (toe-assisted, outside edge)\n\nToe-pick assisted jump from outside edge. Named after Alois Lutz.".to_string()),
        "flip" => Some("**flip** - Flip jump (toe-assisted, inside edge)\n\nToe-pick assisted jump from inside edge.".to_string()),
        "loop" => Some("**loop** - Loop jump (edge jump)\n\nEdge jump with no toe assistance, takeoff and landing on same edge.".to_string()),
        "salchow" => Some("**salchow** - Salchow jump (inside edge)\n\nEdge jump from inside edge. Named after Ulrich Salchow.".to_string()),
        "toe_loop" => Some("**toe_loop** - Toe loop jump (toe-assisted)\n\nToe-pick assisted jump, typically the easiest multi-rotation jump.".to_string()),

        // Spin elements
        "spin" => Some("**spin** - Define a spin element\n\nTypes: camel, sit, upright, layback, biellmann, combination, flying".to_string()),
        "camel" => Some("**camel** - Camel spin (arabesque position)\n\nSpin in arabesque position with free leg extended behind.".to_string()),
        "sit" => Some("**sit** - Sit spin (sitting position)\n\nSpin in sitting position with skating knee bent at least 90°.".to_string()),
        "upright" => Some("**upright** - Upright spin\n\nSpin in standing position.".to_string()),
        "layback" => Some("**layback** - Layback spin\n\nUpright spin with head and shoulders arched backward.".to_string()),
        "biellmann" => Some("**biellmann** - Biellmann spin\n\nLayback spin with free leg pulled overhead. Named after Denise Biellmann.".to_string()),

        // Pairs elements
        "lift" => Some("**lift** - Pairs lift element\n\nPartner lifts skater overhead. Required in pairs programs.".to_string()),
        "throw" => Some("**throw** - Pairs throw jump\n\nPartner assists in launching skater into jump.".to_string()),
        "twist" => Some("**twist** - Pairs twist lift\n\nSkater thrown into air, rotates, and is caught by partner.".to_string()),
        "death_spiral" => Some("**death_spiral** - Pairs death spiral\n\nPivot element where partner is in pivot while holding skater in spiral position near ice.".to_string()),

        // Ice dance
        "pattern" => Some("**pattern** - Ice dance pattern\n\nPrescribed set of steps for ice dance.".to_string()),

        // Timing
        "timing" => Some("**timing** - Element timing specification\n\nSyntax: `timing { ... }`".to_string()),
        "music" => Some("**music** - Music reference\n\nSyntax: `music \"<title>\" by \"<artist>\"`".to_string()),
        "tempo" => Some("**tempo** - Tempo specification\n\nSyntax: `tempo <bpm>`".to_string()),

        // Segment types
        "short" => Some("**short** - Short program segment\n\nRequired elements with time limit (singles: 2:40, pairs: 2:40, ice dance: varies)".to_string()),
        "free" => Some("**free** - Free skate segment\n\nFree choice of elements within limits (singles: 4:00, pairs: 4:00)".to_string()),
        "rhythm" => Some("**rhythm** - Rhythm dance segment (ice dance)\n\nPattern dance with required elements.".to_string()),

        // Rotation qualifiers
        "single" => Some("**single** - Single rotation (360°)".to_string()),
        "double" => Some("**double** - Double rotation (720°)".to_string()),
        "triple" => Some("**triple** - Triple rotation (1080°)".to_string()),
        "quad" => Some("**quad** - Quadruple rotation (1440°)".to_string()),

        // Edges
        "inside" => Some("**inside** - Inside edge\n\nSkate edge toward body center.".to_string()),
        "outside" => Some("**outside** - Outside edge\n\nSkate edge away from body center.".to_string()),
        "forward" => Some("**forward** - Forward direction".to_string()),
        "backward" => Some("**backward** - Backward direction".to_string()),

        _ => None,
    }
}
