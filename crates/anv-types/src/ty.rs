// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Type definitions for Anvomidav.
//!
//! This module defines the internal representation of types used during
//! type checking. Types are more detailed than AST type expressions and
//! include resolved references and type variables.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

/// Unique identifier for type variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub u32);

impl TypeVar {
    /// Generate a fresh type variable.
    pub fn fresh() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        TypeVar(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl fmt::Display for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use Greek letters for type variables
        let letters = ['α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ'];
        if (self.0 as usize) < letters.len() {
            write!(f, "{}", letters[self.0 as usize])
        } else {
            write!(f, "τ{}", self.0)
        }
    }
}

/// A type in the Anvomidav type system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // === Primitive types ===
    /// Unit type (no value).
    Unit,
    /// Boolean type.
    Bool,
    /// Integer type.
    Int,
    /// Floating-point type.
    Float,
    /// String type.
    String,

    // === Domain-specific types ===
    /// Time type (seconds).
    Time,
    /// Duration type (seconds).
    Duration,
    /// Position type (2D coordinates in meters).
    Position,
    /// Position3 type (3D coordinates in meters).
    Position3,
    /// Angle type (radians).
    Angle,
    /// Velocity type (m/s).
    Velocity,
    /// Beat type (musical beat reference).
    Beat,
    /// Edge type.
    Edge,
    /// Level type.
    Level,

    // === Skating element types ===
    /// Jump element type.
    Jump,
    /// Spin element type.
    Spin,
    /// Step sequence type.
    StepSequence,
    /// Lift element type (pairs).
    Lift,
    /// Throw element type (pairs).
    Throw,
    /// Twist element type (pairs).
    Twist,
    /// Death spiral type (pairs).
    DeathSpiral,
    /// Transition element type.
    Transition,
    /// Choreographic element type.
    Choreographic,
    /// Generic element type (supertype of all element types).
    Element,
    /// Sequence type.
    Sequence,
    /// Segment type.
    Segment,
    /// Program type.
    Program,

    // === Compound types ===
    /// Function type: (param_types) -> return_type.
    Fn(Vec<Type>, Box<Type>),
    /// Tuple type: (T1, T2, ...).
    Tuple(Vec<Type>),
    /// Array type: [T].
    Array(Box<Type>),
    /// Optional type: T?.
    Optional(Box<Type>),
    /// Record type: { field: Type, ... }.
    Record(Vec<(String, Type)>),
    /// Enum/variant type.
    Variant(Vec<(String, Option<Type>)>),

    // === Type constructors ===
    /// Named type (user-defined or imported).
    Named(String),
    /// Generic type application: T<A, B, ...>.
    App(Box<Type>, Vec<Type>),

    // === Type variables for inference ===
    /// Unification variable (for type inference).
    Var(TypeVar),

    // === Refinement types ===
    /// Refinement type: { x: T | predicate }.
    Refinement {
        base: Box<Type>,
        predicate: String, // For now, store as string; will be SMT expression
    },

    // === Error type ===
    /// Error type (used during type checking when errors occur).
    Error,
}

