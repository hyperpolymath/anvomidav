// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Type system for Anvomidav.
//!
//! This crate provides type checking and inference for the Anvomidav DSL.
//! It includes:
//!
//! - **Types** (`ty` module): Internal type representation
//! - **Environment** (`env` module): Type environment with scoping
//! - **Checker** (`check` module): Type checking and inference
//!
//! # Example
//!
//! ```ignore
//! use anv_types::check;
//! use anv_syntax::parse;
//! use anv_core::source::FileId;
//!
//! let source = r#"
//!     program my_routine {
//!         segment intro: short {
//!             sequence {
//!                 jump triple axel at 1:30
//!             }
//!         }
//!     }
//! "#;
//!
//! let program = parse(source, FileId(0)).unwrap();
//! let result = check(&program, FileId(0));
//! assert!(result.is_ok());
//! ```

pub mod check;
pub mod env;
pub mod ty;

pub use check::{check, TypeChecker, TypeError, TypeResult};
pub use env::{TypeDef, TypeDefBody, TypeEnv};
pub use ty::{Type, TypeScheme, TypeVar};
