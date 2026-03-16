// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Lexical tokens for Anvomidav.
//!
//! This module defines all tokens recognized by the lexer, using the `logos` crate
//! for efficient DFA-based tokenization.

use logos::Logos;
use ordered_float::OrderedFloat;
use std::fmt;
use std::hash::Hash;

/// Lexical token types for Anvomidav.
#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    // === Keywords ===
    #[token("program")]
    Program,

    #[token("segment")]
    Segment,

    #[token("sequence")]
    Sequence,

    #[token("element")]
    Element,

    #[token("jump")]
    Jump,

    #[token("spin")]
    Spin,

    #[token("step")]
    Step,

    #[token("lift")]
    Lift,

    #[token("death_spiral")]
    DeathSpiral,

    #[token("throw")]
    Throw,

    #[token("twist")]
    Twist,

    #[token("transition")]
    Transition,

    #[token("choreographic")]
    Choreographic,

    #[token("let")]
    Let,

    #[token("if")]
    If,

    #[token("then")]
    Then,

    #[token("else")]
    Else,

    #[token("match")]
    Match,

    #[token("with")]
    With,

    #[token("for")]
    For,

    #[token("in")]
    In,

    #[token("while")]
    While,

    #[token("do")]
    Do,

    #[token("end")]
    End,

    #[token("return")]
    Return,

    #[token("fn")]
    Fn,

    #[token("type")]
    Type,

    #[token("import")]
    Import,

    #[token("export")]
    Export,

    #[token("from")]
    From,

    #[token("as")]
    As,

    #[token("music")]
    Music,

    #[token("tempo")]
    Tempo,

    #[token("beat")]
    Beat,

    #[token("at")]
    At,

    #[token("to")]
    To,

    #[token("duration")]
    Duration,

    #[token("position")]
    Position,

    #[token("skater")]
    Skater,

    #[token("pairs")]
    Pairs,

    #[token("ice_dance")]
    IceDance,

    #[token("singles")]
    Singles,

    #[token("short")]
    Short,

    #[token("free")]
    Free,

    #[token("pattern")]
    Pattern,

    #[token("rhythm")]
    Rhythm,

    #[token("exhibition")]
    Exhibition,

    #[token("repeat")]
    Repeat,

    #[token("parallel")]
    Parallel,

    #[token("sync")]
    Sync,

    #[token("true")]
    True,

    #[token("false")]
    False,

    // === Jump Types ===
    #[token("axel")]
    Axel,

    #[token("lutz")]
    Lutz,

    #[token("flip")]
    Flip,

    #[token("loop")]
    Loop,

    #[token("salchow")]
    Salchow,

    #[token("toe_loop")]
    ToeLoop,

    #[token("euler")]
    Euler,

    // === Rotation Counts ===
    #[token("single")]
    Single,

    #[token("double")]
    Double,

    #[token("triple")]
    Triple,

    #[token("quad")]
    Quad,

    // === Spin Positions ===
    #[token("upright")]
    Upright,

    #[token("sit")]
    Sit,

    #[token("camel")]
    Camel,

    #[token("layback")]
    Layback,

    #[token("biellmann")]
    Biellmann,

    // === Edges ===
    #[token("LFO")]
    LFO,

    #[token("LFI")]
    LFI,

    #[token("LBO")]
    LBO,

    #[token("LBI")]
    LBI,

    #[token("RFO")]
    RFO,

    #[token("RFI")]
    RFI,

    #[token("RBO")]
    RBO,

    #[token("RBI")]
    RBI,

    // === Levels ===
    #[token("B")]
    LevelB,

    #[token("L1")]
    Level1,

    #[token("L2")]
    Level2,

    #[token("L3")]
    Level3,

    #[token("L4")]
    Level4,

    // === Lift Groups ===
    #[token("Gr1")]
    LiftGroup1,

    #[token("Gr2")]
    LiftGroup2,

    #[token("Gr3")]
    LiftGroup3,

    #[token("Gr4")]
    LiftGroup4,

    #[token("Gr5")]
    LiftGroup5,

    // === Choreographic Elements ===
    #[token("spiral")]
    Spiral,

    #[token("spread")]
    Spread,

    #[token("ina")]
    Ina,

    #[token("hydroblading")]
    Hydroblading,

    #[token("pivot")]
    Pivot,

    // === Operators ===
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("==")]
    EqEq,

    #[token("!=")]
    NotEq,

    #[token("<")]
    Lt,

    #[token("<=")]
    LtEq,

    #[token(">")]
    Gt,

    #[token(">=")]
    GtEq,

    #[token("&&")]
    AndAnd,

    #[token("||")]
    OrOr,

    #[token("!")]
    Not,

    #[token("=")]
    Eq,

    #[token("->")]
    Arrow,

    #[token("=>")]
    FatArrow,

    #[token(":")]
    Colon,

    #[token("::")]
    ColonColon,

    #[token(";")]
    Semi,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token("..")]
    DotDot,

    #[token("...")]
    DotDotDot,

    #[token("|")]
    Pipe,

    #[token("@")]
    At_,

    #[token("#")]
    Hash,

    #[token("?")]
    Question,

    // === Delimiters ===
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    // === Literals ===
    /// Integer literal
    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    Integer(i64),

    /// Float literal
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok().map(OrderedFloat))]
    Float(OrderedFloat<f64>),

    /// Time literal (mm:ss or hh:mm:ss format)
    #[regex(r"[0-9]+:[0-9]+(:[0-9]+)?(\.[0-9]+)?", |lex| lex.slice().to_string())]
    Time(String),

    /// String literal
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        // Remove surrounding quotes
        s[1..s.len()-1].to_string()
    })]
    String(String),

    /// Identifier
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 1, callback = |lex| lex.slice().to_string())]
    Ident(String),

    /// Documentation comment
    #[regex(r"///[^\n]*", |lex| lex.slice()[3..].trim().to_string())]
    DocComment(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Program => write!(f, "program"),
            Token::Segment => write!(f, "segment"),
            Token::Sequence => write!(f, "sequence"),
            Token::Element => write!(f, "element"),
            Token::Jump => write!(f, "jump"),
            Token::Spin => write!(f, "spin"),
            Token::Step => write!(f, "step"),
            Token::Lift => write!(f, "lift"),
            Token::DeathSpiral => write!(f, "death_spiral"),
            Token::Throw => write!(f, "throw"),
            Token::Twist => write!(f, "twist"),
            Token::Transition => write!(f, "transition"),
            Token::Choreographic => write!(f, "choreographic"),
            Token::Let => write!(f, "let"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::Match => write!(f, "match"),
            Token::With => write!(f, "with"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::While => write!(f, "while"),
            Token::Do => write!(f, "do"),
            Token::End => write!(f, "end"),
            Token::Return => write!(f, "return"),
            Token::Fn => write!(f, "fn"),
            Token::Type => write!(f, "type"),
            Token::Import => write!(f, "import"),
            Token::Export => write!(f, "export"),
            Token::From => write!(f, "from"),
            Token::As => write!(f, "as"),
            Token::Music => write!(f, "music"),
            Token::Tempo => write!(f, "tempo"),
            Token::Beat => write!(f, "beat"),
            Token::At => write!(f, "at"),
            Token::To => write!(f, "to"),
            Token::Duration => write!(f, "duration"),
            Token::Position => write!(f, "position"),
            Token::Skater => write!(f, "skater"),
            Token::Pairs => write!(f, "pairs"),
            Token::IceDance => write!(f, "ice_dance"),
            Token::Singles => write!(f, "singles"),
            Token::Short => write!(f, "short"),
            Token::Free => write!(f, "free"),
            Token::Pattern => write!(f, "pattern"),
            Token::Rhythm => write!(f, "rhythm"),
            Token::Exhibition => write!(f, "exhibition"),
            Token::Repeat => write!(f, "repeat"),
            Token::Parallel => write!(f, "parallel"),
            Token::Sync => write!(f, "sync"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Axel => write!(f, "axel"),
            Token::Lutz => write!(f, "lutz"),
            Token::Flip => write!(f, "flip"),
            Token::Loop => write!(f, "loop"),
            Token::Salchow => write!(f, "salchow"),
            Token::ToeLoop => write!(f, "toe_loop"),
            Token::Euler => write!(f, "euler"),
            Token::Single => write!(f, "single"),
            Token::Double => write!(f, "double"),
            Token::Triple => write!(f, "triple"),
            Token::Quad => write!(f, "quad"),
            Token::Upright => write!(f, "upright"),
            Token::Sit => write!(f, "sit"),
            Token::Camel => write!(f, "camel"),
            Token::Layback => write!(f, "layback"),
            Token::Biellmann => write!(f, "biellmann"),
            Token::LFO => write!(f, "LFO"),
            Token::LFI => write!(f, "LFI"),
            Token::LBO => write!(f, "LBO"),
            Token::LBI => write!(f, "LBI"),
            Token::RFO => write!(f, "RFO"),
            Token::RFI => write!(f, "RFI"),
            Token::RBO => write!(f, "RBO"),
            Token::RBI => write!(f, "RBI"),
            Token::LevelB => write!(f, "B"),
            Token::Level1 => write!(f, "L1"),
            Token::Level2 => write!(f, "L2"),
            Token::Level3 => write!(f, "L3"),
            Token::Level4 => write!(f, "L4"),
            Token::LiftGroup1 => write!(f, "Gr1"),
            Token::LiftGroup2 => write!(f, "Gr2"),
            Token::LiftGroup3 => write!(f, "Gr3"),
            Token::LiftGroup4 => write!(f, "Gr4"),
            Token::LiftGroup5 => write!(f, "Gr5"),
            Token::Spiral => write!(f, "spiral"),
            Token::Spread => write!(f, "spread"),
            Token::Ina => write!(f, "ina"),
            Token::Hydroblading => write!(f, "hydroblading"),
            Token::Pivot => write!(f, "pivot"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::EqEq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::LtEq => write!(f, "<="),
            Token::Gt => write!(f, ">"),
            Token::GtEq => write!(f, ">="),
            Token::AndAnd => write!(f, "&&"),
            Token::OrOr => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::Eq => write!(f, "="),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::Colon => write!(f, ":"),
            Token::ColonColon => write!(f, "::"),
            Token::Semi => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::DotDot => write!(f, ".."),
            Token::DotDotDot => write!(f, "..."),
            Token::Pipe => write!(f, "|"),
            Token::At_ => write!(f, "@"),
            Token::Hash => write!(f, "#"),
            Token::Question => write!(f, "?"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Integer(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::Time(s) => write!(f, "{}", s),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Ident(s) => write!(f, "{}", s),
            Token::DocComment(s) => write!(f, "/// {}", s),
        }
    }
}

impl Token {
    /// Returns true if this token is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::Program
                | Token::Segment
                | Token::Sequence
                | Token::Element
                | Token::Jump
                | Token::Spin
                | Token::Step
                | Token::Lift
                | Token::DeathSpiral
                | Token::Throw
                | Token::Twist
                | Token::Transition
                | Token::Choreographic
                | Token::Let
                | Token::If
                | Token::Then
                | Token::Else
                | Token::Match
                | Token::With
                | Token::For
                | Token::In
                | Token::While
                | Token::Do
                | Token::End
                | Token::Return
                | Token::Fn
                | Token::Type
                | Token::Import
                | Token::Export
                | Token::From
                | Token::As
                | Token::Music
                | Token::Tempo
                | Token::Beat
                | Token::At
                | Token::To
                | Token::Duration
                | Token::Position
                | Token::Skater
                | Token::Pairs
                | Token::IceDance
                | Token::Singles
                | Token::Short
                | Token::Free
                | Token::Pattern
                | Token::Repeat
                | Token::Parallel
                | Token::Sync
                | Token::True
                | Token::False
        )
    }

    /// Returns true if this token is a skating-specific keyword.
    pub fn is_skating_keyword(&self) -> bool {
        matches!(
            self,
            Token::Axel
                | Token::Lutz
                | Token::Flip
                | Token::Loop
                | Token::Salchow
                | Token::ToeLoop
                | Token::Euler
                | Token::Single
                | Token::Double
                | Token::Triple
                | Token::Quad
                | Token::Upright
                | Token::Sit
                | Token::Camel
                | Token::Layback
                | Token::Biellmann
                | Token::LFO
                | Token::LFI
                | Token::LBO
                | Token::LBI
                | Token::RFO
                | Token::RFI
                | Token::RBO
                | Token::RBI
                | Token::LevelB
                | Token::Level1
                | Token::Level2
                | Token::Level3
                | Token::Level4
        )
    }

    /// Returns true if this is an operator token.
    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::EqEq
                | Token::NotEq
                | Token::Lt
                | Token::LtEq
                | Token::Gt
                | Token::GtEq
                | Token::AndAnd
                | Token::OrOr
                | Token::Not
                | Token::Eq
        )
    }
}