impl Type {
    /// Returns true if this is a primitive type.
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Type::Unit | Type::Bool | Type::Int | Type::Float | Type::String
        )
    }

    /// Returns true if this is a domain-specific type.
    pub fn is_domain(&self) -> bool {
        matches!(
            self,
            Type::Time
                | Type::Duration
                | Type::Position
                | Type::Position3
                | Type::Angle
                | Type::Velocity
                | Type::Beat
                | Type::Edge
                | Type::Level
        )
    }

    /// Returns true if this is a skating element type.
    pub fn is_element(&self) -> bool {
        matches!(
            self,
            Type::Jump
                | Type::Spin
                | Type::StepSequence
                | Type::Lift
                | Type::Throw
                | Type::Twist
                | Type::DeathSpiral
                | Type::Transition
                | Type::Choreographic
                | Type::Element
        )
    }

    /// Returns true if this is a function type.
    pub fn is_function(&self) -> bool {
        matches!(self, Type::Fn(_, _))
    }

    /// Returns true if this type contains unresolved type variables.
    pub fn has_vars(&self) -> bool {
        match self {
            Type::Var(_) => true,
            Type::Fn(params, ret) => params.iter().any(|t| t.has_vars()) || ret.has_vars(),
            Type::Tuple(ts) => ts.iter().any(|t| t.has_vars()),
            Type::Array(t) | Type::Optional(t) => t.has_vars(),
            Type::Record(fields) => fields.iter().any(|(_, t)| t.has_vars()),
            Type::Variant(variants) => variants.iter().any(|(_, t)| t.as_ref().is_some_and(|t| t.has_vars())),
            Type::App(base, args) => base.has_vars() || args.iter().any(|t| t.has_vars()),
            Type::Refinement { base, .. } => base.has_vars(),
            _ => false,
        }
    }

    /// Substitute type variables according to the given mapping.
    pub fn substitute(&self, subst: &HashMap<TypeVar, Type>) -> Type {
        match self {
            Type::Var(v) => subst.get(v).cloned().unwrap_or_else(|| self.clone()),
            Type::Fn(params, ret) => Type::Fn(
                params.iter().map(|t| t.substitute(subst)).collect(),
                Box::new(ret.substitute(subst)),
            ),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| t.substitute(subst)).collect()),
            Type::Array(t) => Type::Array(Box::new(t.substitute(subst))),
            Type::Optional(t) => Type::Optional(Box::new(t.substitute(subst))),
            Type::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(n, t)| (n.clone(), t.substitute(subst)))
                    .collect(),
            ),
            Type::Variant(variants) => Type::Variant(
                variants
                    .iter()
                    .map(|(n, t)| (n.clone(), t.as_ref().map(|t| t.substitute(subst))))
                    .collect(),
            ),
            Type::App(base, args) => Type::App(
                Box::new(base.substitute(subst)),
                args.iter().map(|t| t.substitute(subst)).collect(),
            ),
            Type::Refinement { base, predicate } => Type::Refinement {
                base: Box::new(base.substitute(subst)),
                predicate: predicate.clone(),
            },
            _ => self.clone(),
        }
    }

    /// Collect all type variables in this type.
    pub fn free_vars(&self) -> Vec<TypeVar> {
        let mut vars = Vec::new();
        self.collect_vars(&mut vars);
        vars
    }

    fn collect_vars(&self, vars: &mut Vec<TypeVar>) {
        match self {
            Type::Var(v) => {
                if !vars.contains(v) {
                    vars.push(*v);
                }
            }
            Type::Fn(params, ret) => {
                for p in params {
                    p.collect_vars(vars);
                }
                ret.collect_vars(vars);
            }
            Type::Tuple(ts) => {
                for t in ts {
                    t.collect_vars(vars);
                }
            }
            Type::Array(t) | Type::Optional(t) => t.collect_vars(vars),
            Type::Record(fields) => {
                for (_, t) in fields {
                    t.collect_vars(vars);
                }
            }
            Type::Variant(variants) => {
                for (_, t) in variants {
                    if let Some(t) = t {
                        t.collect_vars(vars);
                    }
                }
            }
            Type::App(base, args) => {
                base.collect_vars(vars);
                for a in args {
                    a.collect_vars(vars);
                }
            }
            Type::Refinement { base, .. } => base.collect_vars(vars),
            _ => {}
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::Bool => write!(f, "Bool"),
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::String => write!(f, "String"),
            Type::Time => write!(f, "Time"),
            Type::Duration => write!(f, "Duration"),
            Type::Position => write!(f, "Position"),
            Type::Position3 => write!(f, "Position3"),
            Type::Angle => write!(f, "Angle"),
            Type::Velocity => write!(f, "Velocity"),
            Type::Beat => write!(f, "Beat"),
            Type::Edge => write!(f, "Edge"),
            Type::Level => write!(f, "Level"),
            Type::Jump => write!(f, "Jump"),
            Type::Spin => write!(f, "Spin"),
            Type::StepSequence => write!(f, "StepSequence"),
            Type::Lift => write!(f, "Lift"),
            Type::Throw => write!(f, "Throw"),
            Type::Twist => write!(f, "Twist"),
            Type::DeathSpiral => write!(f, "DeathSpiral"),
            Type::Transition => write!(f, "Transition"),
            Type::Choreographic => write!(f, "Choreographic"),
            Type::Element => write!(f, "Element"),
            Type::Sequence => write!(f, "Sequence"),
            Type::Segment => write!(f, "Segment"),
            Type::Program => write!(f, "Program"),
            Type::Fn(params, ret) => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Tuple(ts) => {
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Array(t) => write!(f, "[{}]", t),
            Type::Optional(t) => write!(f, "{}?", t),
            Type::Record(fields) => {
                write!(f, "{{ ")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, ty)?;
                }
                write!(f, " }}")
            }
            Type::Variant(variants) => {
                for (i, (name, ty)) in variants.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", name)?;
                    if let Some(ty) = ty {
                        write!(f, "({})", ty)?;
                    }
                }
                Ok(())
            }
            Type::Named(name) => write!(f, "{}", name),
            Type::App(base, args) => {
                write!(f, "{}<", base)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ">")
            }
            Type::Var(v) => write!(f, "{}", v),
            Type::Refinement { base, predicate } => {
                write!(f, "{{ x: {} | {} }}", base, predicate)
            }
            Type::Error => write!(f, "<error>"),
        }
    }
}

