// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Type checker for Anvomidav.
//!
//! This module implements type checking and inference for Anvomidav programs.
//! It uses a Hindley-Milner style type inference with extensions for
//! domain-specific types and refinement types.

use crate::env::TypeEnv;
use crate::ty::{Type, TypeScheme, TypeVar};
use anv_core::diagnostics::{Diagnostic, Diagnostics, ErrorCode, Label, Severity};
use anv_core::source::{FileId, Span};
use anv_syntax::ast::*;
use std::collections::HashMap;

/// Result of type checking.
pub type TypeResult<T> = Result<T, TypeError>;

/// A type error.
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: Span,
    pub expected: Option<Type>,
    pub found: Option<Type>,
}

impl TypeError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        TypeError {
            message: message.into(),
            span,
            expected: None,
            found: None,
        }
    }

    pub fn mismatch(expected: Type, found: Type, span: Span) -> Self {
        TypeError {
            message: format!("type mismatch: expected {}, found {}", expected, found),
            span,
            expected: Some(expected),
            found: Some(found),
        }
    }

    pub fn undefined(name: &str, span: Span) -> Self {
        TypeError {
            message: format!("undefined variable: {}", name),
            span,
            expected: None,
            found: None,
        }
    }

    pub fn to_diagnostic(&self) -> Diagnostic {
        let code = if self.expected.is_some() && self.found.is_some() {
            ErrorCode::TYPE_MISMATCH
        } else {
            ErrorCode::UNDEFINED_VARIABLE
        };

        let mut diag = Diagnostic::error(&self.message).with_code(code);
        diag = diag.with_label(self.span, "here");

        if let (Some(expected), Some(found)) = (&self.expected, &self.found) {
            diag = diag.with_note(format!("expected: {}", expected));
            diag = diag.with_note(format!("   found: {}", found));
        }

        diag
    }
}

/// The type checker.
pub struct TypeChecker {
    /// Type environment.
    env: TypeEnv,
    /// Substitution (for unification).
    subst: HashMap<TypeVar, Type>,
    /// Collected errors.
    errors: Vec<TypeError>,
    /// Current file being checked.
    file_id: FileId,
}

impl TypeChecker {
    /// Create a new type checker.
    pub fn new(file_id: FileId) -> Self {
        TypeChecker {
            env: TypeEnv::new(),
            subst: HashMap::new(),
            errors: Vec::new(),
            file_id,
        }
    }

    /// Check a program.
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        // Check all function definitions
        for func in &program.functions {
            if let Err(e) = self.check_fn_def(func) {
                self.errors.push(e);
            }
        }

