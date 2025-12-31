// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Visualization output for Anvomidav.
//!
//! This crate generates visual representations of skating programs:
//! - SVG rink diagrams with element paths
//! - Timeline charts
//! - Element position markers
//!
//! # Example
//!
//! ```ignore
//! use anv_viz::{RinkRenderer, SvgOptions};
//! use anv_ir::Timeline;
//!
//! let timeline: Timeline = // ... from lowering
//! let svg = RinkRenderer::new(SvgOptions::default())
//!     .render(&timeline);
//! ```

pub mod rink_svg;
pub mod timeline_svg;

pub use rink_svg::{RinkRenderer, SvgOptions};
pub use timeline_svg::TimelineRenderer;
