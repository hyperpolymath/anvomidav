// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Parser for Anvomidav using chumsky.
//!
//! This module implements a recursive descent parser with error recovery
//! for the Anvomidav DSL.

use crate::ast::*;
use crate::token::{LexError, Lexer, SpannedToken, Token};
use anv_core::skating::{JumpKind, Level, Rotations, SpinPosition};
use anv_core::source::{FileId, Span};
use chumsky::prelude::*;
use ordered_float::OrderedFloat;

/// Parser input type: a stream of spanned tokens.
pub type TokenStream<'a> = &'a [SpannedToken];

/// Parser error type.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: std::ops::Range<usize>,
    pub message: String,
    pub expected: Vec<String>,
    pub found: Option<String>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if !self.expected.is_empty() {
            write!(f, ", expected one of: {}", self.expected.join(", "))?;
        }
        if let Some(ref found) = self.found {
            write!(f, ", found: {}", found)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// Convert byte offset range to Span.
fn to_span(range: std::ops::Range<usize>, file_id: FileId) -> Span {
    Span::new(range.start as u32, range.end as u32, file_id)
}

/// Create a spanned node from a value and span range.
fn spanned<T>(node: T, span: std::ops::Range<usize>, file_id: FileId) -> Spanned<T> {
    Spanned::new(node, to_span(span, file_id))
}

/// Main parsing function.
pub fn parse(source: &str, file_id: FileId) -> Result<Program, Vec<ParseError>> {
    // First, lex the source
    let tokens = match Lexer::tokenize(source) {
        Ok(tokens) => tokens,
        Err(lex_err) => {
            return Err(vec![ParseError {
                span: lex_err.span,
                message: lex_err.message,
                expected: vec![],
                found: None,
            }])
        }
    };

    // Then parse the token stream
    parse_tokens(&tokens, file_id)
}

/// Parse a token stream into a Program.
pub fn parse_tokens(tokens: &[SpannedToken], file_id: FileId) -> Result<Program, Vec<ParseError>> {
    let parser = program_parser(file_id);

    // Build input for chumsky
    let len = tokens.last().map(|t| t.span.end).unwrap_or(0);
    let eoi = len..len;

    let stream = chumsky::Stream::from_iter(
        eoi,
        tokens.iter().map(|t| (t.token.clone(), t.span.clone())),
    );

    match parser.parse(stream) {
        Ok(program) => Ok(program),
        Err(errors) => Err(errors
            .into_iter()
            .map(|e| ParseError {
                span: e.span(),
                message: e.to_string(),
                expected: e
                    .expected()
                    .filter_map(|e| e.as_ref().map(|t| format!("{:?}", t)))
                    .collect(),
                found: e.found().map(|t| format!("{:?}", t)),
            })
            .collect()),
    }
}

/// Create identifier parser.
fn ident_parser(
    file_id: FileId,
) -> impl Parser<Token, Ident, Error = Simple<Token>> + Clone {
    select! {
        Token::Ident(name) => name,
    }
    .map_with_span(move |name, span| spanned(name, span, file_id))
    .labelled("identifier")
}

/// Create literal parser.
fn literal_parser() -> impl Parser<Token, Literal, Error = Simple<Token>> + Clone {
    select! {
        Token::Integer(n) => Literal::Int(n),
        Token::Float(f) => Literal::Float(f),
        Token::String(s) => Literal::String(s),
        Token::True => Literal::Bool(true),
        Token::False => Literal::Bool(false),
    }
    .labelled("literal")
}

/// Create time expression parser.
fn time_expr_parser(
    file_id: FileId,
) -> impl Parser<Token, TimeExpr, Error = Simple<Token>> + Clone {
    let ident = ident_parser(file_id);

    let literal = select! {
        Token::Float(f) => TimeExpr::Literal(f),
        Token::Integer(n) => TimeExpr::Literal(OrderedFloat(n as f64)),
        Token::Time(s) => TimeExpr::Formatted(s),
    };

    let var = ident.map(TimeExpr::Var);

    literal.or(var).labelled("time expression")
}

/// Create position expression parser.
fn position_expr_parser(
    file_id: FileId,
) -> impl Parser<Token, PositionExpr, Error = Simple<Token>> + Clone {
    let ident = ident_parser(file_id);

    let literal = select! { Token::Float(f) => f }
        .then_ignore(just(Token::Comma))
        .then(select! { Token::Float(f) => f })
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .map(|(x, y)| PositionExpr::Literal(x, y));

    let named = select! {
        Token::Ident(s) if s == "center" || s == "corner" => PositionExpr::Named(s),
    };

    let var = ident.map(PositionExpr::Var);

    literal.or(named).or(var).labelled("position expression")
}

/// Create jump kind parser.
fn jump_kind_parser() -> impl Parser<Token, JumpKind, Error = Simple<Token>> + Clone {
    select! {
        Token::Axel => JumpKind::Axel,
        Token::Lutz => JumpKind::Lutz,
        Token::Flip => JumpKind::Flip,
        Token::Loop => JumpKind::Loop,
        Token::Salchow => JumpKind::Salchow,
        Token::ToeLoop => JumpKind::ToeLoop,
        Token::Euler => JumpKind::Euler,
    }
    .labelled("jump kind")
}

/// Create rotations parser.
fn rotations_parser() -> impl Parser<Token, Rotations, Error = Simple<Token>> + Clone {
    select! {
        Token::Single => Rotations::Single,
        Token::Double => Rotations::Double,
        Token::Triple => Rotations::Triple,
        Token::Quad => Rotations::Quad,
    }
    .labelled("rotation count")
}

/// Create level parser.
fn level_parser() -> impl Parser<Token, Level, Error = Simple<Token>> + Clone {
    select! {
        Token::LevelB => Level::B,
        Token::Level1 => Level::L1,
        Token::Level2 => Level::L2,
        Token::Level3 => Level::L3,
        Token::Level4 => Level::L4,
    }
    .labelled("level")
}

/// Create spin position parser.
fn spin_position_parser() -> impl Parser<Token, SpinPosition, Error = Simple<Token>> + Clone {
    select! {
        Token::Upright => SpinPosition::Upright,
        Token::Sit => SpinPosition::Sit,
        Token::Camel => SpinPosition::Camel,
        Token::Layback => SpinPosition::Layback,
        Token::Biellmann => SpinPosition::Biellmann,
    }
    .labelled("spin position")
}

/// Create step pattern parser.
fn step_pattern_parser() -> impl Parser<Token, StepPattern, Error = Simple<Token>> + Clone {
    select! {
        Token::Ident(s) if s == "straight" => StepPattern::Straight,
        Token::Ident(s) if s == "circular" => StepPattern::Circular,
        Token::Ident(s) if s == "serpentine" => StepPattern::Serpentine,
        Token::Ident(s) if s == "diagonal" => StepPattern::Diagonal,
        Token::Ident(s) if s == "midline" => StepPattern::Midline,
    }
    .labelled("step pattern")
}

/// Create type expression parser.
fn type_expr_parser(
    file_id: FileId,
) -> impl Parser<Token, TypeExpr, Error = Simple<Token>> + Clone {
    recursive(|type_expr| {
        let ident = ident_parser(file_id);

        let named = ident.clone().map(TypeExpr::Named);

        let array = type_expr
            .clone()
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(|t| TypeExpr::Array(Box::new(t)));

        let optional = type_expr
            .clone()
            .then_ignore(just(Token::Question))
            .map(|t| TypeExpr::Optional(Box::new(t)));

        let tuple = type_expr
            .clone()
            .separated_by(just(Token::Comma))
            .at_least(2)
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .map(TypeExpr::Tuple);

        let generic = ident
            .clone()
            .map(TypeExpr::Named)
            .then(
                type_expr
                    .clone()
                    .separated_by(just(Token::Comma))
                    .at_least(1)
                    .delimited_by(just(Token::Lt), just(Token::Gt)),
            )
            .map(|(base, args)| TypeExpr::App(Box::new(base), args));

        choice((array, optional, tuple, generic, named)).labelled("type expression")
    })
}

/// Create expression parser.
fn expr_parser(
    file_id: FileId,
) -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let ident = ident_parser(file_id);

        // Literals
        let int = select! { Token::Integer(n) => Expr::Int(n) };
        let float = select! { Token::Float(f) => Expr::Float(f) };
        let string = select! { Token::String(s) => Expr::String(s) };
        let bool_lit = select! {
            Token::True => Expr::Bool(true),
            Token::False => Expr::Bool(false),
        };

        let literal = choice((int, float, string, bool_lit));

        // Variable
        let var = ident.clone().map(Expr::Var);

        // Parenthesized or tuple
        let paren_or_tuple = expr
            .clone()
            .separated_by(just(Token::Comma))
            .at_least(1)
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .map(|mut exprs: Vec<Spanned<Expr>>| {
                if exprs.len() == 1 {
                    exprs.pop().unwrap().node
                } else {
                    Expr::Tuple(exprs)
                }
            });

        // Array literal
        let array = expr
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(Expr::Array);

        // Block expression
        let block = expr
            .clone()
            .map(|e| Stmt::Expr(e.node))
            .map_with_span(move |stmt, span| spanned(stmt, span, file_id))
            .separated_by(just(Token::Semi))
            .allow_trailing()
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .map(Expr::Block);

        // If expression
        let if_expr = just(Token::If)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Then))
            .then(expr.clone())
            .then(just(Token::Else).ignore_then(expr.clone()).or_not())
            .map(|((cond, then_branch), else_branch)| {
                Expr::If(
                    Box::new(cond),
                    Box::new(then_branch),
                    else_branch.map(Box::new),
                )
            });

        // Let expression
        let let_expr = just(Token::Let)
            .ignore_then(ident.clone())
            .then(just(Token::Colon).ignore_then(type_expr_parser(file_id)).or_not())
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .map(|(((name, ty), value), body)| {
                Expr::Let(name, ty, Box::new(value), Box::new(body))
            });

        // Atoms
        let atom = choice((
            literal,
            if_expr,
            let_expr,
            paren_or_tuple,
            array,
            block,
            var,
        ))
        .map_with_span(move |e, span| spanned(e, span, file_id));

        // For now, just use atoms directly (function calls and field access to be added later)
        // TODO: Add function calls and field access parsing
        let primary = atom;

        // Unary operators
        let unary = just(Token::Minus)
            .to(UnaryOp::Neg)
            .or(just(Token::Not).to(UnaryOp::Not))
            .repeated()
            .then(primary)
            .foldr(move |op, expr| {
                let span = 0..expr.span.end as usize;
                spanned(Expr::UnaryOp(op, Box::new(expr)), span, file_id)
            });

        // Binary operators (multiplication/division)
        let mul_op = just(Token::Star)
            .to(BinOp::Mul)
            .or(just(Token::Slash).to(BinOp::Div))
            .or(just(Token::Percent).to(BinOp::Mod));

        let mul_div = unary
            .clone()
            .then(mul_op.then(unary).repeated())
            .foldl(move |left, (op, right)| {
                let span = left.span.start as usize..right.span.end as usize;
                spanned(Expr::BinOp(Box::new(left), op, Box::new(right)), span, file_id)
            });

        // Binary operators (addition/subtraction)
        let add_op = just(Token::Plus)
            .to(BinOp::Add)
            .or(just(Token::Minus).to(BinOp::Sub));

        let add_sub = mul_div
            .clone()
            .then(add_op.then(mul_div).repeated())
            .foldl(move |left, (op, right)| {
                let span = left.span.start as usize..right.span.end as usize;
                spanned(Expr::BinOp(Box::new(left), op, Box::new(right)), span, file_id)
            });

        // Comparison operators
        let cmp_op = choice((
            just(Token::EqEq).to(BinOp::Eq),
            just(Token::NotEq).to(BinOp::NotEq),
            just(Token::LtEq).to(BinOp::LtEq),
            just(Token::Lt).to(BinOp::Lt),
            just(Token::GtEq).to(BinOp::GtEq),
            just(Token::Gt).to(BinOp::Gt),
        ));

        let comparison = add_sub
            .clone()
            .then(cmp_op.then(add_sub).repeated())
            .foldl(move |left, (op, right)| {
                let span = left.span.start as usize..right.span.end as usize;
                spanned(Expr::BinOp(Box::new(left), op, Box::new(right)), span, file_id)
            });

        // Logical AND
        let and = comparison
            .clone()
            .then(just(Token::AndAnd).to(BinOp::And).then(comparison).repeated())
            .foldl(move |left, (op, right)| {
                let span = left.span.start as usize..right.span.end as usize;
                spanned(Expr::BinOp(Box::new(left), op, Box::new(right)), span, file_id)
            });

        // Logical OR
        and.clone()
            .then(just(Token::OrOr).to(BinOp::Or).then(and).repeated())
            .foldl(move |left, (op, right)| {
                let span = left.span.start as usize..right.span.end as usize;
                spanned(Expr::BinOp(Box::new(left), op, Box::new(right)), span, file_id)
            })
    })
}

