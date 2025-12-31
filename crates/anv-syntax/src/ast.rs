// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Abstract Syntax Tree for Anvomidav.
//!
//! This module defines the AST produced by parsing Anvomidav source code.

use anv_core::skating::{Curve, Edge, JumpKind, Level, LiftGroup, Rotations, SpinPosition};
use anv_core::source::Span;
use ordered_float::OrderedFloat;
use std::fmt;

/// A node with source location information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Spanned { node, span }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}

/// An identifier.
pub type Ident = Spanned<String>;

/// A program is the top-level compilation unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Program name.
    pub name: Ident,
    /// Documentation comments.
    pub docs: Vec<String>,
    /// Import declarations.
    pub imports: Vec<Import>,
    /// Type definitions.
    pub types: Vec<TypeDef>,
    /// Function definitions.
    pub functions: Vec<FnDef>,
    /// Segment definitions (the main content).
    pub segments: Vec<Segment>,
}

/// An import declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// Module path.
    pub path: Vec<Ident>,
    /// Optional alias.
    pub alias: Option<Ident>,
    /// Specific items to import (None means all).
    pub items: Option<Vec<ImportItem>>,
    pub span: Span,
}

/// An imported item.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportItem {
    pub name: Ident,
    pub alias: Option<Ident>,
}

/// A type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    /// Type name.
    pub name: Ident,
    /// Type parameters.
    pub params: Vec<TypeParam>,
    /// Type body.
    pub body: TypeBody,
    /// Documentation.
    pub docs: Vec<String>,
    pub span: Span,
}

/// A type parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: Ident,
    pub bounds: Vec<TypeExpr>,
}

/// The body of a type definition.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeBody {
    /// Type alias.
    Alias(TypeExpr),
    /// Record type.
    Record(Vec<Field>),
    /// Enum/variant type.
    Enum(Vec<Variant>),
}

/// A record field.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: Ident,
    pub ty: TypeExpr,
    pub docs: Vec<String>,
}

/// An enum variant.
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: Ident,
    pub fields: Option<Vec<Field>>,
    pub docs: Vec<String>,
}

/// A type expression.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// Named type.
    Named(Ident),
    /// Generic type application.
    App(Box<TypeExpr>, Vec<TypeExpr>),
    /// Function type.
    Fn(Vec<TypeExpr>, Box<TypeExpr>),
    /// Tuple type.
    Tuple(Vec<TypeExpr>),
    /// Array type.
    Array(Box<TypeExpr>),
    /// Optional type.
    Optional(Box<TypeExpr>),
    /// Refinement type: { x: T | P(x) }
    Refinement {
        base: Box<TypeExpr>,
        var: Ident,
        predicate: Box<Expr>,
    },
}

/// A function definition.
#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    /// Function name.
    pub name: Ident,
    /// Type parameters.
    pub type_params: Vec<TypeParam>,
    /// Parameters.
    pub params: Vec<Param>,
    /// Return type.
    pub return_ty: Option<TypeExpr>,
    /// Function body.
    pub body: Expr,
    /// Documentation.
    pub docs: Vec<String>,
    pub span: Span,
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub default: Option<Expr>,
}

/// A program segment (e.g., short program, free skate).
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// Segment name.
    pub name: Ident,
    /// Segment kind.
    pub kind: SegmentKind,
    /// Music information.
    pub music: Option<MusicDef>,
    /// Duration constraints.
    pub duration: Option<DurationDef>,
    /// Skater definitions (for pairs/ice dance).
    pub skaters: Vec<SkaterDef>,
    /// Element sequences.
    pub sequences: Vec<Sequence>,
    /// Documentation.
    pub docs: Vec<String>,
    pub span: Span,
}

/// Segment kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    /// Short program (singles/pairs).
    Short,
    /// Free skate (singles/pairs).
    Free,
    /// Pattern dance (ice dance).
    Pattern,
    /// Rhythm dance (ice dance).
    Rhythm,
    /// Exhibition/gala performance.
    Exhibition,
}

impl fmt::Display for SegmentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SegmentKind::Short => write!(f, "short"),
            SegmentKind::Free => write!(f, "free"),
            SegmentKind::Pattern => write!(f, "pattern"),
            SegmentKind::Rhythm => write!(f, "rhythm"),
            SegmentKind::Exhibition => write!(f, "exhibition"),
        }
    }
}

