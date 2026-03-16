// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Syntax processing for Anvomidav.
//!
//! This crate provides lexical analysis and parsing for the Anvomidav DSL.
//! It includes:
//!
//! - **Lexer** (`token` module): Tokenizes source code using `logos`
//! - **AST** (`ast` module): Abstract syntax tree definitions
//! - **Parser** (`parser` module): Parses tokens into AST using `chumsky`
//!
//! # Example
//!
//! ```
//! use anv_syntax::{parse, ast::Program};
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
//! let result = parse(source, FileId(0));
//! assert!(result.is_ok());
//! ```

#![forbid(unsafe_code)]
pub mod ast;
pub mod format;
pub mod parser;
pub mod token;

pub use ast::*;
pub use parser::{parse, parse_tokens, ParseError};
pub use token::{Lexer, LexError, SpannedToken, Token};