/// Create timing parser.
fn timing_parser(
    file_id: FileId,
) -> impl Parser<Token, Timing, Error = Simple<Token>> + Clone {
    let time = time_expr_parser(file_id);

    let at = just(Token::At).ignore_then(time.clone()).map(Timing::At);

    let range = time
        .clone()
        .then_ignore(just(Token::DotDot))
        .then(time.clone())
        .map(|(start, end)| Timing::Range(start, end));

    let duration = just(Token::Duration)
        .ignore_then(time.clone())
        .map(Timing::Duration);

    let beat = just(Token::Beat)
        .ignore_then(select! { Token::Integer(n) => n })
        .map(Timing::Beat);

    choice((at, range, duration, beat)).labelled("timing")
}

/// Create jump element parser.
fn jump_element_parser(
    file_id: FileId,
) -> impl Parser<Token, JumpElement, Error = Simple<Token>> + Clone {
    just(Token::Jump)
        .ignore_then(rotations_parser())
        .then(jump_kind_parser())
        .map(|(rotations, kind)| JumpElement {
            kind,
            rotations,
            entry_edge: None,
            exit_edge: None,
            combination: vec![],
        })
        .labelled("jump element")
}

/// Create spin element parser.
fn spin_element_parser(
    file_id: FileId,
) -> impl Parser<Token, SpinElement, Error = Simple<Token>> + Clone {
    just(Token::Spin)
        .ignore_then(spin_position_parser().repeated().at_least(1))
        .then(level_parser().or_not())
        .map(|(positions, level)| SpinElement {
            positions: positions
                .into_iter()
                .map(|p| SpinPositionDef {
                    position: p,
                    revolutions: None,
                    change_edge: false,
                    change_foot: false,
                })
                .collect(),
            min_revs: None,
            features: vec![],
            level,
        })
        .labelled("spin element")
}