/// Type scheme (for polymorphic types).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScheme {
    /// Quantified type variables.
    pub vars: Vec<TypeVar>,
    /// The body type.
    pub ty: Type,
}

impl TypeScheme {
    /// Create a monomorphic type scheme (no quantified variables).
    pub fn mono(ty: Type) -> Self {
        TypeScheme { vars: vec![], ty }
    }

    /// Create a polymorphic type scheme.
    pub fn poly(vars: Vec<TypeVar>, ty: Type) -> Self {
        TypeScheme { vars, ty }
    }

    /// Instantiate the type scheme with fresh type variables.
    pub fn instantiate(&self) -> Type {
        let subst: HashMap<TypeVar, Type> = self
            .vars
            .iter()
            .map(|v| (*v, Type::Var(TypeVar::fresh())))
            .collect();
        self.ty.substitute(&subst)
    }
}

impl fmt::Display for TypeScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.is_empty() {
            write!(f, "{}", self.ty)
        } else {
            write!(f, "∀")?;
            for (i, v) in self.vars.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", v)?;
            }
            write!(f, ". {}", self.ty)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::Int), "Int");
        assert_eq!(format!("{}", Type::Time), "Time");
        assert_eq!(format!("{}", Type::Jump), "Jump");
        assert_eq!(
            format!("{}", Type::Fn(vec![Type::Int, Type::Int], Box::new(Type::Bool))),
            "(Int, Int) -> Bool"
        );
        assert_eq!(
            format!("{}", Type::Array(Box::new(Type::Element))),
            "[Element]"
        );
    }

    #[test]
    fn test_type_var_fresh() {
        let v1 = TypeVar::fresh();
        let v2 = TypeVar::fresh();
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_type_scheme_instantiate() {
        let alpha = TypeVar::fresh();
        let scheme = TypeScheme::poly(
            vec![alpha],
            Type::Fn(vec![Type::Var(alpha)], Box::new(Type::Var(alpha))),
        );

        let t1 = scheme.instantiate();
        let t2 = scheme.instantiate();

        // Each instantiation should produce different type variables
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_type_substitute() {
        let alpha = TypeVar::fresh();
        let ty = Type::Fn(vec![Type::Var(alpha)], Box::new(Type::Int));

        let mut subst = HashMap::new();
        subst.insert(alpha, Type::String);

        let result = ty.substitute(&subst);
        assert_eq!(result, Type::Fn(vec![Type::String], Box::new(Type::Int)));
    }

    #[test]
    fn test_free_vars() {
        let alpha = TypeVar::fresh();
        let beta = TypeVar::fresh();
        let ty = Type::Fn(vec![Type::Var(alpha)], Box::new(Type::Var(beta)));

        let vars = ty.free_vars();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&alpha));
        assert!(vars.contains(&beta));
    }
}
