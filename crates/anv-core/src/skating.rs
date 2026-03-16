// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Figure skating domain types.
//!
//! This module defines the core skating primitives: edges, elements (jumps, spins,
//! steps, lifts), and related enumerations used throughout Anvomidav.

use serde::{Deserialize, Serialize};
use std::fmt;

// =============================================================================
// Edge Types
// =============================================================================

/// Skating foot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Foot {
    Left,
    Right,
}

impl fmt::Display for Foot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Foot::Left => write!(f, "L"),
            Foot::Right => write!(f, "R"),
        }
    }
}

/// Skating direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Forward,
    Backward,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Forward => write!(f, "F"),
            Direction::Backward => write!(f, "B"),
        }
    }
}

/// Edge curve type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Curve {
    Inside,
    Outside,
    Flat,
}

impl fmt::Display for Curve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Curve::Inside => write!(f, "I"),
            Curve::Outside => write!(f, "O"),
            Curve::Flat => write!(f, ""),
        }
    }
}

/// Complete edge specification.
///
/// An edge is defined by foot (L/R), direction (F/B), and curve (I/O).
/// For example: LFO = Left Forward Outside, RBI = Right Backward Inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Edge {
    LFO, LFI, LBO, LBI,
    RFO, RFI, RBO, RBI,
}

impl Edge {
    /// All possible edges.
    pub const ALL: [Edge; 8] = [
        Edge::LFO, Edge::LFI, Edge::LBO, Edge::LBI,
        Edge::RFO, Edge::RFI, Edge::RBO, Edge::RBI,
    ];

    /// Decompose edge into components.
    pub fn decompose(&self) -> (Foot, Direction, Curve) {
        match self {
            Edge::LFO => (Foot::Left, Direction::Forward, Curve::Outside),
            Edge::LFI => (Foot::Left, Direction::Forward, Curve::Inside),
            Edge::LBO => (Foot::Left, Direction::Backward, Curve::Outside),
            Edge::LBI => (Foot::Left, Direction::Backward, Curve::Inside),
            Edge::RFO => (Foot::Right, Direction::Forward, Curve::Outside),
            Edge::RFI => (Foot::Right, Direction::Forward, Curve::Inside),
            Edge::RBO => (Foot::Right, Direction::Backward, Curve::Outside),
            Edge::RBI => (Foot::Right, Direction::Backward, Curve::Inside),
        }
    }

    /// Compose edge from components.
    pub fn compose(foot: Foot, direction: Direction, curve: Curve) -> Option<Self> {
        match (foot, direction, curve) {
            (Foot::Left, Direction::Forward, Curve::Outside) => Some(Edge::LFO),
            (Foot::Left, Direction::Forward, Curve::Inside) => Some(Edge::LFI),
            (Foot::Left, Direction::Backward, Curve::Outside) => Some(Edge::LBO),
            (Foot::Left, Direction::Backward, Curve::Inside) => Some(Edge::LBI),
            (Foot::Right, Direction::Forward, Curve::Outside) => Some(Edge::RFO),
            (Foot::Right, Direction::Forward, Curve::Inside) => Some(Edge::RFI),
            (Foot::Right, Direction::Backward, Curve::Outside) => Some(Edge::RBO),
            (Foot::Right, Direction::Backward, Curve::Inside) => Some(Edge::RBI),
            _ => None, // Flat edge doesn't have a simple representation
        }
    }

    /// Get the foot.
    pub fn foot(&self) -> Foot {
        self.decompose().0
    }

    /// Get the direction.
    pub fn direction(&self) -> Direction {
        self.decompose().1
    }

    /// Get the curve.
    pub fn curve(&self) -> Curve {
        self.decompose().2
    }

    /// Switch to the other foot (same direction and curve).
    pub fn switch_foot(&self) -> Self {
        let (foot, dir, curve) = self.decompose();
        let new_foot = match foot {
            Foot::Left => Foot::Right,
            Foot::Right => Foot::Left,
        };
        // SAFETY: decompose only returns Inside/Outside curves, never Flat,
        // so compose is guaranteed to return Some for the same direction and curve
        Edge::compose(new_foot, dir, curve)
            .expect("switch_foot: decompose/compose invariant violated")
    }

    /// Parse from string like "LFO", "RBI", etc.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "LFO" => Some(Edge::LFO),
            "LFI" => Some(Edge::LFI),
            "LBO" => Some(Edge::LBO),
            "LBI" => Some(Edge::LBI),
            "RFO" => Some(Edge::RFO),
            "RFI" => Some(Edge::RFI),
            "RBO" => Some(Edge::RBO),
            "RBI" => Some(Edge::RBI),
            _ => None,
        }
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (foot, dir, curve) = self.decompose();
        write!(f, "{}{}{}", foot, dir, curve)
    }
}

