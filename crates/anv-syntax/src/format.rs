// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Formatter for Anvomidav source code.
//!
//! Produces consistent, readable formatting for .anv files.

use crate::ast::*;

/// Formatting options.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Indentation string (default: 4 spaces).
    pub indent: String,
    /// Maximum line width before breaking.
    pub max_width: usize,
    /// Add blank lines between segments.
    pub blank_between_segments: bool,
    /// Add blank lines between sequences.
    pub blank_between_sequences: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent: "    ".to_string(),
            max_width: 100,
            blank_between_segments: true,
            blank_between_sequences: false,
        }
    }
}

/// Formatter for Anvomidav programs.
pub struct Formatter {
    options: FormatOptions,
    output: String,
    indent_level: usize,
}

impl Formatter {
    /// Create a new formatter with default options.
    pub fn new() -> Self {
        Self::with_options(FormatOptions::default())
    }

    /// Create a new formatter with custom options.
    pub fn with_options(options: FormatOptions) -> Self {
        Self {
            options,
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Format a program.
    pub fn format(&mut self, program: &Program) -> String {
        self.output.clear();
        self.format_program(program);
        self.output.clone()
    }

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str(&self.options.indent);
        }
    }

    fn newline(&mut self) {
        self.output.push('\n');
    }

    fn format_program(&mut self, program: &Program) {
        // Doc comments
        for doc in &program.docs {
            self.output.push_str("/// ");
            self.output.push_str(doc);
            self.newline();
        }

        // Program declaration
        self.output.push_str("program ");
        self.output.push_str(&program.name.node);
        self.output.push_str(" {");
        self.newline();
        self.indent_level += 1;

        // Imports
        for import in &program.imports {
            self.format_import(import);
        }

        if !program.imports.is_empty() && !program.segments.is_empty() {
            self.newline();
        }

        // Functions
        for func in &program.functions {
            self.format_fn_def(func);
            self.newline();
        }

        // Segments
        for (i, segment) in program.segments.iter().enumerate() {
            if i > 0 && self.options.blank_between_segments {
                self.newline();
            }
            self.format_segment(segment);
        }

        self.indent_level -= 1;
        self.output.push('}');
        self.newline();
    }

    fn format_import(&mut self, import: &Import) {
        self.indent();
        self.output.push_str("import ");

        let path: Vec<_> = import.path.iter().map(|p| p.node.as_str()).collect();
        self.output.push_str(&path.join("::"));

        if let Some(alias) = &import.alias {
            self.output.push_str(" as ");
            self.output.push_str(&alias.node);
        }

        self.newline();
    }