/// A token with its span in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: std::ops::Range<usize>,
}

/// Lexer for Anvomidav source code.
pub struct Lexer<'src> {
    inner: logos::Lexer<'src, Token>,
}

impl<'src> Lexer<'src> {
    /// Create a new lexer for the given source code.
    pub fn new(source: &'src str) -> Self {
        Lexer {
            inner: Token::lexer(source),
        }
    }

    /// Tokenize the entire source, returning all tokens with their spans.
    pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>, LexError> {
        let lexer = Lexer::new(source);
        let mut tokens = Vec::new();

        for result in lexer {
            match result {
                Ok(token) => tokens.push(token),
                Err(e) => return Err(e),
            }
        }

        Ok(tokens)
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Result<SpannedToken, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(Ok(token)) => Some(Ok(SpannedToken {
                token,
                span: self.inner.span(),
            })),
            Some(Err(())) => Some(Err(LexError {
                span: self.inner.span(),
                message: format!("unexpected character: {:?}", self.inner.slice()),
            })),
            None => None,
        }
    }
}

/// Error during lexical analysis.
#[derive(Debug, Clone, thiserror::Error)]
#[error("lex error at {span:?}: {message}")]
pub struct LexError {
    pub span: std::ops::Range<usize>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let source = "program segment sequence element jump spin step";
        let tokens: Vec<_> = Lexer::new(source)
            .map(|r| r.unwrap().token)
            .collect();

