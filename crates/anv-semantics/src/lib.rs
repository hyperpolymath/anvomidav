// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Semantic analysis and ISU rules validation for Anvomidav.
//!
//! This module implements ISU (International Skating Union) rules validation
//! for figure skating programs, including element counts, required elements,
//! and various constraints for different disciplines.
//!
//! # Example
//!
//! ```ignore
//! use anv_semantics::{validate_program, Discipline};
//! use anv_syntax::parse;
//! use anv_core::source::FileId;
//!
//! let source = r#"
//!     program my_program {
//!         segment short: short {
//!             sequence { jump triple axel }
//!         }
//!     }
//! "#;
//! let program = parse(source, FileId(0)).unwrap();
//! let errors = validate_program(&program, Discipline::MenSingles);
//! ```

#![forbid(unsafe_code)]
pub mod rules;
pub mod validate;

pub use rules::{Discipline, ISURules, SegmentRules};
pub use validate::{validate_program, SemanticError, ValidationResult};
