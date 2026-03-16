// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Type environment for Anvomidav.
//!
//! This module defines the type environment used during type checking,
//! which maps variables to their types and tracks type definitions.

use crate::ty::{Type, TypeScheme, TypeVar};
use std::collections::HashMap;

/// The type environment.
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// Variable bindings: name -> type scheme.
    bindings: Vec<HashMap<String, TypeScheme>>,
    /// Type definitions: name -> type.
    types: HashMap<String, TypeDef>,
}

/// A type definition.
#[derive(Debug, Clone)]
pub struct TypeDef {
    /// Type parameters.
    pub params: Vec<String>,
    /// The definition body.
    pub body: TypeDefBody,
}

/// Body of a type definition.
#[derive(Debug, Clone)]
pub enum TypeDefBody {
    /// Type alias.
    Alias(Type),
    /// Record type.
    Record(Vec<(String, Type)>),
    /// Enum/variant type.
    Enum(Vec<(String, Option<Type>)>),
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeEnv {
    /// Create a new empty environment.
    pub fn new() -> Self {
        let mut env = TypeEnv {
            bindings: vec![HashMap::new()],
            types: HashMap::new(),
        };
        env.init_builtins();
        env
    }

    /// Initialize built-in types and functions.
    fn init_builtins(&mut self) {
        // Built-in skating element constructors
        self.define(
            "triple_axel",
            TypeScheme::mono(Type::Jump),
        );
        self.define(
            "triple_lutz",
            TypeScheme::mono(Type::Jump),
        );
        self.define(
            "triple_flip",
            TypeScheme::mono(Type::Jump),
        );
        self.define(
            "triple_loop",
            TypeScheme::mono(Type::Jump),
        );
        self.define(
            "triple_salchow",
            TypeScheme::mono(Type::Jump),
        );
        self.define(
            "triple_toe_loop",
            TypeScheme::mono(Type::Jump),
        );
        self.define(
            "quad_axel",
            TypeScheme::mono(Type::Jump),
        );
        self.define(
            "quad_lutz",
            TypeScheme::mono(Type::Jump),
        );

        // Time/duration functions
        self.define(
            "seconds",
            TypeScheme::mono(Type::Fn(vec![Type::Float], Box::new(Type::Duration))),
        );
        self.define(
            "minutes",
            TypeScheme::mono(Type::Fn(vec![Type::Float], Box::new(Type::Duration))),
        );

        // Position functions
        self.define(
            "center",
            TypeScheme::mono(Type::Position),
        );
        self.define(
            "corner",
            TypeScheme::mono(Type::Fn(vec![Type::Int], Box::new(Type::Position))),
        );
        self.define(
            "offset",
            TypeScheme::mono(Type::Fn(
                vec![Type::Position, Type::Float, Type::Float],
                Box::new(Type::Position),
            )),
        );

        // Arithmetic operators (as polymorphic functions)
        let num = TypeVar::fresh();
        self.define(
            "+",
            TypeScheme::poly(
                vec![num],
                Type::Fn(vec![Type::Var(num), Type::Var(num)], Box::new(Type::Var(num))),
            ),
        );
        self.define(
            "-",
            TypeScheme::poly(
                vec![num],
                Type::Fn(vec![Type::Var(num), Type::Var(num)], Box::new(Type::Var(num))),
            ),
        );
        self.define(
            "*",
            TypeScheme::poly(
                vec![num],
                Type::Fn(vec![Type::Var(num), Type::Var(num)], Box::new(Type::Var(num))),
            ),
        );
        self.define(
            "/",
            TypeScheme::poly(
                vec![num],
                Type::Fn(vec![Type::Var(num), Type::Var(num)], Box::new(Type::Var(num))),
            ),
        );

        // Comparison operators
        let ord = TypeVar::fresh();
        self.define(
            "<",
            TypeScheme::poly(
                vec![ord],
                Type::Fn(vec![Type::Var(ord), Type::Var(ord)], Box::new(Type::Bool)),
            ),
        );
        self.define(
            "<=",
            TypeScheme::poly(
                vec![ord],
                Type::Fn(vec![Type::Var(ord), Type::Var(ord)], Box::new(Type::Bool)),
            ),
        );
        self.define(
            ">",
            TypeScheme::poly(
                vec![ord],
                Type::Fn(vec![Type::Var(ord), Type::Var(ord)], Box::new(Type::Bool)),
            ),
        );
        self.define(
            ">=",
            TypeScheme::poly(
                vec![ord],
                Type::Fn(vec![Type::Var(ord), Type::Var(ord)], Box::new(Type::Bool)),
            ),
        );

        // Equality operators
        let eq = TypeVar::fresh();
        self.define(
            "==",
            TypeScheme::poly(
                vec![eq],
                Type::Fn(vec![Type::Var(eq), Type::Var(eq)], Box::new(Type::Bool)),
            ),
        );
        self.define(
            "!=",
            TypeScheme::poly(
                vec![eq],
                Type::Fn(vec![Type::Var(eq), Type::Var(eq)], Box::new(Type::Bool)),
            ),
        );

        // Logical operators
        self.define(
            "&&",
            TypeScheme::mono(Type::Fn(vec![Type::Bool, Type::Bool], Box::new(Type::Bool))),
        );
        self.define(
            "||",
            TypeScheme::mono(Type::Fn(vec![Type::Bool, Type::Bool], Box::new(Type::Bool))),
        );
        self.define(
            "!",
            TypeScheme::mono(Type::Fn(vec![Type::Bool], Box::new(Type::Bool))),
        );
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.bindings.push(HashMap::new());
    }

    /// Pop the current scope.
    pub fn pop_scope(&mut self) {
        if self.bindings.len() > 1 {
            self.bindings.pop();
        }
    }

    /// Define a variable in the current scope.
    pub fn define(&mut self, name: impl Into<String>, scheme: TypeScheme) {
        if let Some(scope) = self.bindings.last_mut() {
            scope.insert(name.into(), scheme);
        }
    }

    /// Look up a variable.
    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        for scope in self.bindings.iter().rev() {
            if let Some(scheme) = scope.get(name) {
                return Some(scheme);
            }
        }
        None
    }

