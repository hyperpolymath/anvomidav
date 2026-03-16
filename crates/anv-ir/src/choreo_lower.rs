// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Choreography lowering: Timeline -> Choreography.
//!
//! This module transforms the flat [`Timeline`] into a structured
//! [`Choreography`], enriching each event with spatial paths, element
//! decomposition into phases, and connecting transitions.
//!
//! # Pipeline
//!
//! ```text
//! AST  --[lower]--> Timeline  --[lower_to_choreo]--> Choreography
//! ```
//!
//! The choreography lowering pass:
//! 1. Creates a `Choreography` with metadata extracted from the timeline
//! 2. Groups events into `ChoreoSegment`s
//! 3. Converts each scoring event into a `PlacedElement` with default phases
//! 4. Generates `Transition` paths between consecutive elements
//! 5. Optionally aligns elements to a music map

use crate::choreo::{
    ChoreoSegment, Choreography, Discipline, ElementId, ElementPlacement, MusicMap,
    PlacedElement, ProgramMetadata, Transition, TransitionId, TransitionQuality,
};
use crate::path::{step_sequence_path, IcePath};
use crate::rink::Position;
use crate::timeline::{EventKind, Timeline};
use crate::types::{Duration, TimeCode};

/// Lower a [`Timeline`] into a [`Choreography`].
///
/// This is the main entry point for the choreography lowering pass.
pub fn lower_to_choreo(timeline: &Timeline) -> Choreography {
    let mut builder = ChoreoBuilder::new(timeline);
    builder.build();
    builder.choreography
}

/// Lower a [`Timeline`] into a [`Choreography`] with a music map.
pub fn lower_to_choreo_with_music(timeline: &Timeline, music: MusicMap) -> Choreography {
    let mut choreo = lower_to_choreo(timeline);
    choreo.music_map = Some(music);
    choreo
}

/// Internal builder for constructing a `Choreography` from a `Timeline`.
struct ChoreoBuilder<'a> {
    timeline: &'a Timeline,
    choreography: Choreography,
    next_element_id: u32,
    next_transition_id: u32,
}

impl<'a> ChoreoBuilder<'a> {
    fn new(timeline: &'a Timeline) -> Self {
        let metadata = ProgramMetadata {
            name: timeline.name.clone(),
            discipline: infer_discipline(timeline),
            segment_type: None,
            skaters: Vec::new(),
            season: None,
        };

        Self {
            timeline,
            choreography: Choreography::new(metadata),
            next_element_id: 1,
            next_transition_id: 1,
        }
    }

    fn next_element_id(&mut self) -> ElementId {
        let id = ElementId(self.next_element_id);
        self.next_element_id += 1;
        id
    }

    fn next_transition_id(&mut self) -> TransitionId {
        let id = TransitionId(self.next_transition_id);
        self.next_transition_id += 1;
        id
    }

    fn build(&mut self) {
        // If the timeline has segments, use them; otherwise create a single segment
        if self.timeline.segments.is_empty() {
            let mut segment = ChoreoSegment::new(
                &self.timeline.name,
                "unknown",
                TimeCode::ZERO,
            );
            self.populate_segment(&mut segment, TimeCode::ZERO, self.timeline.duration);
            segment.end = self.timeline.duration.as_time_code();
            self.choreography.segments.push(segment);
        } else {
            for seg_marker in &self.timeline.segments {
                let mut segment = ChoreoSegment::new(
                    &seg_marker.name,
                    &seg_marker.kind,
                    seg_marker.start,
                );
                self.populate_segment(&mut segment, seg_marker.start, Duration::from_secs(
                    seg_marker.end.as_secs() - seg_marker.start.as_secs(),
                ));
                segment.end = seg_marker.end;
                self.choreography.segments.push(segment);
            }
        }

        self.choreography.total_duration = self.timeline.duration;
    }