/// Music definition.
#[derive(Debug, Clone, PartialEq)]
pub struct MusicDef {
    /// Music file or reference.
    pub source: String,
    /// Tempo in BPM.
    pub tempo: Option<f64>,
    /// Beat structure.
    pub beats: Option<Vec<BeatMark>>,
    pub span: Span,
}

/// A beat marker.
#[derive(Debug, Clone, PartialEq)]
pub struct BeatMark {
    pub time: TimeExpr,
    pub beat: i64,
}

/// Duration definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DurationDef {
    /// Minimum duration.
    pub min: Option<TimeExpr>,
    /// Maximum duration.
    pub max: Option<TimeExpr>,
    pub span: Span,
}

/// Skater definition (for pairs/ice dance).
#[derive(Debug, Clone, PartialEq)]
pub struct SkaterDef {
    pub name: Ident,
    pub role: Option<SkaterRole>,
    pub span: Span,
}

/// Skater role in pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkaterRole {
    Lead,
    Follow,
}

/// A sequence of elements.
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub name: Option<Ident>,
    pub elements: Vec<Element>,
    pub span: Span,
}

/// A skating element.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub kind: ElementKind,
    pub timing: Option<Timing>,
    pub position: Option<PositionExpr>,
    pub annotations: Vec<Annotation>,
    pub span: Span,
}

/// Element kind.
#[derive(Debug, Clone, PartialEq)]
pub enum ElementKind {
    /// Jump element.
    Jump(JumpElement),
    /// Spin element.
    Spin(SpinElement),
    /// Step sequence.
    StepSequence(StepSequence),
    /// Lift (pairs).
    Lift(LiftElement),
    /// Death spiral (pairs).
    DeathSpiral(DeathSpiralElement),
    /// Throw jump (pairs).
    Throw(ThrowElement),
    /// Twist lift (pairs).
    Twist(TwistElement),
    /// Transition.
    Transition(TransitionElement),
    /// Choreographic element.
    Choreographic(ChoreographicElement),
    /// Pattern (ice dance).
    Pattern(PatternElement),
    /// Parallel execution (pairs/ice dance).
    Parallel(ParallelElement),
    /// Synchronized execution.
    Sync(SyncElement),
}

/// A jump element.
#[derive(Debug, Clone, PartialEq)]
pub struct JumpElement {
    /// Jump kind.
    pub kind: JumpKind,
    /// Rotation count.
    pub rotations: Rotations,
    /// Entry edge (if specified).
    pub entry_edge: Option<Edge>,
    /// Exit edge (if specified).
    pub exit_edge: Option<Edge>,
    /// Combination (following jumps).
    pub combination: Vec<JumpElement>,
}

/// A spin element.
#[derive(Debug, Clone, PartialEq)]
pub struct SpinElement {
    /// Spin positions.
    pub positions: Vec<SpinPositionDef>,
    /// Minimum revolutions.
    pub min_revs: Option<i64>,
    /// Level features.
    pub features: Vec<SpinFeature>,
    /// Target level.
    pub level: Option<Level>,
}

/// A spin position definition.
#[derive(Debug, Clone, PartialEq)]
pub struct SpinPositionDef {
    pub position: SpinPosition,
    pub revolutions: Option<i64>,
    pub change_edge: bool,
    pub change_foot: bool,
}

/// Spin level features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinFeature {
    DifficultVariation,
    ChangeOfFoot,
    ChangeOfEdge,
    JumpEntry,
    FlyingEntry,
    BackEntry,
    DifficultPosition,
    TravelSuppressed,
}

/// A step sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct StepSequence {
    /// Pattern type.
    pub pattern: StepPattern,
    /// Target level.
    pub level: Option<Level>,
    /// Individual steps.
    pub steps: Vec<StepDef>,
}

/// Step sequence pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepPattern {
    Straight,
    Circular,
    Serpentine,
    Diagonal,
    Midline,
}

impl fmt::Display for StepPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepPattern::Straight => write!(f, "straight"),
            StepPattern::Circular => write!(f, "circular"),
            StepPattern::Serpentine => write!(f, "serpentine"),
            StepPattern::Diagonal => write!(f, "diagonal"),
            StepPattern::Midline => write!(f, "midline"),
        }
    }
}

