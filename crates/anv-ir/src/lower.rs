// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! AST to IR lowering.
//!
//! This module transforms the hierarchical AST into a flat timeline representation.

use crate::rink::Position;
use crate::timeline::{
    ChoreographicKind, Event, EventKind, SegmentMarker, StepPattern, Timeline,
};
use crate::types::{Duration, TimeCode};
use anv_syntax::ast::{
    ChoreographicElement, DeathSpiralElement, Element, ElementKind, JumpElement, LiftElement,
    PatternElement, Program, Segment, Sequence, SpinElement, StepSequence, ThrowElement,
    TwistElement,
};

/// Lower an AST Program to an IR Timeline.
pub fn lower(program: &Program) -> Timeline {
    let mut lowerer = Lowerer::new(&program.name.node);
    lowerer.lower_program(program);
    lowerer.timeline
}

/// Lowering context.
struct Lowerer {
    timeline: Timeline,
    current_time: TimeCode,
    next_event_id: u32,
}

impl Lowerer {
    fn new(name: &str) -> Self {
        Self {
            timeline: Timeline::new(name),
            current_time: TimeCode::ZERO,
            next_event_id: 1,
        }
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_event_id;
        self.next_event_id += 1;
        id
    }

    fn lower_program(&mut self, program: &Program) {
        // Add program start event
        let start_id = self.next_id();
        self.timeline.push_event(
            Event::new(start_id, EventKind::ProgramStart, TimeCode::ZERO)
                .with_label(&program.name.node),
        );

        // Lower each segment
        for segment in &program.segments {
            self.lower_segment(segment);
        }

        // Add program end event
        let end_id = self.next_id();
        let end_time = self.current_time;
        self.timeline.push_event(
            Event::new(end_id, EventKind::ProgramEnd, end_time),
        );
    }

    fn lower_segment(&mut self, segment: &Segment) {
        let segment_start = self.current_time;

        // Lower each sequence in the segment
        for sequence in &segment.sequences {
            self.lower_sequence(sequence);
        }

        // Record segment marker
        self.timeline.push_segment(SegmentMarker {
            name: segment.name.node.clone(),
            kind: segment.kind.to_string(),
            start: segment_start,
            end: self.current_time,
        });
    }

    fn lower_sequence(&mut self, sequence: &Sequence) {
        for element in &sequence.elements {
            self.lower_element(element);
        }
    }

    fn lower_element(&mut self, element: &Element) {
        let event = match &element.kind {
            ElementKind::Jump(jump) => self.lower_jump(jump),
            ElementKind::Spin(spin) => self.lower_spin(spin),
            ElementKind::StepSequence(step) => self.lower_step(step),
            ElementKind::Lift(lift) => self.lower_lift(lift),
            ElementKind::Throw(throw) => self.lower_throw(throw),
            ElementKind::Twist(twist) => self.lower_twist(twist),
            ElementKind::DeathSpiral(ds) => self.lower_death_spiral(ds),
            ElementKind::Choreographic(choreo) => self.lower_choreographic(choreo),
            ElementKind::Pattern(pattern) => self.lower_pattern(pattern),
            // Skip structural elements for now
            ElementKind::Parallel(_) | ElementKind::Sync(_) | ElementKind::Transition(_) => return,
        };

        if let Some(event) = event {
            self.current_time = self.current_time + event.duration;
            self.timeline.push_event(event);
        }
    }

    fn lower_jump(&mut self, jump: &JumpElement) -> Option<Event> {
        let id = self.next_id();
        let kind = EventKind::Jump {
            rotations: jump.rotations,
            kind: jump.kind,
        };

        let mut event = Event::new(id, kind, self.current_time)
            .with_duration(Duration::jump_duration())
            .with_position(self.estimate_position());

        // Set ISU code
        if let Some(code) = event.kind.isu_code() {
            event = event.with_isu_code(code);
        }

        Some(event)
    }

