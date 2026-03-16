// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Core types and utilities for Anvomidav.
//!
//! This crate provides the foundational domain types used throughout the
//! Anvomidav compiler and runtime, including skating-specific primitives,
//! source locations, and diagnostics.

#![forbid(unsafe_code)]
pub mod diagnostics;
pub mod skating;
pub mod source;
pub mod types;

pub use diagnostics::*;
pub use skating::*;
pub use source::*;
pub use types::*;