/// A step definition.
#[derive(Debug, Clone, PartialEq)]
pub struct StepDef {
    pub edge: Edge,
    pub curve: Option<Curve>,
    pub turn: Option<TurnKind>,
}

/// Turn types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnKind {
    ThreeTurn,
    Bracket,
    Rocker,
    Counter,
    Mohawk,
    Choctaw,
    Twizzle,
}

/// A lift element (pairs).
#[derive(Debug, Clone, PartialEq)]
pub struct LiftElement {
    /// Lift group.
    pub group: LiftGroup,
    /// Difficulty level.
    pub level: Option<Level>,
    /// Entry method.
    pub entry: Option<String>,
    /// Exit method.
    pub exit: Option<String>,
}

/// A death spiral element (pairs).
#[derive(Debug, Clone, PartialEq)]
pub struct DeathSpiralElement {
    /// Edge (BI, FI, BO, FO).
    pub edge: Edge,
    /// Difficulty level.
    pub level: Option<Level>,
}

/// A throw jump element (pairs).
#[derive(Debug, Clone, PartialEq)]
pub struct ThrowElement {
    /// Jump kind.
    pub kind: JumpKind,
    /// Rotation count.
    pub rotations: Rotations,
}

/// A twist lift element (pairs).
#[derive(Debug, Clone, PartialEq)]
pub struct TwistElement {
    /// Rotation count.
    pub rotations: Rotations,
    /// Difficulty level.
    pub level: Option<Level>,
}

/// A transition element.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionElement {
    /// Transition description.
    pub description: Option<String>,
    /// Movement quality.
    pub quality: Option<MovementQuality>,
}

/// Movement quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementQuality {
    Gliding,
    Stroking,
    Crossovers,
    EdgeWork,
    Footwork,
}

/// A choreographic element.
#[derive(Debug, Clone, PartialEq)]
pub struct ChoreographicElement {
    /// Element kind.
    pub kind: ChoreographicKind,
    /// Description.
    pub description: Option<String>,
}

/// Choreographic element kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoreographicKind {
    Spiral,
    Spread,
    Ina,
    Hydroblading,
    Pivot,
    Choreographic,
}

/// A pattern element (ice dance).
#[derive(Debug, Clone, PartialEq)]
pub struct PatternElement {
    /// Pattern name.
    pub name: String,
    /// Key points.
    pub key_points: Vec<KeyPoint>,
}

/// A key point in a pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyPoint {
    pub position: PositionExpr,
    pub edge: Edge,
}

/// Parallel execution for pairs/ice dance.
#[derive(Debug, Clone, PartialEq)]
pub struct ParallelElement {
    pub branches: Vec<(Ident, Vec<Element>)>,
}

/// Synchronized execution.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncElement {
    pub elements: Vec<Element>,
}

/// Timing specification.
#[derive(Debug, Clone, PartialEq)]
pub enum Timing {
    /// At specific time.
    At(TimeExpr),
    /// Range of time.
    Range(TimeExpr, TimeExpr),
    /// Duration only.
    Duration(TimeExpr),
    /// Beat-relative.
    Beat(i64),
    /// After previous.
    After(TimeExpr),
}

/// Time expression.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeExpr {
    /// Literal time (seconds).
    Literal(OrderedFloat<f64>),
    /// Time from string (mm:ss format).
    Formatted(String),
    /// Variable reference.
    Var(Ident),
    /// Binary operation.
    BinOp(Box<TimeExpr>, TimeBinOp, Box<TimeExpr>),
}

/// Time binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeBinOp {
    Add,
    Sub,
}

/// Position expression.
#[derive(Debug, Clone, PartialEq)]
pub enum PositionExpr {
    /// Literal (x, y) in meters.
    Literal(OrderedFloat<f64>, OrderedFloat<f64>),
    /// Named position (center, corner, etc.).
    Named(String),
    /// Variable reference.
    Var(Ident),
    /// Relative to another position.
    Relative(Box<PositionExpr>, OrderedFloat<f64>, OrderedFloat<f64>),
}

/// An annotation on an element.
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub name: Ident,
    pub args: Vec<Expr>,
}