// =============================================================================
// Rotation Direction
// =============================================================================

/// Rotation direction for jumps and spins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum RotationDirection {
    /// Counter-clockwise (most common for right-handed skaters)
    #[default]
    CounterClockwise,
    /// Clockwise (most common for left-handed skaters)
    Clockwise,
}

// =============================================================================
// Level (for spins, steps, lifts)
// =============================================================================

/// ISU level for leveled elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub enum Level {
    /// Base level (no features)
    #[default]
    B,
    /// Level 1
    L1,
    /// Level 2
    L2,
    /// Level 3
    L3,
    /// Level 4
    L4,
}

impl Level {
    pub fn as_number(&self) -> u8 {
        match self {
            Level::B => 0,
            Level::L1 => 1,
            Level::L2 => 2,
            Level::L3 => 3,
            Level::L4 => 4,
        }
    }

    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            0 => Some(Level::B),
            1 => Some(Level::L1),
            2 => Some(Level::L2),
            3 => Some(Level::L3),
            4 => Some(Level::L4),
            _ => None,
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::B => write!(f, "B"),
            Level::L1 => write!(f, "1"),
            Level::L2 => write!(f, "2"),
            Level::L3 => write!(f, "3"),
            Level::L4 => write!(f, "4"),
        }
    }
}

// =============================================================================
// Jump Types
// =============================================================================

/// Type of jump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JumpKind {
    /// Axel (A) - forward takeoff, extra half rotation
    Axel,
    /// Lutz (Lz) - back outside edge, toe pick
    Lutz,
    /// Flip (F) - back inside edge, toe pick
    Flip,
    /// Loop (Lo) - back outside edge, no toe pick
    Loop,
    /// Salchow (S) - back inside edge, no toe pick
    Salchow,
    /// Toe Loop (T) - back outside edge, toe pick
    ToeLoop,
    /// Euler/Half Loop (Eu) - connector jump in combinations
    Euler,
}

impl JumpKind {
    /// ISU abbreviation.
    pub fn abbrev(&self) -> &'static str {
        match self {
            JumpKind::Axel => "A",
            JumpKind::Lutz => "Lz",
            JumpKind::Flip => "F",
            JumpKind::Loop => "Lo",
            JumpKind::Salchow => "S",
            JumpKind::ToeLoop => "T",
            JumpKind::Euler => "Eu",
        }
    }

    /// Required entry edge for correct technique.
    pub fn required_entry_edge(&self) -> Edge {
        match self {
            JumpKind::Axel => Edge::LFO,
            JumpKind::Lutz => Edge::LBO,
            JumpKind::Flip => Edge::LBI,
            JumpKind::Loop => Edge::RBO,
            JumpKind::Salchow => Edge::LBI,
            JumpKind::ToeLoop => Edge::RBO,
            JumpKind::Euler => Edge::RBO,
        }
    }

    /// Standard exit edge after landing.
    pub fn standard_exit_edge(&self) -> Edge {
        // All jumps land on RBO for CCW rotation
        Edge::RBO
    }

    /// Is this a toe jump (uses toe pick for takeoff)?
    pub fn is_toe_jump(&self) -> bool {
        matches!(self, JumpKind::Lutz | JumpKind::Flip | JumpKind::ToeLoop)
    }

    /// Parse from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "axel" | "a" => Some(JumpKind::Axel),
            "lutz" | "lz" => Some(JumpKind::Lutz),
            "flip" | "f" => Some(JumpKind::Flip),
            "loop" | "lo" => Some(JumpKind::Loop),
            "salchow" | "s" => Some(JumpKind::Salchow),
            "toe_loop" | "toeloop" | "toe" | "t" => Some(JumpKind::ToeLoop),
            "euler" | "eu" | "half_loop" => Some(JumpKind::Euler),
            _ => None,
        }
    }
}

impl fmt::Display for JumpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbrev())
    }
}

/// Number of rotations for a jump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Rotations {
    Single = 1,
    Double = 2,
    Triple = 3,
    Quad = 4,
}

impl Rotations {
    pub fn as_number(&self) -> u8 {
        *self as u8
    }

    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Rotations::Single),
            2 => Some(Rotations::Double),
            3 => Some(Rotations::Triple),
            4 => Some(Rotations::Quad),
            _ => None,
        }
    }

    /// Prefix for element notation (e.g., "3" for triple)
    pub fn prefix(&self) -> &'static str {
        match self {
            Rotations::Single => "1",
            Rotations::Double => "2",
            Rotations::Triple => "3",
            Rotations::Quad => "4",
        }
    }
}

