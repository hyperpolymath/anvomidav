// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Intermediate Representation for Anvomidav.
//!
//! The IR provides two levels of program representation:
//!
//! 1. **Timeline** (`timeline` module): A flat, event-based representation
//!    where each scoring element is an [`Event`] with a time, position, and kind.
//!    This is produced by the first lowering pass from the AST.
//!
//! 2. **Choreography** (`choreo` module): A structured representation with
//!    [`PlacedElement`]s decomposed into execution phases, linked by
//!    [`Transition`]s with ice paths. This is produced by the second lowering
//!    pass from the Timeline.
//!
//! # Architecture
//!
//! ```text
//! AST (hierarchical)
//!   Program
//!     Segment                 ──[lower]──>  Timeline (flat)
//!       Sequence                              Event[]
//!         Element                               - position (x, y)
//!                                               - time (start, end)
//!                             ──[lower_to_choreo]──>  Choreography (structured)
//!                                                       ChoreoSegment[]
//!                                                         PlacedElement[]
//!                                                           - phases (entry/exec/exit)
//!                                                           - spatial placement
//!                                                         Transition[]
//!                                                           - ice path (waypoints)
//!                                                           - edge work
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use anv_ir::{lower, lower_to_choreo, Timeline, Choreography};
//! use anv_syntax::parse;
//! use anv_core::source::FileId;
//!
//! let source = r#"program test { segment sp: short { sequence s { jump triple axel } } }"#;
//! let ast = parse(source, FileId(0)).unwrap();
//!
//! // First pass: AST -> Timeline
//! let timeline = lower(&ast);
//!
//! // Second pass: Timeline -> Choreography
//! let choreography = lower_to_choreo(&timeline);
//! ```

pub mod choreo;
pub mod choreo_lower;
pub mod lower;
pub mod path;
pub mod rink;
pub mod timeline;
pub mod types;

// === Timeline IR (pass 1) ===
pub use lower::lower;
pub use rink::{Heading, Position, RinkDimensions};
pub use timeline::{Event, EventKind, Timeline};
pub use types::{Duration, TimeCode};

// === Choreography IR (pass 2) ===
pub use choreo::{
    Choreography, ChoreoSegment, Discipline, ElementId, ElementPhase, ElementPlacement,
    MusicMap, PhaseData, PhaseKind, PlacedElement, ProgramMetadata, SkaterRef, SkaterRole,
    Transition, TransitionId, TransitionQuality,
};
pub use choreo_lower::{lower_to_choreo, lower_to_choreo_with_music};
pub use path::{CurveKind, IcePath, Waypoint};