        // Check all segments
        for segment in &program.segments {
            if let Err(e) = self.check_segment(segment) {
                self.errors.push(e);
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Check a function definition.
    fn check_fn_def(&mut self, func: &FnDef) -> TypeResult<Type> {
        self.env.push_scope();

        // Add parameters to environment
        let mut param_types = Vec::new();
        for param in &func.params {
            let ty = if let Some(ref ty_expr) = param.ty {
                self.resolve_type_expr(ty_expr)?
            } else {
                Type::Var(TypeVar::fresh())
            };
            self.env.define(&param.name.node, TypeScheme::mono(ty.clone()));
            param_types.push(ty);
        }

        // Check body
        let body_ty = self.infer_expr(&func.body)?;

        // Check return type if specified
        if let Some(ref ret_ty_expr) = func.return_ty {
            let ret_ty = self.resolve_type_expr(ret_ty_expr)?;
            self.unify(&ret_ty, &body_ty, func.span)?;
        }

        self.env.pop_scope();

        // Create function type
        let fn_ty = Type::Fn(param_types, Box::new(body_ty));

        // Add to environment with generalization
        let scheme = self.env.generalize(&fn_ty);
        self.env.define(&func.name.node, scheme);

        Ok(fn_ty)
    }

    /// Check a segment.
    fn check_segment(&mut self, segment: &Segment) -> TypeResult<Type> {
        self.env.push_scope();

        // Check all sequences
        for sequence in &segment.sequences {
            self.check_sequence(sequence)?;
        }

        self.env.pop_scope();

        Ok(Type::Segment)
    }

    /// Check a sequence.
    fn check_sequence(&mut self, sequence: &Sequence) -> TypeResult<Type> {
        for element in &sequence.elements {
            self.check_element(element)?;
        }
        Ok(Type::Sequence)
    }

    /// Check an element.
    fn check_element(&mut self, element: &Element) -> TypeResult<Type> {
        let ty = match &element.kind {
            ElementKind::Jump(jump) => self.check_jump(jump, element.span)?,
            ElementKind::Spin(spin) => self.check_spin(spin, element.span)?,
            ElementKind::StepSequence(steps) => self.check_step_sequence(steps, element.span)?,
            ElementKind::Lift(lift) => self.check_lift(lift, element.span)?,
            ElementKind::DeathSpiral(_) => Type::DeathSpiral,
            ElementKind::Throw(throw) => self.check_throw(throw, element.span)?,
            ElementKind::Twist(_) => Type::Twist,
            ElementKind::Transition(_) => Type::Transition,
            ElementKind::Choreographic(_) => Type::Choreographic,
            ElementKind::Pattern(_) => Type::Element,
            ElementKind::Parallel(_) => Type::Element,
            ElementKind::Sync(_) => Type::Element,
        };

        // Check timing if present
        if let Some(ref timing) = element.timing {
            self.check_timing(timing)?;
        }

        // Check position if present
        if let Some(ref position) = element.position {
            self.check_position_expr(position)?;
        }

        Ok(ty)
    }

    /// Check a jump element.
    fn check_jump(&mut self, jump: &JumpElement, span: Span) -> TypeResult<Type> {
        // Validate entry edge matches jump requirements
        if let Some(ref entry_edge) = jump.entry_edge {
            let required = jump.kind.required_entry_edge();
            if *entry_edge != required {
                self.errors.push(TypeError {
                    message: format!(
                        "invalid entry edge for {:?}: expected {:?}, found {:?}",
                        jump.kind, required, entry_edge
                    ),
                    span,
                    expected: None,
                    found: None,
                });
            }
        }

        // Check combinations recursively
        for combo_jump in &jump.combination {
            self.check_jump(combo_jump, span)?;
        }

        Ok(Type::Jump)
    }

    /// Check a spin element.
    fn check_spin(&mut self, spin: &SpinElement, span: Span) -> TypeResult<Type> {
        // Validate minimum revolutions for level requirements
        if let Some(level) = &spin.level {
            if let Some(min_revs) = spin.min_revs {
                let required_revs = match level {
                    anv_core::skating::Level::L4 => 8,
                    anv_core::skating::Level::L3 => 6,
                    anv_core::skating::Level::L2 => 4,
                    _ => 2,
                };
                if min_revs < required_revs {
                    self.errors.push(TypeError {
                        message: format!(
                            "insufficient revolutions for level {:?}: requires {}, specified {}",
                            level, required_revs, min_revs
                        ),
                        span,
                        expected: None,
                        found: None,
                    });
                }
            }
        }

        Ok(Type::Spin)
    }

    /// Check a step sequence.
    fn check_step_sequence(&mut self, _steps: &StepSequence, _span: Span) -> TypeResult<Type> {
        // TODO: Validate step sequence requirements
        Ok(Type::StepSequence)
    }

    /// Check a lift element.
    fn check_lift(&mut self, _lift: &LiftElement, _span: Span) -> TypeResult<Type> {
        // TODO: Validate lift requirements
        Ok(Type::Lift)
    }

    /// Check a throw element.
    fn check_throw(&mut self, _throw: &ThrowElement, _span: Span) -> TypeResult<Type> {
        // TODO: Validate throw requirements
        Ok(Type::Throw)
    }

    /// Check timing.
    fn check_timing(&mut self, timing: &Timing) -> TypeResult<Type> {
        match timing {
            Timing::At(time) => self.check_time_expr(time),
            Timing::Range(start, end) => {
                self.check_time_expr(start)?;
                self.check_time_expr(end)
            }
            Timing::Duration(dur) => self.check_time_expr(dur),
            Timing::Beat(_) => Ok(Type::Beat),
            Timing::After(time) => self.check_time_expr(time),
        }
    }

    /// Check a time expression.
    fn check_time_expr(&mut self, time: &TimeExpr) -> TypeResult<Type> {
        match time {
            TimeExpr::Literal(_) => Ok(Type::Time),
            TimeExpr::Formatted(_) => Ok(Type::Time),
            TimeExpr::Var(ident) => {
                if let Some(scheme) = self.env.lookup(&ident.node) {
                    let ty = scheme.instantiate();
                    self.unify(&ty, &Type::Time, ident.span)?;
                    Ok(Type::Time)
                } else {
                    Err(TypeError::undefined(&ident.node, ident.span))
                }
            }
            TimeExpr::BinOp(left, _op, right) => {
                self.check_time_expr(left)?;
                self.check_time_expr(right)?;
                Ok(Type::Time)
            }
        }
    }

    /// Check a position expression.
    fn check_position_expr(&mut self, pos: &PositionExpr) -> TypeResult<Type> {
        match pos {
            PositionExpr::Literal(_, _) => Ok(Type::Position),
            PositionExpr::Named(_) => Ok(Type::Position),
            PositionExpr::Var(ident) => {
                if let Some(scheme) = self.env.lookup(&ident.node) {
                    let ty = scheme.instantiate();
                    self.unify(&ty, &Type::Position, ident.span)?;
                    Ok(Type::Position)
                } else {
                    Err(TypeError::undefined(&ident.node, ident.span))
                }
            }
            PositionExpr::Relative(base, _, _) => {
                self.check_position_expr(base)?;
                Ok(Type::Position)
            }
        }
    }

    /// Infer the type of an expression.
    fn infer_expr(&mut self, expr: &Expr) -> TypeResult<Type> {
        match expr {
            Expr::Int(_) => Ok(Type::Int),
            Expr::Float(_) => Ok(Type::Float),
            Expr::String(_) => Ok(Type::String),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::Time(time) => self.check_time_expr(time),
            Expr::Position(pos) => self.check_position_expr(pos),
            Expr::Var(ident) => {
                if let Some(scheme) = self.env.lookup(&ident.node) {
                    Ok(scheme.instantiate())
                } else {
                    Err(TypeError::undefined(&ident.node, ident.span))
                }
            }
            Expr::BinOp(left, op, right) => {
                let left_ty = self.infer_spanned_expr(left)?;
                let right_ty = self.infer_spanned_expr(right)?;
                self.check_binop(*op, &left_ty, &right_ty, left.span)
            }
            Expr::UnaryOp(op, operand) => {
                let operand_ty = self.infer_spanned_expr(operand)?;
                self.check_unaryop(*op, &operand_ty, operand.span)
            }
            Expr::Call(func, args) => {
                let func_ty = self.infer_spanned_expr(func)?;
                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.infer_spanned_expr(arg)?);
                }
                self.check_call(&func_ty, &arg_types, func.span)
            }
            Expr::Field(base, field) => {
                let base_ty = self.infer_spanned_expr(base)?;
                self.check_field_access(&base_ty, &field.node, field.span)
            }
            Expr::Index(base, index) => {
                let base_ty = self.infer_spanned_expr(base)?;
                let _index_ty = self.infer_spanned_expr(index)?;
                self.check_index(&base_ty, base.span)
            }
            Expr::If(cond, then_branch, else_branch) => {
                let cond_ty = self.infer_spanned_expr(cond)?;
                self.unify(&cond_ty, &Type::Bool, cond.span)?;

                let then_ty = self.infer_spanned_expr(then_branch)?;

                if let Some(else_branch) = else_branch {
                    let else_ty = self.infer_spanned_expr(else_branch)?;
                    self.unify(&then_ty, &else_ty, else_branch.span)?;
                    Ok(then_ty)
                } else {
                    Ok(Type::Unit)
                }
            }
            Expr::Let(name, ty_ann, value, body) => {
                let value_ty = self.infer_spanned_expr(value)?;

                if let Some(ty_expr) = ty_ann {
                    let ann_ty = self.resolve_type_expr(ty_expr)?;
                    self.unify(&value_ty, &ann_ty, value.span)?;
                }

                self.env.push_scope();
                let scheme = self.env.generalize(&value_ty);
                self.env.define(&name.node, scheme);

                let body_ty = self.infer_spanned_expr(body)?;
                self.env.pop_scope();

                Ok(body_ty)
            }
            Expr::Block(stmts) => {
                self.env.push_scope();
                let mut last_ty = Type::Unit;
                for stmt in stmts {
                    last_ty = self.check_stmt(&stmt.node)?;
                }
                self.env.pop_scope();
                Ok(last_ty)
            }
            Expr::Lambda(params, body) => {
                self.env.push_scope();
                let mut param_types = Vec::new();
                for param in params {
                    let ty = if let Some(ref ty_expr) = param.ty {
                        self.resolve_type_expr(ty_expr)?
                    } else {
                        Type::Var(TypeVar::fresh())
                    };
                    self.env.define(&param.name.node, TypeScheme::mono(ty.clone()));
                    param_types.push(ty);
                }
                let body_ty = self.infer_spanned_expr(body)?;
                self.env.pop_scope();
                Ok(Type::Fn(param_types, Box::new(body_ty)))
            }
            Expr::Tuple(exprs) => {
                let types: Vec<Type> = exprs
                    .iter()
                    .map(|e| self.infer_spanned_expr(e))
                    .collect::<Result<_, _>>()?;
                Ok(Type::Tuple(types))
            }
            Expr::Array(exprs) => {
                if exprs.is_empty() {
                    Ok(Type::Array(Box::new(Type::Var(TypeVar::fresh()))))
                } else {
                    let first_ty = self.infer_spanned_expr(&exprs[0])?;
                    for expr in &exprs[1..] {
                        let ty = self.infer_spanned_expr(expr)?;
                        self.unify(&first_ty, &ty, expr.span)?;
                    }
                    Ok(Type::Array(Box::new(first_ty)))
                }
            }
            Expr::Record(fields) => {
                let field_types: Vec<(String, Type)> = fields
                    .iter()
                    .map(|(name, expr)| {
                        let ty = self.infer_spanned_expr(expr)?;
                        Ok((name.node.clone(), ty))
                    })
                    .collect::<Result<_, TypeError>>()?;
                Ok(Type::Record(field_types))
            }
            Expr::Element(element) => self.check_element(element),
            Expr::Match(_, _) => {
                // TODO: Implement match expression type checking
                Ok(Type::Var(TypeVar::fresh()))
            }
        }
    }

