// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

/**
 * Tree-sitter grammar for Anvomidav figure skating DSL.
 */
module.exports = grammar({
  name: "anvomidav",

  extras: ($) => [/\s/, $.line_comment],

  word: ($) => $.identifier,

  rules: {
    // Top-level program
    source_file: ($) =>
      seq(repeat($.doc_comment), $.program_definition),

    program_definition: ($) =>
      seq(
        "program",
        field("name", $.identifier),
        "{",
        repeat(choice($.import_declaration, $.function_definition, $.segment)),
        "}"
      ),

    // === Comments ===
    doc_comment: ($) => token(seq("///", /.*/)),
    line_comment: ($) => token(seq("//", /.*/)),

    // === Imports ===
    import_declaration: ($) =>
      seq(
        "import",
        $.module_path,
        optional(seq("as", $.identifier))
      ),

    module_path: ($) => sep1($.identifier, "::"),

    // === Functions ===
    function_definition: ($) =>
      seq(
        "fn",
        field("name", $.identifier),
        "(",
        optional($.parameter_list),
        ")",
        optional(seq("->", $.type)),
        "=",
        $.expression
      ),

    parameter_list: ($) => sep1($.parameter, ","),

    parameter: ($) =>
      seq($.identifier, optional(seq(":", $.type))),

    // === Types ===
    type: ($) =>
      choice(
        $.identifier,
        $.array_type,
        $.optional_type,
        $.tuple_type
      ),

    array_type: ($) => seq("[", $.type, "]"),
    optional_type: ($) => seq($.type, "?"),
    tuple_type: ($) => seq("(", sep1($.type, ","), ")"),

    // === Segments ===
    segment: ($) =>
      seq(
        "segment",
        field("name", $.identifier),
        ":",
        field("kind", $.segment_kind),
        "{",
        repeat($.sequence),
        "}"
      ),

    segment_kind: ($) =>
      choice("short", "free", "pattern", "rhythm", "exhibition"),

    // === Sequences ===
    sequence: ($) =>
      seq(
        "sequence",
        optional(field("name", $.identifier)),
        "{",
        repeat($.element),
        "}"
      ),

    // === Elements ===
    element: ($) =>
      seq(
        choice(
          $.jump_element,
          $.spin_element,
          $.step_element,
          $.lift_element,
          $.throw_element,
          $.twist_element,
          $.death_spiral_element,
          $.choreographic_element
        ),
        optional($.timing)
      ),

    // Jump element
    jump_element: ($) =>
      seq("jump", $.rotation, $.jump_kind),

    rotation: ($) => choice("single", "double", "triple", "quad"),

    jump_kind: ($) =>
      choice("axel", "lutz", "flip", "loop", "salchow", "toe_loop", "euler"),

    // Spin element
    spin_element: ($) =>
      seq("spin", repeat1($.spin_position), optional($.level)),

    spin_position: ($) =>
      choice("upright", "sit", "camel", "layback", "biellmann"),

    // Step element
    step_element: ($) =>
      seq("step", $.step_pattern, optional($.level)),

    step_pattern: ($) =>
      choice("straight", "circular", "serpentine", "diagonal", "midline"),

    // Lift element (pairs)
    lift_element: ($) =>
      seq("lift", $.lift_group, optional($.level)),

    lift_group: ($) => choice("Gr1", "Gr2", "Gr3", "Gr4", "Gr5"),

    // Throw element (pairs)
    throw_element: ($) =>
      seq("throw", $.rotation, $.jump_kind),

    // Twist element (pairs)
    twist_element: ($) =>
      seq("twist", $.rotation, optional($.level)),

    // Death spiral element (pairs)
    death_spiral_element: ($) =>
      seq("death_spiral", $.edge, optional($.level)),

    edge: ($) =>
      choice("LFO", "LFI", "LBO", "LBI", "RFO", "RFI", "RBO", "RBI"),

    // Choreographic element
    choreographic_element: ($) =>
      seq("choreographic", $.choreographic_kind),

    choreographic_kind: ($) =>
      choice("spiral", "spread", "ina", "hydroblading", "pivot"),

    // Level indicator
    level: ($) => choice("B", "L1", "L2", "L3", "L4"),

    // Timing
    timing: ($) =>
      choice(
        seq("at", $.time_expr),
        seq("duration", $.time_expr),
        seq("beat", $.integer)
      ),

    time_expr: ($) =>
      choice($.time_literal, $.number, $.identifier),

    time_literal: ($) => /[0-9]+:[0-9]+(:[0-9]+)?(\.[0-9]+)?/,

    // === Expressions ===
    expression: ($) =>
      choice(
        $.identifier,
        $.number,
        $.string,
        $.boolean,
        $.binary_expression,
        $.unary_expression,
        $.paren_expression,
        $.block_expression,
        $.if_expression,
        $.let_expression
      ),

    binary_expression: ($) =>
      prec.left(
        1,
        seq($.expression, $.binary_operator, $.expression)
      ),

    unary_expression: ($) =>
      prec(2, seq($.unary_operator, $.expression)),

    paren_expression: ($) => seq("(", $.expression, ")"),

    block_expression: ($) =>
      seq("{", repeat(seq($.expression, optional(";"))), "}"),

    if_expression: ($) =>
      seq(
        "if",
        $.expression,
        "then",
        $.expression,
        optional(seq("else", $.expression))
      ),

    let_expression: ($) =>
      seq(
        "let",
        $.identifier,
        optional(seq(":", $.type)),
        "=",
        $.expression,
        "in",
        $.expression
      ),

    binary_operator: ($) =>
      choice(
        "+", "-", "*", "/", "%",
        "==", "!=", "<", ">", "<=", ">=",
        "&&", "||"
      ),

    unary_operator: ($) => choice("-", "!"),

    // === Literals ===
    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    number: ($) => choice($.integer, $.float),
    integer: ($) => /[0-9]+/,
    float: ($) => /[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?/,

    string: ($) =>
      seq('"', repeat(choice(/[^"\\]/, $.escape_sequence)), '"'),

    escape_sequence: ($) => /\\./,

    boolean: ($) => choice("true", "false"),
  },
});

/**
 * Separate by delimiter (1 or more).
 */
function sep1(rule, delimiter) {
  return seq(rule, repeat(seq(delimiter, rule)));
}
