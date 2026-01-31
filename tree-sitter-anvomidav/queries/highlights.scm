; SPDX-FileCopyrightText: 2025 hyperpolymath
; SPDX-License-Identifier: PMPL-1.0-or-later

; Anvomidav syntax highlighting queries for tree-sitter

; === Comments ===
(doc_comment) @comment.documentation
(line_comment) @comment

; === Keywords ===
[
  "program"
  "segment"
  "sequence"
  "import"
  "fn"
  "let"
  "in"
  "if"
  "then"
  "else"
  "at"
  "duration"
  "beat"
  "as"
] @keyword

; === Segment kinds ===
(segment_kind) @keyword.storage

; === Element keywords ===
[
  "jump"
  "spin"
  "step"
  "lift"
  "throw"
  "twist"
  "death_spiral"
  "choreographic"
] @keyword.function

; === Skating-specific constants ===
(rotation) @constant
(jump_kind) @constant.builtin
(spin_position) @constant.builtin
(step_pattern) @constant.builtin
(lift_group) @constant.builtin
(edge) @constant.builtin
(choreographic_kind) @constant.builtin
(level) @constant

; === Types ===
(type) @type

; === Literals ===
(integer) @number
(float) @number.float
(time_literal) @number
(string) @string
(escape_sequence) @string.escape
(boolean) @constant.builtin.boolean

; === Identifiers ===
(identifier) @variable

; Function names
(function_definition
  name: (identifier) @function)

; Program name
(program_definition
  name: (identifier) @namespace)

; Segment name
(segment
  name: (identifier) @label)

; Sequence name
(sequence
  name: (identifier) @label)

; Parameter names
(parameter
  (identifier) @variable.parameter)

; === Operators ===
(binary_operator) @operator
(unary_operator) @operator

; === Punctuation ===
[
  "{"
  "}"
  "("
  ")"
  "["
  "]"
] @punctuation.bracket

[
  ":"
  "::"
  ";"
  ","
] @punctuation.delimiter

[
  "->"
  "="
] @punctuation.special