/// An expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal.
    Int(i64),
    /// Float literal.
    Float(OrderedFloat<f64>),
    /// String literal.
    String(String),
    /// Boolean literal.
    Bool(bool),
    /// Time literal.
    Time(TimeExpr),
    /// Position literal.
    Position(PositionExpr),
    /// Variable reference.
    Var(Ident),
    /// Binary operation.
    BinOp(Box<Spanned<Expr>>, BinOp, Box<Spanned<Expr>>),
    /// Unary operation.
    UnaryOp(UnaryOp, Box<Spanned<Expr>>),
    /// Function call.
    Call(Box<Spanned<Expr>>, Vec<Spanned<Expr>>),
    /// Field access.
    Field(Box<Spanned<Expr>>, Ident),
    /// Index access.
    Index(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    /// If expression.
    If(Box<Spanned<Expr>>, Box<Spanned<Expr>>, Option<Box<Spanned<Expr>>>),
    /// Match expression.
    Match(Box<Spanned<Expr>>, Vec<MatchArm>),
    /// Let binding.
    Let(Ident, Option<TypeExpr>, Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    /// Block expression.
    Block(Vec<Spanned<Stmt>>),
    /// Lambda expression.
    Lambda(Vec<Param>, Box<Spanned<Expr>>),
    /// Tuple expression.
    Tuple(Vec<Spanned<Expr>>),
    /// Array expression.
    Array(Vec<Spanned<Expr>>),
    /// Record expression.
    Record(Vec<(Ident, Spanned<Expr>)>),
    /// Element expression (inline skating element).
    Element(Box<Element>),
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::NotEq => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::LtEq => write!(f, "<="),
            BinOp::Gt => write!(f, ">"),
            BinOp::GtEq => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
        }
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

/// A match arm.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Spanned<Expr>>,
    pub body: Spanned<Expr>,
}

/// A pattern for matching.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Wildcard pattern.
    Wildcard,
    /// Variable binding.
    Var(Ident),
    /// Literal pattern.
    Literal(Literal),
    /// Constructor pattern.
    Constructor(Ident, Vec<Pattern>),
    /// Tuple pattern.
    Tuple(Vec<Pattern>),
    /// Record pattern.
    Record(Vec<(Ident, Pattern)>),
    /// Or pattern.
    Or(Vec<Pattern>),
    /// Guard pattern.
    Guard(Box<Pattern>, Spanned<Expr>),
}

/// A literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(OrderedFloat<f64>),
    String(String),
    Bool(bool),
}

/// A statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Expression statement.
    Expr(Expr),
    /// Let binding.
    Let(Ident, Option<TypeExpr>, Expr),
    /// Assignment.
    Assign(Spanned<Expr>, Expr),
    /// Return statement.
    Return(Option<Expr>),
    /// For loop.
    For(Ident, Expr, Vec<Spanned<Stmt>>),
    /// While loop.
    While(Expr, Vec<Spanned<Stmt>>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use anv_core::source::FileId;

    fn dummy_span() -> Span {
        Span::new(0, 0, FileId(0))
    }

    fn ident(name: &str) -> Ident {
        Spanned::new(name.to_string(), dummy_span())
    }

    #[test]
    fn test_spanned() {
        let spanned = Spanned::new(42, dummy_span());
        assert_eq!(spanned.node, 42);
    }

    #[test]
    fn test_segment_kind_display() {
        assert_eq!(format!("{}", SegmentKind::Short), "short");
        assert_eq!(format!("{}", SegmentKind::Free), "free");
        assert_eq!(format!("{}", SegmentKind::Pattern), "pattern");
    }

    #[test]
    fn test_jump_element() {
        let jump = JumpElement {
            kind: JumpKind::Axel,
            rotations: Rotations::Triple,
            entry_edge: None,
            exit_edge: None,
            combination: vec![],
        };
        assert_eq!(jump.kind, JumpKind::Axel);
        assert_eq!(jump.rotations, Rotations::Triple);
    }

    #[test]
    fn test_program_structure() {
        let program = Program {
            name: ident("test_program"),
            docs: vec![],
            imports: vec![],
            types: vec![],
            functions: vec![],
            segments: vec![],
        };
        assert_eq!(program.name.node, "test_program");
    }
}