    /// Infer type of a spanned expression.
    fn infer_spanned_expr(&mut self, expr: &Spanned<Expr>) -> TypeResult<Type> {
        self.infer_expr(&expr.node)
    }

    /// Check a statement.
    fn check_stmt(&mut self, stmt: &Stmt) -> TypeResult<Type> {
        match stmt {
            Stmt::Expr(expr) => self.infer_expr(expr),
            Stmt::Let(name, ty_ann, value) => {
                let value_ty = self.infer_expr(value)?;

                if let Some(ty_expr) = ty_ann {
                    let ann_ty = self.resolve_type_expr(ty_expr)?;
                    self.unify(&value_ty, &ann_ty, name.span)?;
                }

                let scheme = self.env.generalize(&value_ty);
                self.env.define(&name.node, scheme);

                Ok(Type::Unit)
            }
            Stmt::Assign(target, value) => {
                let target_ty = self.infer_spanned_expr(target)?;
                let value_ty = self.infer_expr(value)?;
                self.unify(&target_ty, &value_ty, target.span)?;
                Ok(Type::Unit)
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.infer_expr(expr)
                } else {
                    Ok(Type::Unit)
                }
            }
            Stmt::For(var, iter, body) => {
                let iter_ty = self.infer_expr(iter)?;
                let elem_ty = match iter_ty {
                    Type::Array(t) => *t,
                    _ => Type::Var(TypeVar::fresh()),
                };

                self.env.push_scope();
                self.env.define(&var.node, TypeScheme::mono(elem_ty));
                for stmt in body {
                    self.check_stmt(&stmt.node)?;
                }
                self.env.pop_scope();

                Ok(Type::Unit)
            }
            Stmt::While(cond, body) => {
                let cond_ty = self.infer_expr(cond)?;
                self.unify(&cond_ty, &Type::Bool, Span::new(0, 0, self.file_id))?;

                self.env.push_scope();
                for stmt in body {
                    self.check_stmt(&stmt.node)?;
                }
                self.env.pop_scope();

                Ok(Type::Unit)
            }
        }
    }

    /// Check a binary operation.
    fn check_binop(
        &mut self,
        op: BinOp,
        left: &Type,
        right: &Type,
        span: Span,
    ) -> TypeResult<Type> {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                self.unify(left, right, span)?;
                // Numeric operations return the same type
                Ok(left.clone())
            }
            BinOp::Eq | BinOp::NotEq => {
                self.unify(left, right, span)?;
                Ok(Type::Bool)
            }
            BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
                self.unify(left, right, span)?;
                Ok(Type::Bool)
            }
            BinOp::And | BinOp::Or => {
                self.unify(left, &Type::Bool, span)?;
                self.unify(right, &Type::Bool, span)?;
                Ok(Type::Bool)
            }
        }
    }

    /// Check a unary operation.
    fn check_unaryop(&mut self, op: UnaryOp, operand: &Type, span: Span) -> TypeResult<Type> {
        match op {
            UnaryOp::Neg => {
                // Negation works on numeric types
                Ok(operand.clone())
            }
            UnaryOp::Not => {
                self.unify(operand, &Type::Bool, span)?;
                Ok(Type::Bool)
            }
        }
    }

    /// Check a function call.
    fn check_call(
        &mut self,
        func: &Type,
        args: &[Type],
        span: Span,
    ) -> TypeResult<Type> {
        match func {
            Type::Fn(params, ret) => {
                if params.len() != args.len() {
                    return Err(TypeError {
                        message: format!(
                            "function expects {} arguments, got {}",
                            params.len(),
                            args.len()
                        ),
                        span,
                        expected: None,
                        found: None,
                    });
                }

                for (param, arg) in params.iter().zip(args.iter()) {
                    self.unify(param, arg, span)?;
                }

                Ok((**ret).clone())
            }
            Type::Var(_) => {
                // Function type is unknown, create fresh types
                let ret = Type::Var(TypeVar::fresh());
                let expected = Type::Fn(args.to_vec(), Box::new(ret.clone()));
                self.unify(func, &expected, span)?;
                Ok(ret)
            }
            _ => Err(TypeError {
                message: format!("cannot call non-function type: {}", func),
                span,
                expected: Some(Type::Fn(vec![], Box::new(Type::Var(TypeVar::fresh())))),
                found: Some(func.clone()),
            }),
        }
    }

    /// Check field access.
    fn check_field_access(
        &mut self,
        base: &Type,
        field: &str,
        span: Span,
    ) -> TypeResult<Type> {
        match base {
            Type::Record(fields) => {
                for (name, ty) in fields {
                    if name == field {
                        return Ok(ty.clone());
                    }
                }
                Err(TypeError {
                    message: format!("record has no field '{}'", field),
                    span,
                    expected: None,
                    found: None,
                })
            }
            Type::Var(_) => {
                // Unknown type, assume it has the field
                Ok(Type::Var(TypeVar::fresh()))
            }
            _ => Err(TypeError {
                message: format!("cannot access field '{}' on type {}", field, base),
                span,
                expected: None,
                found: Some(base.clone()),
            }),
        }
    }

    /// Check index access.
    fn check_index(&mut self, base: &Type, span: Span) -> TypeResult<Type> {
        match base {
            Type::Array(elem) => Ok((**elem).clone()),
            Type::Tuple(elems) => {
                // For tuples, we'd need the index to be a literal
                // For now, return a fresh type variable
                if elems.is_empty() {
                    Ok(Type::Var(TypeVar::fresh()))
                } else {
                    Ok(elems[0].clone())
                }
            }
            Type::Var(_) => Ok(Type::Var(TypeVar::fresh())),
            _ => Err(TypeError {
                message: format!("cannot index into type {}", base),
                span,
                expected: None,
                found: Some(base.clone()),
            }),
        }
    }

    /// Resolve a type expression to a Type.
    fn resolve_type_expr(&mut self, ty_expr: &TypeExpr) -> TypeResult<Type> {
        match ty_expr {
            TypeExpr::Named(ident) => {
                // Look up built-in types
                match ident.node.as_str() {
                    "Int" => Ok(Type::Int),
                    "Float" => Ok(Type::Float),
                    "Bool" => Ok(Type::Bool),
                    "String" => Ok(Type::String),
                    "Time" => Ok(Type::Time),
                    "Duration" => Ok(Type::Duration),
                    "Position" => Ok(Type::Position),
                    "Position3" => Ok(Type::Position3),
                    "Angle" => Ok(Type::Angle),
                    "Velocity" => Ok(Type::Velocity),
                    "Beat" => Ok(Type::Beat),
                    "Edge" => Ok(Type::Edge),
                    "Level" => Ok(Type::Level),
                    "Jump" => Ok(Type::Jump),
                    "Spin" => Ok(Type::Spin),
                    "StepSequence" => Ok(Type::StepSequence),
                    "Lift" => Ok(Type::Lift),
                    "Throw" => Ok(Type::Throw),
                    "Twist" => Ok(Type::Twist),
                    "DeathSpiral" => Ok(Type::DeathSpiral),
                    "Transition" => Ok(Type::Transition),
                    "Choreographic" => Ok(Type::Choreographic),
                    "Element" => Ok(Type::Element),
                    "Sequence" => Ok(Type::Sequence),
                    "Segment" => Ok(Type::Segment),
                    "Program" => Ok(Type::Program),
                    _ => {
                        // Check user-defined types
                        if self.env.lookup_type(&ident.node).is_some() {
                            Ok(Type::Named(ident.node.clone()))
                        } else {
                            Err(TypeError {
                                message: format!("unknown type: {}", ident.node),
                                span: ident.span,
                                expected: None,
                                found: None,
                            })
                        }
                    }
                }
            }
            TypeExpr::Fn(params, ret) => {
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|t| self.resolve_type_expr(t))
                    .collect::<Result<_, _>>()?;
                let ret_type = self.resolve_type_expr(ret)?;
                Ok(Type::Fn(param_types, Box::new(ret_type)))
            }
            TypeExpr::Tuple(types) => {
                let types: Vec<Type> = types
                    .iter()
                    .map(|t| self.resolve_type_expr(t))
                    .collect::<Result<_, _>>()?;
                Ok(Type::Tuple(types))
            }
            TypeExpr::Array(elem) => {
                let elem_type = self.resolve_type_expr(elem)?;
                Ok(Type::Array(Box::new(elem_type)))
            }
            TypeExpr::Optional(inner) => {
                let inner_type = self.resolve_type_expr(inner)?;
                Ok(Type::Optional(Box::new(inner_type)))
            }
            TypeExpr::App(base, args) => {
                let base_type = self.resolve_type_expr(base)?;
                let arg_types: Vec<Type> = args
                    .iter()
                    .map(|t| self.resolve_type_expr(t))
                    .collect::<Result<_, _>>()?;
                Ok(Type::App(Box::new(base_type), arg_types))
            }
            TypeExpr::Refinement { base, var: _, predicate: _ } => {
                // TODO: Handle refinement types properly with SMT
                let base_type = self.resolve_type_expr(base)?;
                Ok(base_type)
            }
        }
    }

    /// Unify two types.
    fn unify(&mut self, t1: &Type, t2: &Type, span: Span) -> TypeResult<()> {
        let t1 = self.apply_subst(t1);
        let t2 = self.apply_subst(t2);

        match (&t1, &t2) {
            // Same types unify
            (a, b) if a == b => Ok(()),

            // Type variables
            (Type::Var(v), t) | (t, Type::Var(v)) => {
                if t.free_vars().contains(v) {
                    Err(TypeError {
                        message: format!("infinite type: {} ~ {}", v, t),
                        span,
                        expected: None,
                        found: None,
                    })
                } else {
                    self.subst.insert(*v, t.clone());
                    Ok(())
                }
            }

            // Function types
            (Type::Fn(p1, r1), Type::Fn(p2, r2)) => {
                if p1.len() != p2.len() {
                    return Err(TypeError::mismatch(t1.clone(), t2.clone(), span));
                }
                for (a, b) in p1.iter().zip(p2.iter()) {
                    self.unify(a, b, span)?;
                }
                self.unify(r1, r2, span)
            }

            // Tuple types
            (Type::Tuple(ts1), Type::Tuple(ts2)) => {
                if ts1.len() != ts2.len() {
                    return Err(TypeError::mismatch(t1.clone(), t2.clone(), span));
                }
                for (a, b) in ts1.iter().zip(ts2.iter()) {
                    self.unify(a, b, span)?;
                }
                Ok(())
            }

            // Array types
            (Type::Array(e1), Type::Array(e2)) => self.unify(e1, e2, span),

            // Optional types
            (Type::Optional(e1), Type::Optional(e2)) => self.unify(e1, e2, span),

            // Record types
            (Type::Record(f1), Type::Record(f2)) => {
                if f1.len() != f2.len() {
                    return Err(TypeError::mismatch(t1.clone(), t2.clone(), span));
                }
                for ((n1, t1), (n2, t2)) in f1.iter().zip(f2.iter()) {
                    if n1 != n2 {
                        return Err(TypeError {
                            message: format!("field name mismatch: {} vs {}", n1, n2),
                            span,
                            expected: None,
                            found: None,
                        });
                    }
                    self.unify(t1, t2, span)?;
                }
                Ok(())
            }

            // Element subtyping
            (Type::Jump, Type::Element)
            | (Type::Spin, Type::Element)
            | (Type::StepSequence, Type::Element)
            | (Type::Lift, Type::Element)
            | (Type::Throw, Type::Element)
            | (Type::Twist, Type::Element)
            | (Type::DeathSpiral, Type::Element)
            | (Type::Transition, Type::Element)
            | (Type::Choreographic, Type::Element) => Ok(()),

            // Error type unifies with anything (to allow error recovery)
            (Type::Error, _) | (_, Type::Error) => Ok(()),

            // No match
            _ => Err(TypeError::mismatch(t1, t2, span)),
        }
    }

    /// Apply the current substitution to a type.
    fn apply_subst(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => {
                if let Some(t) = self.subst.get(v) {
                    self.apply_subst(t)
                } else {
                    ty.clone()
                }
            }
            Type::Fn(params, ret) => Type::Fn(
                params.iter().map(|t| self.apply_subst(t)).collect(),
                Box::new(self.apply_subst(ret)),
            ),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| self.apply_subst(t)).collect()),
            Type::Array(t) => Type::Array(Box::new(self.apply_subst(t))),
            Type::Optional(t) => Type::Optional(Box::new(self.apply_subst(t))),
            Type::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.apply_subst(t)))
                    .collect(),
            ),
            Type::App(base, args) => Type::App(
                Box::new(self.apply_subst(base)),
                args.iter().map(|t| self.apply_subst(t)).collect(),
            ),
            _ => ty.clone(),
        }
    }

    /// Get all collected diagnostics.
    pub fn into_diagnostics(self) -> Diagnostics {
        let mut diagnostics = Diagnostics::new();
        for error in self.errors {
            diagnostics.add(error.to_diagnostic());
        }
        diagnostics
    }
}

/// Convenience function to type-check a program.
pub fn check(program: &Program, file_id: FileId) -> Result<(), Diagnostics> {
    let mut checker = TypeChecker::new(file_id);
    match checker.check_program(program) {
        Ok(()) => Ok(()),
        Err(_) => Err(checker.into_diagnostics()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anv_syntax::parse;

    fn check_source(source: &str) -> Result<(), Vec<TypeError>> {
        let program = parse(source, FileId(0)).expect("parse error");
        let mut checker = TypeChecker::new(FileId(0));
        checker.check_program(&program)
    }

    #[test]
    fn test_empty_program() {
        let result = check_source("program test {}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_program_with_jump() {
        let result = check_source(
            r#"
            program test {
                segment main: short {
                    sequence {
                        jump triple axel at 1:30
                    }
                }
            }
        "#,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_program_with_spin() {
        let result = check_source(
            r#"
            program test {
                segment main: free {
                    sequence {
                        spin camel sit upright L3
                    }
                }
            }
        "#,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_definition() {
        let result = check_source(
            r#"
            program test {
                fn add(x: Int, y: Int) -> Int = x
            }
        "#,
        );
        assert!(result.is_ok());
    }
}