    fn lower_spin(&mut self, spin: &SpinElement) -> Option<Event> {
        let id = self.next_id();
        let positions: Vec<_> = spin.positions.iter().map(|p| p.position).collect();
        let level = spin.level.unwrap_or_default();

        // Detect flying entry from features
        let flying = spin.features.iter().any(|f| {
            matches!(
                f,
                anv_syntax::ast::SpinFeature::FlyingEntry | anv_syntax::ast::SpinFeature::JumpEntry
            )
        });

        // Detect change of foot from features or from position definitions
        let change_foot = spin
            .features
            .iter()
            .any(|f| matches!(f, anv_syntax::ast::SpinFeature::ChangeOfFoot))
            || spin.positions.iter().any(|p| p.change_foot);

        let kind = EventKind::Spin {
            positions,
            level,
            flying,
            change_foot,
        };

        let mut event = Event::new(id, kind, self.current_time)
            .with_duration(Duration::spin_duration())
            .with_position(self.estimate_position());

        if let Some(code) = event.kind.isu_code() {
            event = event.with_isu_code(code);
        }

        Some(event)
    }

    fn lower_step(&mut self, step: &StepSequence) -> Option<Event> {
        let id = self.next_id();
        let pattern = match step.pattern {
            anv_syntax::ast::StepPattern::Straight => StepPattern::Straight,
            anv_syntax::ast::StepPattern::Circular => StepPattern::Circular,
            anv_syntax::ast::StepPattern::Serpentine => StepPattern::Serpentine,
            anv_syntax::ast::StepPattern::Diagonal => StepPattern::Diagonal,
            anv_syntax::ast::StepPattern::Midline => StepPattern::Straight, // Map to straight
        };
        let level = step.level.unwrap_or_default();

        let kind = EventKind::StepSequence { pattern, level };

        let mut event = Event::new(id, kind, self.current_time)
            .with_duration(Duration::step_sequence_duration())
            .with_position(self.estimate_position());

        if let Some(code) = event.kind.isu_code() {
            event = event.with_isu_code(code);
        }

        Some(event)
    }

    fn lower_lift(&mut self, lift: &LiftElement) -> Option<Event> {
        let id = self.next_id();
        let level = lift.level.unwrap_or_default();

        let kind = EventKind::Lift {
            group: lift.group,
            level,
        };

        let mut event = Event::new(id, kind, self.current_time)
            .with_duration(Duration::lift_duration())
            .with_position(self.estimate_position());

        if let Some(code) = event.kind.isu_code() {
            event = event.with_isu_code(code);
        }

        Some(event)
    }

    fn lower_throw(&mut self, throw: &ThrowElement) -> Option<Event> {
        let id = self.next_id();
        let kind = EventKind::Throw {
            rotations: throw.rotations,
            kind: throw.kind,
        };

        let mut event = Event::new(id, kind, self.current_time)
            .with_duration(Duration::throw_duration())
            .with_position(self.estimate_position());

        if let Some(code) = event.kind.isu_code() {
            event = event.with_isu_code(code);
        }

        Some(event)
    }

    fn lower_twist(&mut self, twist: &TwistElement) -> Option<Event> {
        let id = self.next_id();
        let level = twist.level.unwrap_or_default();

        let kind = EventKind::Twist {
            rotations: twist.rotations,
            level,
        };

        let mut event = Event::new(id, kind, self.current_time)
            .with_duration(Duration::twist_duration())
            .with_position(self.estimate_position());

        if let Some(code) = event.kind.isu_code() {
            event = event.with_isu_code(code);
        }

        Some(event)
    }

    fn lower_death_spiral(&mut self, ds: &DeathSpiralElement) -> Option<Event> {
        let id = self.next_id();
        let level = ds.level.unwrap_or_default();

        let kind = EventKind::DeathSpiral {
            edge: ds.edge,
            level,
        };

        let mut event = Event::new(id, kind, self.current_time)
            .with_duration(Duration::death_spiral_duration())
            .with_position(self.estimate_position());

        if let Some(code) = event.kind.isu_code() {
            event = event.with_isu_code(code);
        }

        Some(event)
    }