impl fmt::Display for Rotations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.prefix())
    }
}

// =============================================================================
// Spin Types
// =============================================================================

/// Basic spin position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpinPosition {
    Upright,
    Sit,
    Camel,
    Layback,
    Biellmann,
}

impl SpinPosition {
    pub fn abbrev(&self) -> &'static str {
        match self {
            SpinPosition::Upright => "USp",
            SpinPosition::Sit => "SSp",
            SpinPosition::Camel => "CSp",
            SpinPosition::Layback => "LSp",
            SpinPosition::Biellmann => "BSp",
        }
    }
}

impl fmt::Display for SpinPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpinPosition::Upright => write!(f, "upright"),
            SpinPosition::Sit => write!(f, "sit"),
            SpinPosition::Camel => write!(f, "camel"),
            SpinPosition::Layback => write!(f, "layback"),
            SpinPosition::Biellmann => write!(f, "biellmann"),
        }
    }
}

/// Spin modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpinModifier {
    /// Flying entry
    Flying,
    /// Change of foot
    Change,
    /// Combination spin (multiple positions)
    Combination,
}

// =============================================================================
// Step Types
// =============================================================================

/// Types of turns and steps in step sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StepKind {
    ThreeTurn,
    Bracket,
    Rocker,
    Counter,
    Twizzle,
    Choctaw,
    Mohawk,
    Loop,
    CrossRoll,
    ChangeEdge,
}

impl StepKind {
    /// Is this a difficult turn (for level features)?
    pub fn is_difficult(&self) -> bool {
        matches!(
            self,
            StepKind::Bracket | StepKind::Rocker | StepKind::Counter | StepKind::Twizzle
        )
    }
}

impl fmt::Display for StepKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepKind::ThreeTurn => write!(f, "three_turn"),
            StepKind::Bracket => write!(f, "bracket"),
            StepKind::Rocker => write!(f, "rocker"),
            StepKind::Counter => write!(f, "counter"),
            StepKind::Twizzle => write!(f, "twizzle"),
            StepKind::Choctaw => write!(f, "choctaw"),
            StepKind::Mohawk => write!(f, "mohawk"),
            StepKind::Loop => write!(f, "loop"),
            StepKind::CrossRoll => write!(f, "cross_roll"),
            StepKind::ChangeEdge => write!(f, "change_edge"),
        }
    }
}

// =============================================================================
// Lift Types (Pairs/Dance)
// =============================================================================

/// Lift group for pairs skating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LiftGroup {
    Group1, // Armpit holds
    Group2, // Armpit holds
    Group3, // Waist holds
    Group4, // Waist holds
    Group5, // Hand-to-hand
}

impl fmt::Display for LiftGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiftGroup::Group1 => write!(f, "Gr1"),
            LiftGroup::Group2 => write!(f, "Gr2"),
            LiftGroup::Group3 => write!(f, "Gr3"),
            LiftGroup::Group4 => write!(f, "Gr4"),
            LiftGroup::Group5 => write!(f, "Gr5"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_edge_decompose() {
        let (foot, dir, curve) = Edge::LFO.decompose();
        assert_eq!(foot, Foot::Left);
        assert_eq!(dir, Direction::Forward);
        assert_eq!(curve, Curve::Outside);
    }

    #[test]
    fn test_edge_compose() {
        assert_eq!(
            Edge::compose(Foot::Right, Direction::Backward, Curve::Inside),
            Some(Edge::RBI)
        );
    }

    #[test]
    fn test_edge_switch_foot() {
        assert_eq!(Edge::LFO.switch_foot(), Edge::RFO);
        assert_eq!(Edge::RBI.switch_foot(), Edge::LBI);
    }

    #[test]
    fn test_edge_parse() {
        assert_eq!(Edge::parse("LFO"), Some(Edge::LFO));
        assert_eq!(Edge::parse("rbi"), Some(Edge::RBI));
        assert_eq!(Edge::parse("invalid"), None);
    }

    #[test]
    fn test_jump_kind_entry() {
        assert_eq!(JumpKind::Lutz.required_entry_edge(), Edge::LBO);
        assert_eq!(JumpKind::Flip.required_entry_edge(), Edge::LBI);
    }

    #[test]
    fn test_level_ordering() {
        assert!(Level::B < Level::L1);
        assert!(Level::L1 < Level::L4);
    }
}
