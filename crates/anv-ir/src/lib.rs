// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Intermediate Representation for Anvomidav.
//!
//! The IR provides a flat, timeline-based representation of a skating program
//! suitable for visualization and code generation. It transforms the hierarchical
//! AST into a linear sequence of positioned, timed events.
//!
//! # Architecture
//!
//! ```text
//! AST (hierarchical)     →     IR (flat timeline)     →     Output (SVG, etc.)
//!   Program                      Timeline
//!     Segment                      Event[]
//!       Sequence                     - position (x, y)
//!         Element                    - time (start, end)
//!                                    - element data
//! ```
//!
//! # Example
//!
//! ```ignore
//! use anv_ir::{lower, Timeline};
//! use anv_syntax::parse;
//! use anv_core::source::FileId;
//!
//! let source = r#"program test { segment sp: short { sequence s { jump triple axel } } }"#;
//! let ast = parse(source, FileId(0)).unwrap();
//! let timeline = lower(&ast);
//! ```

pub mod lower;
pub mod rink;
pub mod timeline;
pub mod types;

pub use lower::lower;
pub use rink::{Position, RinkDimensions};
pub use timeline::{Event, EventKind, Timeline};
pub use types::{Duration, TimeCode};