        assert_eq!(tokens, vec![
            Token::Program,
            Token::Segment,
            Token::Sequence,
            Token::Element,
            Token::Jump,
            Token::Spin,
            Token::Step,
        ]);
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / == != < <= > >= && || !";
        let tokens: Vec<_> = Lexer::new(source)
            .map(|r| r.unwrap().token)
            .collect();

        assert_eq!(tokens, vec![
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::EqEq,
            Token::NotEq,
            Token::Lt,
            Token::LtEq,
            Token::Gt,
            Token::GtEq,
            Token::AndAnd,
            Token::OrOr,
            Token::Not,
        ]);
    }

    #[test]
    fn test_literals() {
        let source = r#"42 2.5 1:30 2:30:45 "hello world" my_var"#;
        let tokens: Vec<_> = Lexer::new(source)
            .map(|r| r.unwrap().token)
            .collect();

        assert_eq!(tokens, vec![
            Token::Integer(42),
            Token::Float(OrderedFloat(2.5)),
            Token::Time("1:30".to_string()),
            Token::Time("2:30:45".to_string()),
            Token::String("hello world".to_string()),
            Token::Ident("my_var".to_string()),
        ]);
    }

    #[test]
    fn test_edges() {
        let source = "LFO LFI LBO LBI RFO RFI RBO RBI";
        let tokens: Vec<_> = Lexer::new(source)
            .map(|r| r.unwrap().token)
            .collect();

        assert_eq!(tokens, vec![
            Token::LFO,
            Token::LFI,
            Token::LBO,
            Token::LBI,
            Token::RFO,
            Token::RFI,
            Token::RBO,
            Token::RBI,
        ]);
    }

    #[test]
    fn test_jump_types() {
        let source = "axel lutz flip loop salchow toe_loop euler";
        let tokens: Vec<_> = Lexer::new(source)
            .map(|r| r.unwrap().token)
            .collect();

        assert_eq!(tokens, vec![
            Token::Axel,
            Token::Lutz,
            Token::Flip,
            Token::Loop,
            Token::Salchow,
            Token::ToeLoop,
            Token::Euler,
        ]);
    }

    #[test]
    fn test_comments() {
        let source = "program // this is a comment\nsegment /* block comment */ element";
        let tokens: Vec<_> = Lexer::new(source)
            .map(|r| r.unwrap().token)
            .collect();

        assert_eq!(tokens, vec![
            Token::Program,
            Token::Segment,
            Token::Element,
        ]);
    }

    #[test]
    fn test_doc_comment() {
        let source = "/// This is documentation\nprogram";
        let tokens: Vec<_> = Lexer::new(source)
            .map(|r| r.unwrap().token)
            .collect();

        assert_eq!(tokens, vec![
            Token::DocComment("This is documentation".to_string()),
            Token::Program,
        ]);
    }

    #[test]
    fn test_skating_element() {
        let source = "jump triple axel at 1:30";
        let tokens: Vec<_> = Lexer::new(source)
            .map(|r| r.unwrap().token)
            .collect();

        assert_eq!(tokens, vec![
            Token::Jump,
            Token::Triple,
            Token::Axel,
            Token::At,
            Token::Time("1:30".to_string()),
        ]);
    }
}