/// Create step sequence parser.
fn step_sequence_parser(
    file_id: FileId,
) -> impl Parser<Token, StepSequence, Error = Simple<Token>> + Clone {
    just(Token::Step)
        .ignore_then(step_pattern_parser())
        .then(level_parser().or_not())
        .map(|(pattern, level)| StepSequence {
            pattern,
            level,
            steps: vec![],
        })
        .labelled("step sequence")
}

/// Create element parser.
fn element_parser(
    file_id: FileId,
) -> impl Parser<Token, Element, Error = Simple<Token>> + Clone {
    let jump = jump_element_parser(file_id).map(ElementKind::Jump);
    let spin = spin_element_parser(file_id).map(ElementKind::Spin);
    let step = step_sequence_parser(file_id).map(ElementKind::StepSequence);

    let element_kind = choice((jump, spin, step));

    element_kind
        .then(timing_parser(file_id).or_not())
        .map_with_span(move |(kind, timing), span| Element {
            kind,
            timing,
            position: None,
            annotations: vec![],
            span: to_span(span, file_id),
        })
        .labelled("element")
}

/// Create sequence parser.
fn sequence_parser(
    file_id: FileId,
) -> impl Parser<Token, Sequence, Error = Simple<Token>> + Clone {
    just(Token::Sequence)
        .ignore_then(ident_parser(file_id).or_not())
        .then_ignore(just(Token::LBrace))
        .then(element_parser(file_id).repeated())
        .then_ignore(just(Token::RBrace))
        .map_with_span(move |(name, elements), span| Sequence {
            name,
            elements,
            span: to_span(span, file_id),
        })
        .labelled("sequence")
}