    fn format_fn_def(&mut self, func: &FnDef) {
        self.indent();
        self.output.push_str("fn ");
        self.output.push_str(&func.name.node);
        self.output.push('(');

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&param.name.node);
            if let Some(ty) = &param.ty {
                self.output.push_str(": ");
                self.format_type(ty);
            }
        }

        self.output.push(')');

        if let Some(ret) = &func.return_ty {
            self.output.push_str(" -> ");
            self.format_type(ret);
        }

        self.output.push_str(" = ");
        self.format_expr(&func.body);
        self.newline();
    }

    fn format_type(&mut self, ty: &TypeExpr) {
        match ty {
            TypeExpr::Named(name) => {
                self.output.push_str(&name.node);
            }
            TypeExpr::Array(inner) => {
                self.output.push('[');
                self.format_type(inner);
                self.output.push(']');
            }
            TypeExpr::Optional(inner) => {
                self.format_type(inner);
                self.output.push('?');
            }
            TypeExpr::Tuple(types) => {
                self.output.push('(');
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_type(t);
                }
                self.output.push(')');
            }
            TypeExpr::App(base, args) => {
                self.format_type(base);
                self.output.push('<');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_type(arg);
                }
                self.output.push('>');
            }
            TypeExpr::Fn(params, ret) => {
                self.output.push_str("fn(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_type(p);
                }
                self.output.push_str(") -> ");
                self.format_type(ret);
            }
            TypeExpr::Refinement { base, var, predicate } => {
                self.output.push_str("{ ");
                self.output.push_str(&var.node);
                self.output.push_str(": ");
                self.format_type(base);
                self.output.push_str(" | ");
                self.format_expr(predicate);
                self.output.push_str(" }");
            }
        }
    }

    fn format_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(n) => {
                self.output.push_str(&n.to_string());
            }
            Expr::Float(f) => {
                self.output.push_str(&f.to_string());
            }
            Expr::String(s) => {
                self.output.push('"');
                self.output.push_str(s);
                self.output.push('"');
            }
            Expr::Bool(b) => {
                self.output.push_str(if *b { "true" } else { "false" });
            }
            Expr::Time(time) => {
                self.format_time_expr(time);
            }
            Expr::Position(pos) => {
                self.format_position_expr(pos);
            }
            Expr::Var(v) => {
                self.output.push_str(&v.node);
            }
            Expr::BinOp(left, op, right) => {
                self.format_expr(&left.node);
                self.output.push(' ');
                self.format_binop(op);
                self.output.push(' ');
                self.format_expr(&right.node);
            }
            Expr::UnaryOp(op, expr) => {
                match op {
                    UnaryOp::Neg => self.output.push('-'),
                    UnaryOp::Not => self.output.push('!'),
                }
                self.format_expr(&expr.node);
            }
            Expr::Tuple(exprs) => {
                self.output.push('(');
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expr(&e.node);
                }
                self.output.push(')');
            }
            Expr::Array(exprs) => {
                self.output.push('[');
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expr(&e.node);
                }
                self.output.push(']');
            }
            Expr::Block(stmts) => {
                self.output.push_str("{ ");
                for (i, stmt) in stmts.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str("; ");
                    }
                    self.format_stmt(&stmt.node);
                }
                self.output.push_str(" }");
            }
            Expr::If(cond, then_branch, else_branch) => {
                self.output.push_str("if ");
                self.format_expr(&cond.node);
                self.output.push_str(" then ");
                self.format_expr(&then_branch.node);
                if let Some(else_b) = else_branch {
                    self.output.push_str(" else ");
                    self.format_expr(&else_b.node);
                }
            }
            Expr::Let(name, ty, value, body) => {
                self.output.push_str("let ");
                self.output.push_str(&name.node);
                if let Some(t) = ty {
                    self.output.push_str(": ");
                    self.format_type(t);
                }
                self.output.push_str(" = ");
                self.format_expr(&value.node);
                self.output.push_str(" in ");
                self.format_expr(&body.node);
            }
            Expr::Call(func, args) => {
                self.format_expr(&func.node);
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expr(&arg.node);
                }
                self.output.push(')');
            }
            Expr::Field(expr, field) => {
                self.format_expr(&expr.node);
                self.output.push('.');
                self.output.push_str(&field.node);
            }
            Expr::Index(expr, idx) => {
                self.format_expr(&expr.node);
                self.output.push('[');
                self.format_expr(&idx.node);
                self.output.push(']');
            }
            Expr::Match(expr, arms) => {
                self.output.push_str("match ");
                self.format_expr(&expr.node);
                self.output.push_str(" { ");
                for (i, arm) in arms.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_pattern(&arm.pattern);
                    self.output.push_str(" => ");
                    self.format_expr(&arm.body.node);
                }
                self.output.push_str(" }");
            }
            Expr::Lambda(params, body) => {
                self.output.push_str("|");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&p.name.node);
                    if let Some(t) = &p.ty {
                        self.output.push_str(": ");
                        self.format_type(t);
                    }
                }
                self.output.push_str("| ");
                self.format_expr(&body.node);
            }
            Expr::Record(fields) => {
                self.output.push_str("{ ");
                for (i, (name, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&name.node);
                    self.output.push_str(": ");
                    self.format_expr(&val.node);
                }
                self.output.push_str(" }");
            }
            Expr::Element(elem) => {
                self.format_element(elem);
            }
        }
    }

    fn format_position_expr(&mut self, pos: &PositionExpr) {
        match pos {
            PositionExpr::Literal(x, y) => {
                self.output.push_str(&format!("({}, {})", x, y));
            }
            PositionExpr::Named(name) => {
                self.output.push_str(name);
            }
            PositionExpr::Var(v) => {
                self.output.push_str(&v.node);
            }
            PositionExpr::Relative(base, dx, dy) => {
                self.format_position_expr(base);
                self.output.push_str(&format!(" + ({}, {})", dx, dy));
            }
        }
    }

    fn format_pattern(&mut self, _pattern: &Pattern) {
        // Simplified pattern formatting
        self.output.push_str("_");
    }

    fn format_binop(&mut self, op: &BinOp) {
        let s = match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Eq => "==",
            BinOp::NotEq => "!=",
            BinOp::Lt => "<",
            BinOp::LtEq => "<=",
            BinOp::Gt => ">",
            BinOp::GtEq => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        };
        self.output.push_str(s);
    }

    fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(e) => self.format_expr(e),
            Stmt::Let(name, ty, value) => {
                self.output.push_str("let ");
                self.output.push_str(&name.node);
                if let Some(t) = ty {
                    self.output.push_str(": ");
                    self.format_type(t);
                }
                self.output.push_str(" = ");
                self.format_expr(value);
            }
            Stmt::Assign(target, value) => {
                self.format_expr(&target.node);
                self.output.push_str(" = ");
                self.format_expr(value);
            }
            Stmt::Return(expr) => {
                self.output.push_str("return");
                if let Some(e) = expr {
                    self.output.push(' ');
                    self.format_expr(e);
                }
            }
            Stmt::For(name, iter, body) => {
                self.output.push_str("for ");
                self.output.push_str(&name.node);
                self.output.push_str(" in ");
                self.format_expr(iter);
                self.output.push_str(" { ");
                for stmt in body {
                    self.format_stmt(&stmt.node);
                    self.output.push_str("; ");
                }
                self.output.push('}');
            }
            Stmt::While(cond, body) => {
                self.output.push_str("while ");
                self.format_expr(cond);
                self.output.push_str(" { ");
                for stmt in body {
                    self.format_stmt(&stmt.node);
                    self.output.push_str("; ");
                }
                self.output.push('}');
            }
        }
    }

    fn format_transition(&mut self, trans: &TransitionElement) {
        self.output.push_str("transition");
        if let Some(desc) = &trans.description {
            self.output.push(' ');
            self.output.push('"');
            self.output.push_str(desc);
            self.output.push('"');
        }
    }

    fn format_pattern_element(&mut self, pattern: &PatternElement) {
        self.output.push_str("pattern ");
        self.output.push('"');
        self.output.push_str(&pattern.name);
        self.output.push('"');
    }

    fn format_parallel(&mut self, parallel: &ParallelElement) {
        self.output.push_str("parallel {");
        self.newline();
        self.indent_level += 1;
        for (skater, elements) in &parallel.branches {
            self.indent();
            self.output.push_str(&skater.node);
            self.output.push_str(": {");
            self.newline();
            self.indent_level += 1;
            for elem in elements {
                self.format_element(elem);
            }
            self.indent_level -= 1;
            self.indent();
            self.output.push('}');
            self.newline();
        }
        self.indent_level -= 1;
        self.indent();
        self.output.push('}');
    }

    fn format_sync(&mut self, sync: &SyncElement) {
        self.output.push_str("sync {");
        self.newline();
        self.indent_level += 1;
        for elem in &sync.elements {
            self.format_element(elem);
        }
        self.indent_level -= 1;
        self.indent();
        self.output.push('}');
    }

    fn format_segment(&mut self, segment: &Segment) {
        // Doc comments
        for doc in &segment.docs {
            self.indent();
            self.output.push_str("/// ");
            self.output.push_str(doc);
            self.newline();
        }

        self.indent();
        self.output.push_str("segment ");
        self.output.push_str(&segment.name.node);
        self.output.push_str(": ");
        self.format_segment_kind(&segment.kind);
        self.output.push_str(" {");
        self.newline();
        self.indent_level += 1;

        for (i, seq) in segment.sequences.iter().enumerate() {
            if i > 0 && self.options.blank_between_sequences {
                self.newline();
            }
            self.format_sequence(seq);
        }

        self.indent_level -= 1;
        self.indent();
        self.output.push('}');
        self.newline();
    }

    fn format_segment_kind(&mut self, kind: &SegmentKind) {
        let s = match kind {
            SegmentKind::Short => "short",
            SegmentKind::Free => "free",
            SegmentKind::Pattern => "pattern",
            SegmentKind::Rhythm => "rhythm",
            SegmentKind::Exhibition => "exhibition",
        };
        self.output.push_str(s);
    }

    fn format_sequence(&mut self, seq: &Sequence) {
        self.indent();
        self.output.push_str("sequence");
        if let Some(name) = &seq.name {
            self.output.push(' ');
            self.output.push_str(&name.node);
        }
        self.output.push_str(" {");
        self.newline();
        self.indent_level += 1;

        for element in &seq.elements {
            self.format_element(element);
        }

        self.indent_level -= 1;
        self.indent();
        self.output.push('}');
        self.newline();
    }

    fn format_element(&mut self, element: &Element) {
        self.indent();

        match &element.kind {
            ElementKind::Jump(jump) => {
                self.output.push_str("jump ");
                self.format_rotations(&jump.rotations);
                self.output.push(' ');
                self.format_jump_kind(&jump.kind);
            }
            ElementKind::Spin(spin) => {
                self.output.push_str("spin ");
                for (i, pos) in spin.positions.iter().enumerate() {
                    if i > 0 {
                        self.output.push(' ');
                    }
                    self.format_spin_position(&pos.position);
                }
                if let Some(level) = &spin.level {
                    self.output.push(' ');
                    self.format_level(level);
                }
            }
            ElementKind::StepSequence(step) => {
                self.output.push_str("step ");
                self.format_step_pattern(&step.pattern);
                if let Some(level) = &step.level {
                    self.output.push(' ');
                    self.format_level(level);
                }
            }
            ElementKind::Lift(lift) => {
                self.output.push_str("lift ");
                self.format_lift_group(&lift.group);
                if let Some(level) = &lift.level {
                    self.output.push(' ');
                    self.format_level(level);
                }
            }
            ElementKind::Throw(throw) => {
                self.output.push_str("throw ");
                self.format_rotations(&throw.rotations);
                self.output.push(' ');
                self.format_jump_kind(&throw.kind);
            }
            ElementKind::Twist(twist) => {
                self.output.push_str("twist ");
                self.format_rotations(&twist.rotations);
                if let Some(level) = &twist.level {
                    self.output.push(' ');
                    self.format_level(level);
                }
            }
            ElementKind::DeathSpiral(ds) => {
                self.output.push_str("death_spiral ");
                self.format_edge(&ds.edge);
                if let Some(level) = &ds.level {
                    self.output.push(' ');
                    self.format_level(level);
                }
            }
            ElementKind::Choreographic(choreo) => {
                self.output.push_str("choreographic ");
                self.format_choreo_kind(&choreo.kind);
            }
            ElementKind::Transition(trans) => {
                self.format_transition(trans);
            }
            ElementKind::Pattern(pattern) => {
                self.format_pattern_element(pattern);
            }
            ElementKind::Parallel(parallel) => {
                self.format_parallel(parallel);
            }
            ElementKind::Sync(sync) => {
                self.format_sync(sync);
            }
        }

        // Timing
        if let Some(timing) = &element.timing {
            self.output.push(' ');
            self.format_timing(timing);
        }

        self.newline();
    }

    fn format_rotations(&mut self, rot: &anv_core::skating::Rotations) {
        use anv_core::skating::Rotations;
        let s = match rot {
            Rotations::Single => "single",
            Rotations::Double => "double",
            Rotations::Triple => "triple",
            Rotations::Quad => "quad",
        };
        self.output.push_str(s);
    }

    fn format_jump_kind(&mut self, kind: &anv_core::skating::JumpKind) {
        use anv_core::skating::JumpKind;
        let s = match kind {
            JumpKind::Axel => "axel",
            JumpKind::Lutz => "lutz",
            JumpKind::Flip => "flip",
            JumpKind::Loop => "loop",
            JumpKind::Salchow => "salchow",
            JumpKind::ToeLoop => "toe_loop",
            JumpKind::Euler => "euler",
        };
        self.output.push_str(s);
    }

    fn format_spin_position(&mut self, pos: &anv_core::skating::SpinPosition) {
        use anv_core::skating::SpinPosition;
        let s = match pos {
            SpinPosition::Upright => "upright",
            SpinPosition::Sit => "sit",
            SpinPosition::Camel => "camel",
            SpinPosition::Layback => "layback",
            SpinPosition::Biellmann => "biellmann",
        };
        self.output.push_str(s);
    }

    fn format_step_pattern(&mut self, pattern: &StepPattern) {
        let s = match pattern {
            StepPattern::Straight => "straight",
            StepPattern::Circular => "circular",
            StepPattern::Serpentine => "serpentine",
            StepPattern::Diagonal => "diagonal",
            StepPattern::Midline => "midline",
        };
        self.output.push_str(s);
    }

    fn format_lift_group(&mut self, group: &anv_core::skating::LiftGroup) {
        use anv_core::skating::LiftGroup;
        let s = match group {
            LiftGroup::Group1 => "Gr1",
            LiftGroup::Group2 => "Gr2",
            LiftGroup::Group3 => "Gr3",
            LiftGroup::Group4 => "Gr4",
            LiftGroup::Group5 => "Gr5",
        };
        self.output.push_str(s);
    }

    fn format_edge(&mut self, edge: &anv_core::skating::Edge) {
        use anv_core::skating::Edge;
        let s = match edge {
            Edge::LFO => "LFO",
            Edge::LFI => "LFI",
            Edge::LBO => "LBO",
            Edge::LBI => "LBI",
            Edge::RFO => "RFO",
            Edge::RFI => "RFI",
            Edge::RBO => "RBO",
            Edge::RBI => "RBI",
        };
        self.output.push_str(s);
    }

    fn format_choreo_kind(&mut self, kind: &ChoreographicKind) {
        let s = match kind {
            ChoreographicKind::Spiral => "spiral",
            ChoreographicKind::Spread => "spread",
            ChoreographicKind::Ina => "ina",
            ChoreographicKind::Hydroblading => "hydroblading",
            ChoreographicKind::Pivot => "pivot",
            ChoreographicKind::Choreographic => "choreographic",
        };
        self.output.push_str(s);
    }

    fn format_level(&mut self, level: &anv_core::skating::Level) {
        use anv_core::skating::Level;
        let s = match level {
            Level::B => "B",
            Level::L1 => "L1",
            Level::L2 => "L2",
            Level::L3 => "L3",
            Level::L4 => "L4",
        };
        self.output.push_str(s);
    }

    fn format_timing(&mut self, timing: &Timing) {
        match timing {
            Timing::At(time) => {
                self.output.push_str("at ");
                self.format_time_expr(time);
            }
            Timing::Range(start, end) => {
                self.format_time_expr(start);
                self.output.push_str("..");
                self.format_time_expr(end);
            }
            Timing::Duration(time) => {
                self.output.push_str("duration ");
                self.format_time_expr(time);
            }
            Timing::Beat(n) => {
                self.output.push_str("beat ");
                self.output.push_str(&n.to_string());
            }
            Timing::After(time) => {
                self.output.push_str("after ");
                self.format_time_expr(time);
            }
        }
    }

    fn format_time_expr(&mut self, time: &TimeExpr) {
        match time {
            TimeExpr::Literal(f) => {
                self.output.push_str(&f.to_string());
            }
            TimeExpr::Formatted(s) => {
                self.output.push_str(s);
            }
            TimeExpr::Var(v) => {
                self.output.push_str(&v.node);
            }
            TimeExpr::BinOp(a, op, b) => {
                self.format_time_expr(a);
                let op_str = match op {
                    TimeBinOp::Add => " + ",
                    TimeBinOp::Sub => " - ",
                };
                self.output.push_str(op_str);
                self.format_time_expr(b);
            }
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Format source code.
pub fn format(source: &str) -> Result<String, Vec<crate::ParseError>> {
    let program = crate::parse(source, anv_core::source::FileId(0))?;
    let mut formatter = Formatter::new();
    Ok(formatter.format(&program))
}

/// Format source code with custom options.
pub fn format_with_options(source: &str, options: FormatOptions) -> Result<String, Vec<crate::ParseError>> {
    let program = crate::parse(source, anv_core::source::FileId(0))?;
    let mut formatter = Formatter::with_options(options);
    Ok(formatter.format(&program))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_program() {
        let source = r#"program test {
segment sp: short {
sequence {
jump triple axel
}
}
}"#;
        let result = format(source).unwrap();
        assert!(result.contains("program test"));
        assert!(result.contains("    segment sp: short"));
        assert!(result.contains("        sequence"));
        assert!(result.contains("            jump triple axel"));
    }

    #[test]
    fn test_format_preserves_doc_comments() {
        let source = r#"/// This is a test
program test {}"#;
        let result = format(source).unwrap();
        assert!(result.starts_with("/// This is a test"));
    }

    #[test]
    fn test_format_with_timing() {
        let source = "program t { segment s: free { sequence { jump triple lutz at 1:30 } } }";
        let result = format(source).unwrap();
        assert!(result.contains("jump triple lutz at 1:30"));
    }

    #[test]
    fn test_format_spin_with_positions() {
        let source = "program t { segment s: free { sequence { spin camel sit upright L4 } } }";
        let result = format(source).unwrap();
        assert!(result.contains("spin camel sit upright L4"));
    }

    #[test]
    fn test_format_pairs_elements() {
        let source = "program t { segment s: short { sequence { lift Gr5 L4 throw triple lutz death_spiral LBI L3 } } }";
        let result = format(source).unwrap();
        assert!(result.contains("lift Gr5 L4"));
        assert!(result.contains("throw triple lutz"));
        assert!(result.contains("death_spiral LBI L3"));
    }
}