    fn lower_choreographic(&mut self, choreo: &ChoreographicElement) -> Option<Event> {
        let id = self.next_id();
        let choreo_kind = match choreo.kind {
            anv_syntax::ast::ChoreographicKind::Spiral => ChoreographicKind::Spiral,
            anv_syntax::ast::ChoreographicKind::Spread => ChoreographicKind::SpreadEagle,
            anv_syntax::ast::ChoreographicKind::Ina => ChoreographicKind::InaBauer,
            anv_syntax::ast::ChoreographicKind::Hydroblading => ChoreographicKind::Hydroblading,
            anv_syntax::ast::ChoreographicKind::Pivot => ChoreographicKind::Pivot,
            anv_syntax::ast::ChoreographicKind::Choreographic => ChoreographicKind::Other,
        };

        let kind = EventKind::Choreographic { kind: choreo_kind };

        Some(
            Event::new(id, kind, self.current_time)
                .with_duration(Duration::choreographic_duration())
                .with_position(self.estimate_position()),
        )
    }

    fn lower_pattern(&mut self, pattern: &PatternElement) -> Option<Event> {
        let id = self.next_id();
        let kind = EventKind::PatternDance {
            name: pattern.name.clone(),
        };

        Some(
            Event::new(id, kind, self.current_time)
                .with_duration(Duration::from_secs(60.0)) // Pattern dances are ~1 min
                .with_position(self.estimate_position()),
        )
    }

    /// Estimate current position on ice based on time.
    /// This is a placeholder - real implementation would use path planning.
    fn estimate_position(&self) -> Position {
        // Simple circular pattern around the rink
        let t = self.current_time.as_secs();
        let angle = t * 0.1; // Slow rotation
        let radius = 15.0; // Middle of rink

        Position::new(radius * angle.cos(), radius * angle.sin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anv_core::source::FileId;
    use anv_syntax::parse;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_lower_simple_program() {
        let source = r#"
            program test {
                segment sp: short {
                    sequence elements {
                        jump triple axel
                        spin camel L3
                        step circular L4
                    }
                }
            }
        "#;

        let ast = parse(source, FileId(0)).expect("parse failed");
        let timeline = lower(&ast);

        assert_eq!(timeline.name, "test");
        // ProgramStart + 3 elements + ProgramEnd = 5 events
        assert_eq!(timeline.events.len(), 5);
        assert_eq!(timeline.segments.len(), 1);
        assert_eq!(timeline.segments[0].name, "sp");
    }

    #[test]
    fn test_lower_pairs_elements() {
        let source = r#"
            program pairs_test {
                segment sp: short {
                    sequence elements {
                        lift Gr3 L4
                        throw triple salchow
                        twist double L3
                        death_spiral LBI L4
                    }
                }
            }
        "#;

        let ast = parse(source, FileId(0)).expect("parse failed");
        let timeline = lower(&ast);

        // ProgramStart + 4 elements + ProgramEnd = 6 events
        assert_eq!(timeline.events.len(), 6);

        // Check ISU codes
        let lift = &timeline.events[1];
        assert!(lift.isu_code.as_ref().unwrap().contains("Li"));

        let throw = &timeline.events[2];
        assert!(throw.isu_code.as_ref().unwrap().contains("Th"));
    }

    #[test]
    fn test_timeline_json_export() {
        let source = r#"
            program export_test {
                segment sp: short {
                    sequence elements {
                        jump quad lutz
                    }
                }
            }
        "#;

        let ast = parse(source, FileId(0)).expect("parse failed");
        let timeline = lower(&ast);

        let json = timeline.to_json().expect("JSON export failed");
        assert!(json.contains("export_test"));
        assert!(json.contains("4Lz"));
    }
}