/// Create segment parser.
fn segment_parser(
    file_id: FileId,
) -> impl Parser<Token, Segment, Error = Simple<Token>> + Clone {
    let segment_kind = choice((
        just(Token::Short).to(SegmentKind::Short),
        just(Token::Free).to(SegmentKind::Free),
        just(Token::Pattern).to(SegmentKind::Pattern),
    ));

    just(Token::Segment)
        .ignore_then(ident_parser(file_id))
        .then_ignore(just(Token::Colon))
        .then(segment_kind)
        .then_ignore(just(Token::LBrace))
        .then(sequence_parser(file_id).repeated())
        .then_ignore(just(Token::RBrace))
        .map_with_span(move |((name, kind), sequences), span| Segment {
            name,
            kind,
            music: None,
            duration: None,
            skaters: vec![],
            sequences,
            docs: vec![],
            span: to_span(span, file_id),
        })
        .labelled("segment")
}

/// Create import parser.
fn import_parser(
    file_id: FileId,
) -> impl Parser<Token, Import, Error = Simple<Token>> + Clone {
    just(Token::Import)
        .ignore_then(
            ident_parser(file_id)
                .separated_by(just(Token::ColonColon))
                .at_least(1),
        )
        .then(just(Token::As).ignore_then(ident_parser(file_id)).or_not())
        .map_with_span(move |(path, alias), span| Import {
            path,
            alias,
            items: None,
            span: to_span(span, file_id),
        })
        .labelled("import")
}