    fn populate_segment(
        &mut self,
        segment: &mut ChoreoSegment,
        seg_start: TimeCode,
        seg_duration: Duration,
    ) {
        let seg_end_secs = seg_start.as_secs() + seg_duration.as_secs();

        // Collect scoring events within this segment's time range
        let scoring_events: Vec<_> = self
            .timeline
            .events
            .iter()
            .filter(|e| {
                e.kind.is_element()
                    && e.start.as_secs() >= seg_start.as_secs()
                    && e.start.as_secs() < seg_end_secs
            })
            .collect();

        // Convert each scoring event to a PlacedElement
        let mut placed_elements: Vec<PlacedElement> = Vec::new();
        for event in &scoring_events {
            let elem_id = self.next_element_id();

            let placement = ElementPlacement::new(
                event.start,
                event.duration,
                event.position,
            )
            .with_heading(event.heading);

            let mut placed = PlacedElement::new(elem_id, event.kind.clone(), placement);

            if let Some(code) = &event.isu_code {
                placed = placed.with_isu_code(code);
            }
            if let Some(label) = &event.label {
                placed = placed.with_label(label);
            }

            // Generate default phases
            placed.generate_default_phases();

            // For step sequences, enhance the placement with a proper path
            if let EventKind::StepSequence { pattern, .. } = &event.kind {
                let path = step_sequence_path(pattern, event.position, event.heading);
                if let Some(end_pos) = path.end_position() {
                    placed.placement.end_position = Some(end_pos);
                }
            }

            placed_elements.push(placed);
        }

        // Generate transitions between consecutive elements
        let mut transitions: Vec<Transition> = Vec::new();

        // Transition from segment start to first element
        if let Some(first) = placed_elements.first() {
            let opening_pos = segment_opening_position(&segment.kind);
            if opening_pos.distance_to(&first.placement.position) > 0.1 {
                let trans_id = self.next_transition_id();
                let path = IcePath::straight(opening_pos, first.placement.position);
                let trans_duration = Duration::from_secs(
                    (first.placement.start.as_secs() - seg_start.as_secs()).max(1.0),
                );

                let transition = Transition::new(
                    trans_id,
                    first.id,
                    seg_start,
                    trans_duration,
                    path,
                )
                .with_quality(TransitionQuality::Stroking);

                transitions.push(transition);
            }
        }

        // Transitions between consecutive elements
        for pair in placed_elements.windows(2) {
            let from_elem = &pair[0];
            let to_elem = &pair[1];

            let from_end = from_elem.end_time();
            let gap = to_elem.placement.start.as_secs() - from_end.as_secs();

            // Only create transition if there's a gap
            if gap > 0.01 {
                let trans_id = self.next_transition_id();
                let from_pos = from_elem.end_position();
                let to_pos = to_elem.placement.position;

                let path = generate_transition_path(from_pos, to_pos, gap);
                let quality = classify_transition_quality(gap);

                let transition = Transition::new(
                    trans_id,
                    to_elem.id,
                    from_end,
                    Duration::from_secs(gap),
                    path,
                )
                .with_from(from_elem.id)
                .with_quality(quality)
                .with_connecting_steps(gap > 3.0);

                transitions.push(transition);
            }
        }

        segment.elements = placed_elements;
        segment.transitions = transitions;
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Infer the skating discipline from the timeline content.
fn infer_discipline(timeline: &Timeline) -> Discipline {
    let has_lifts = timeline.events.iter().any(|e| matches!(&e.kind, EventKind::Lift { .. }));
    let has_throws = timeline.events.iter().any(|e| matches!(&e.kind, EventKind::Throw { .. }));
    let has_twists = timeline.events.iter().any(|e| matches!(&e.kind, EventKind::Twist { .. }));
    let has_patterns = timeline.events.iter().any(|e| matches!(&e.kind, EventKind::PatternDance { .. }));
    let has_twizzles = timeline.events.iter().any(|e| matches!(&e.kind, EventKind::Twizzle { .. }));

    if has_patterns || has_twizzles {
        Discipline::IceDance
    } else if has_lifts || has_throws || has_twists {
        Discipline::Pairs
    } else {
        // Default to men's singles; could be enhanced with more heuristics
        Discipline::MenSingles
    }
}

/// Determine the opening position based on segment kind.
fn segment_opening_position(kind: &str) -> Position {
    match kind {
        "short" => Position::center_ice(),
        "free" => Position::center_ice(),
        "rhythm" | "pattern" => Position::far_end(),
        _ => Position::center_ice(),
    }
}

/// Generate a transition path between two positions.
fn generate_transition_path(from: Position, to: Position, duration: f64) -> IcePath {
    let distance = from.distance_to(&to);

    if distance < 5.0 || duration < 2.0 {
        // Short transition: straight line
        IcePath::straight(from, to)
    } else {
        // Longer transition: gentle curve (Bezier-like via arc)
        // Use a slight arc to make it more natural
        let mid_x = (from.x() + to.x()) / 2.0;
        let mid_y = (from.y() + to.y()) / 2.0;
        // Offset the midpoint perpendicular to the line
        let dx = to.x() - from.x();
        let dy = to.y() - from.y();
        let offset = distance * 0.15; // 15% of distance as offset
        let perp_x = -dy / distance * offset;
        let perp_y = dx / distance * offset;

        let mid = Position::new(mid_x + perp_x, mid_y + perp_y);

        let mut path = IcePath::new();
        path.push(crate::path::Waypoint::new(from));
        path.push(crate::path::Waypoint::new(mid));
        path.push(crate::path::Waypoint::new(to));
        path
    }
}

/// Classify the quality of a transition based on its duration.
fn classify_transition_quality(gap_seconds: f64) -> TransitionQuality {
    if gap_seconds < 2.0 {
        TransitionQuality::Stroking
    } else if gap_seconds < 5.0 {
        TransitionQuality::EdgeWork
    } else if gap_seconds < 10.0 {
        TransitionQuality::Footwork
    } else {
        TransitionQuality::Choreographic
    }
}

/// Helper: convert Duration to end TimeCode.
trait DurationExt {
    fn as_time_code(&self) -> TimeCode;
}

impl DurationExt for Duration {
    fn as_time_code(&self) -> TimeCode {
        TimeCode::from_secs(self.as_secs())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower::lower;
    use anv_core::source::FileId;
    use anv_syntax::parse;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_lower_to_choreo_simple() {
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
        let choreo = lower_to_choreo(&timeline);

        assert_eq!(choreo.metadata.name, "test");
        assert_eq!(choreo.segments.len(), 1);
        assert_eq!(choreo.element_count(), 3);

        // Each element should have phases
        for elem in &choreo.segments[0].elements {
            assert!(
                !elem.phases.is_empty(),
                "Element {:?} should have phases",
                elem.element
            );
        }
    }

    #[test]
    fn test_lower_to_choreo_pairs() {
        let source = r#"
            program pairs_test {
                segment sp: short {
                    sequence elements {
                        lift Gr3 L4
                        throw triple salchow
                        death_spiral LBI L4
                    }
                }
            }
        "#;

        let ast = parse(source, FileId(0)).expect("parse failed");
        let timeline = lower(&ast);
        let choreo = lower_to_choreo(&timeline);

        assert_eq!(choreo.metadata.discipline, Discipline::Pairs);
        assert_eq!(choreo.element_count(), 3);

        // Transitions should be generated
        assert!(!choreo.segments[0].transitions.is_empty());
    }

    #[test]
    fn test_transition_generation() {
        let source = r#"
            program trans_test {
                segment sp: short {
                    sequence elements {
                        jump quad lutz
                        jump triple toe_loop
                        spin camel sit upright L4
                    }
                }
            }
        "#;

        let ast = parse(source, FileId(0)).expect("parse failed");
        let timeline = lower(&ast);
        let choreo = lower_to_choreo(&timeline);

        // Should have transitions between elements
        let transitions = &choreo.segments[0].transitions;
        assert!(
            !transitions.is_empty(),
            "Expected transitions between elements"
        );

        // Each transition should have a non-empty path
        for trans in transitions {
            assert!(!trans.path.is_empty(), "Transition path should not be empty");
            assert!(trans.path.total_distance() >= 0.0);
        }
    }

    #[test]
    fn test_choreo_total_distance() {
        let source = r#"
            program dist_test {
                segment sp: short {
                    sequence elements {
                        jump triple axel
                        step circular L4
                        spin camel L4
                    }
                }
            }
        "#;

        let ast = parse(source, FileId(0)).expect("parse failed");
        let timeline = lower(&ast);
        let choreo = lower_to_choreo(&timeline);

        let total_dist = choreo.total_distance();
        assert!(
            total_dist > 0.0,
            "Total distance should be positive, got {}",
            total_dist
        );
    }

    #[test]
    fn test_discipline_inference_singles() {
        let source = r#"
            program singles_test {
                segment sp: short {
                    sequence elements {
                        jump triple axel
                        spin camel L4
                    }
                }
            }
        "#;

        let ast = parse(source, FileId(0)).expect("parse failed");
        let timeline = lower(&ast);
        let choreo = lower_to_choreo(&timeline);

        assert_eq!(choreo.metadata.discipline, Discipline::MenSingles);
    }

    #[test]
    fn test_music_map_integration() {
        let source = r#"
            program music_test {
                segment sp: short {
                    sequence elements {
                        jump triple axel
                    }
                }
            }
        "#;

        let ast = parse(source, FileId(0)).expect("parse failed");
        let timeline = lower(&ast);
        let music = MusicMap::new(130.0, 4);
        let choreo = lower_to_choreo_with_music(&timeline, music);

        assert!(choreo.music_map.is_some());
        let map = choreo.music_map.as_ref().unwrap();
        assert_eq!(map.bpm, 130.0);
        assert_eq!(map.beats_per_measure, 4);
    }

    #[test]
    fn test_element_phases_are_sequential() {
        let source = r#"
            program phase_test {
                segment sp: short {
                    sequence elements {
                        jump quad flip
                    }
                }
            }
        "#;

        let ast = parse(source, FileId(0)).expect("parse failed");
        let timeline = lower(&ast);
        let choreo = lower_to_choreo(&timeline);

        let elem = &choreo.segments[0].elements[0];
        // Verify phases are in chronological order
        for pair in elem.phases.windows(2) {
            let end_prev = pair[0].start.as_secs() + pair[0].duration.as_secs();
            let start_next = pair[1].start.as_secs();
            assert!(
                (end_prev - start_next).abs() < 0.001,
                "Phases should be sequential: prev ends at {}, next starts at {}",
                end_prev,
                start_next
            );
        }
    }
}