    /// Define a type.
    pub fn define_type(&mut self, name: impl Into<String>, def: TypeDef) {
        self.types.insert(name.into(), def);
    }

    /// Look up a type definition.
    pub fn lookup_type(&self, name: &str) -> Option<&TypeDef> {
        self.types.get(name)
    }

    /// Get all variables in scope.
    pub fn all_bindings(&self) -> impl Iterator<Item = (&String, &TypeScheme)> {
        self.bindings.iter().flat_map(|scope| scope.iter())
    }

    /// Generalize a type to a type scheme.
    ///
    /// Type variables that are free in the environment are not generalized.
    pub fn generalize(&self, ty: &Type) -> TypeScheme {
        let free_in_env: Vec<TypeVar> = self
            .bindings
            .iter()
            .flat_map(|scope| scope.values())
            .flat_map(|scheme| scheme.ty.free_vars())
            .collect();

        let free_in_ty = ty.free_vars();
        let quantified: Vec<TypeVar> = free_in_ty
            .into_iter()
            .filter(|v| !free_in_env.contains(v))
            .collect();

        TypeScheme::poly(quantified, ty.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_lookup() {
        let mut env = TypeEnv::new();
        env.define("x", TypeScheme::mono(Type::Int));

        assert!(env.lookup("x").is_some());
        assert_eq!(env.lookup("x").unwrap().ty, Type::Int);
        assert!(env.lookup("y").is_none());
    }

    #[test]
    fn test_env_scoping() {
        let mut env = TypeEnv::new();
        env.define("x", TypeScheme::mono(Type::Int));

        env.push_scope();
        env.define("x", TypeScheme::mono(Type::String));

        // Inner scope shadows outer
        assert_eq!(env.lookup("x").unwrap().ty, Type::String);

        env.pop_scope();

        // Back to outer scope
        assert_eq!(env.lookup("x").unwrap().ty, Type::Int);
    }

    #[test]
    fn test_env_builtins() {
        let env = TypeEnv::new();

        // Check that built-in operators exist
        assert!(env.lookup("+").is_some());
        assert!(env.lookup("<").is_some());
        assert!(env.lookup("&&").is_some());

        // Check that built-in skating constructors exist
        assert!(env.lookup("triple_axel").is_some());
        assert_eq!(env.lookup("triple_axel").unwrap().ty, Type::Jump);
    }

    #[test]
    fn test_generalize() {
        let env = TypeEnv::new();
        let alpha = TypeVar::fresh();
        let ty = Type::Fn(vec![Type::Var(alpha)], Box::new(Type::Var(alpha)));

        let scheme = env.generalize(&ty);
        assert_eq!(scheme.vars.len(), 1);
        assert!(scheme.vars.contains(&alpha));
    }
}