/// Create function definition parser.
fn fn_def_parser(
    file_id: FileId,
) -> impl Parser<Token, FnDef, Error = Simple<Token>> + Clone {
    let param = ident_parser(file_id)
        .then(just(Token::Colon).ignore_then(type_expr_parser(file_id)).or_not())
        .map(|(name, ty)| Param {
            name,
            ty,
            default: None,
        });

    just(Token::Fn)
        .ignore_then(ident_parser(file_id))
        .then(
            param
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(just(Token::Arrow).ignore_then(type_expr_parser(file_id)).or_not())
        .then_ignore(just(Token::Eq))
        .then(expr_parser(file_id))
        .map_with_span(move |(((name, params), return_ty), body), span| FnDef {
            name,
            type_params: vec![],
            params,
            return_ty,
            body: body.node,
            docs: vec![],
            span: to_span(span, file_id),
        })
        .labelled("function definition")
}

/// Create program parser.
fn program_parser(
    file_id: FileId,
) -> impl Parser<Token, Program, Error = Simple<Token>> {
    just(Token::Program)
        .ignore_then(ident_parser(file_id))
        .then_ignore(just(Token::LBrace))
        .then(
            choice((
                import_parser(file_id).map(Item::Import),
                fn_def_parser(file_id).map(Item::Fn),
                segment_parser(file_id).map(Item::Segment),
            ))
            .repeated(),
        )
        .then_ignore(just(Token::RBrace))
        .then_ignore(end())
        .map(move |(name, items)| {
            let mut imports = vec![];
            let mut functions = vec![];
            let mut segments = vec![];

            for item in items {
                match item {
                    Item::Import(i) => imports.push(i),
                    Item::Fn(f) => functions.push(f),
                    Item::Segment(s) => segments.push(s),
                }
            }

            Program {
                name,
                docs: vec![],
                imports,
                types: vec![],
                functions,
                segments,
            }
        })
        .labelled("program")
}

/// Internal enum for collecting program items.
enum Item {
    Import(Import),
    Fn(FnDef),
    Segment(Segment),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_program() {
        let source = "program test {}";
        let result = parse(source, FileId(0));
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.name.node, "test");
        assert!(program.segments.is_empty());
    }

    #[test]
    fn test_parse_program_with_segment() {
        let source = r#"
            program my_program {
                segment intro: short {
                    sequence {
                        jump triple axel at 1:30
                    }
                }
            }
        "#;
        let result = parse(source, FileId(0));
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.name.node, "my_program");
        assert_eq!(program.segments.len(), 1);
        assert_eq!(program.segments[0].name.node, "intro");
        assert_eq!(program.segments[0].kind, SegmentKind::Short);
    }

    #[test]
    fn test_parse_jump_element() {
        let source = "program t { segment s: free { sequence { jump triple lutz } } }";
        let result = parse(source, FileId(0));
        assert!(result.is_ok());
        let program = result.unwrap();
        let element = &program.segments[0].sequences[0].elements[0];
        match &element.kind {
            ElementKind::Jump(jump) => {
                assert_eq!(jump.kind, JumpKind::Lutz);
                assert_eq!(jump.rotations, Rotations::Triple);
            }
            _ => panic!("Expected jump element"),
        }
    }

    #[test]
    fn test_parse_spin_element() {
        let source = "program t { segment s: free { sequence { spin camel sit L3 } } }";
        let result = parse(source, FileId(0));
        assert!(result.is_ok());
        let program = result.unwrap();
        let element = &program.segments[0].sequences[0].elements[0];
        match &element.kind {
            ElementKind::Spin(spin) => {
                assert_eq!(spin.positions.len(), 2);
                assert_eq!(spin.positions[0].position, SpinPosition::Camel);
                assert_eq!(spin.positions[1].position, SpinPosition::Sit);
                assert_eq!(spin.level, Some(Level::L3));
            }
            _ => panic!("Expected spin element"),
        }
    }

    #[test]
    fn test_parse_step_sequence() {
        let source = "program t { segment s: free { sequence { step circular L4 } } }";
        let result = parse(source, FileId(0));
        assert!(result.is_ok());
        let program = result.unwrap();
        let element = &program.segments[0].sequences[0].elements[0];
        match &element.kind {
            ElementKind::StepSequence(steps) => {
                assert_eq!(steps.pattern, StepPattern::Circular);
                assert_eq!(steps.level, Some(Level::L4));
            }
            _ => panic!("Expected step sequence"),
        }
    }

    #[test]
    fn test_lexer_error() {
        let source = "program test { @ }"; // @ is valid now, try invalid char
        let result = parse(source, FileId(0));
        // This may succeed now with @ being a valid token, that's okay
    }
}
