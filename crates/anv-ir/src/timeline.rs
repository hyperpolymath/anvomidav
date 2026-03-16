// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Timeline representation for skating programs.
//!
//! The timeline is a flat, ordered sequence of events that represents
//! everything happening in a skating program.

use crate::rink::{Heading, Position};
use crate::types::{Duration, TimeCode};
use anv_core::skating::{Edge, JumpKind, Level, LiftGroup, Rotations, SpinPosition};
use serde::{Deserialize, Serialize};

/// A complete program timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// Program name.
    pub name: String,
    /// Total duration.
    pub duration: Duration,
    /// All events in chronological order.
    pub events: Vec<Event>,
    /// Segment boundaries.
    pub segments: Vec<SegmentMarker>,
}

impl Timeline {
    /// Create a new empty timeline.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            duration: Duration::ZERO,
            events: Vec::new(),
            segments: Vec::new(),
        }
    }

    /// Add an event to the timeline.
    pub fn push_event(&mut self, event: Event) {
        // Update total duration if this event extends past current end
        let event_end = event.start + event.duration;
        if event_end.as_secs() > self.duration.as_secs() {
            self.duration = Duration::from_secs(event_end.as_secs());
        }
        self.events.push(event);
    }

    /// Add a segment marker.
    pub fn push_segment(&mut self, marker: SegmentMarker) {
        self.segments.push(marker);
    }

    /// Get events within a time range.
    pub fn events_in_range(&self, start: TimeCode, end: TimeCode) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.start.as_secs() >= start.as_secs() && e.start.as_secs() < end.as_secs())
            .collect()
    }

    /// Get all events of a specific kind.
    pub fn events_of_kind(&self, kind: EventKindFilter) -> Vec<&Event> {
        self.events.iter().filter(|e| kind.matches(&e.kind)).collect()
    }

    /// Get the event at a specific time (if any).
    pub fn event_at_time(&self, time: TimeCode) -> Option<&Event> {
        self.events.iter().find(|e| {
            let start = e.start.as_secs();
            let end = start + e.duration.as_secs();
            time.as_secs() >= start && time.as_secs() < end
        })
    }

    /// Calculate total element count.
    pub fn element_count(&self) -> usize {
        self.events.iter().filter(|e| e.kind.is_element()).count()
    }

    /// Export to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// A segment boundary marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentMarker {
    /// Segment name.
    pub name: String,
    /// Segment kind (e.g., "short", "free", "rhythm", "exhibition").
    pub kind: String,
    /// Start time.
    pub start: TimeCode,
    /// End time.
    pub end: TimeCode,
}

/// A single event in the timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event ID.
    pub id: u32,
    /// Event kind.
    pub kind: EventKind,
    /// Start time.
    pub start: TimeCode,
    /// Duration.
    pub duration: Duration,
    /// Starting position on ice.
    pub position: Position,
    /// Ending position on ice (for transitions).
    pub end_position: Option<Position>,
    /// Starting heading.
    pub heading: Heading,
    /// Skater ID (for pairs/dance).
    pub skater: Option<String>,
    /// ISU element code (e.g., "3A", "CCoSp4").
    pub isu_code: Option<String>,
    /// User-defined label.
    pub label: Option<String>,
}

impl Event {
    /// Create a new event.
    pub fn new(id: u32, kind: EventKind, start: TimeCode) -> Self {
        Self {
            id,
            kind,
            start,
            duration: Duration::ZERO,
            position: Position::CENTER,
            end_position: None,
            heading: Heading::default(),
            skater: None,
            isu_code: None,
            label: None,
        }
    }

    /// Builder: set duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Builder: set position.
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Builder: set skater.
    pub fn with_skater(mut self, skater: impl Into<String>) -> Self {
        self.skater = Some(skater.into());
        self
    }

    /// Builder: set ISU code.
    pub fn with_isu_code(mut self, code: impl Into<String>) -> Self {
        self.isu_code = Some(code.into());
        self
    }

    /// Builder: set label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Get end time.
    pub fn end_time(&self) -> TimeCode {
        self.start + self.duration
    }
}

/// Event type classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    // === Jumps ===
    Jump {
        rotations: Rotations,
        kind: JumpKind,
    },
    JumpCombination {
        jumps: Vec<(Rotations, JumpKind)>,
    },

    // === Spins ===
    Spin {
        positions: Vec<SpinPosition>,
        level: Level,
        flying: bool,
        change_foot: bool,
    },

    // === Steps ===
    StepSequence {
        pattern: StepPattern,
        level: Level,
    },
    ChoreographicSequence,

    // === Pairs ===
    Lift {
        group: LiftGroup,
        level: Level,
    },
    Throw {
        rotations: Rotations,
        kind: JumpKind,
    },
    Twist {
        rotations: Rotations,
        level: Level,
    },
    DeathSpiral {
        edge: Edge,
        level: Level,
    },

    // === Ice Dance ===
    PatternDance {
        name: String,
    },
    Twizzle {
        level: Level,
    },

    // === Choreographic ===
    Choreographic {
        kind: ChoreographicKind,
    },

    // === Structural ===
    Transition,
    Pause,
    ProgramStart,
    ProgramEnd,
}

impl EventKind {
    /// Is this a scoring element?
    pub fn is_element(&self) -> bool {
        !matches!(
            self,
            EventKind::Transition | EventKind::Pause | EventKind::ProgramStart | EventKind::ProgramEnd
        )
    }

    /// Get the standard ISU code for this element.
    pub fn isu_code(&self) -> Option<String> {
        match self {
            EventKind::Jump { rotations, kind } => {
                Some(format!("{}{}", rotations, kind))
            }
            EventKind::Spin { positions, level, flying, change_foot } => {
                let prefix = match (flying, change_foot, positions.len() > 1) {
                    (true, _, true) => "FC",
                    (true, _, false) => "F",
                    (_, true, true) => "CC",
                    (_, true, false) => "C",
                    (_, _, true) => "C",
                    _ => "",
                };
                let pos = positions.first().map(|p| p.abbrev()).unwrap_or("Sp");
                Some(format!("{}{}o{}", prefix, pos, level))
            }
            EventKind::StepSequence { level, .. } => {
                Some(format!("StSq{}", level))
            }
            EventKind::Lift { group, level } => {
                Some(format!("{}Li{}", group, level))
            }
            EventKind::Throw { rotations, kind } => {
                Some(format!("{}{}Th", rotations, kind))
            }
            EventKind::Twist { rotations, level } => {
                Some(format!("{}Tw{}", rotations, level))
            }
            EventKind::DeathSpiral { edge, level } => {
                let code = match edge {
                    Edge::LBI => "BDs",
                    Edge::LBO => "FDs",
                    _ => "Ds",
                };
                Some(format!("{}{}", code, level))
            }
            _ => None,
        }
    }
}

/// Step sequence pattern.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepPattern {
    #[default]
    Straight,
    Circular,
    Serpentine,
    Diagonal,
}

/// Choreographic element kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoreographicKind {
    Spiral,
    SpreadEagle,
    InaBauer,
    Hydroblading,
    Pivot,
    Other,
}

/// Filter for querying events by kind.
pub enum EventKindFilter {
    Jumps,
    Spins,
    Steps,
    Lifts,
    AllElements,
}

impl EventKindFilter {
    fn matches(&self, kind: &EventKind) -> bool {
        match self {
            EventKindFilter::Jumps => matches!(kind, EventKind::Jump { .. } | EventKind::JumpCombination { .. }),
            EventKindFilter::Spins => matches!(kind, EventKind::Spin { .. }),
            EventKindFilter::Steps => matches!(kind, EventKind::StepSequence { .. }),
            EventKindFilter::Lifts => matches!(kind, EventKind::Lift { .. }),
            EventKindFilter::AllElements => kind.is_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_timeline_push_event() {
        let mut timeline = Timeline::new("test");

        let event = Event::new(1, EventKind::Jump {
            rotations: Rotations::Triple,
            kind: JumpKind::Axel,
        }, TimeCode::from_secs(10.0))
            .with_duration(Duration::from_secs(3.0));

        timeline.push_event(event);

        assert_eq!(timeline.events.len(), 1);
        assert_eq!(timeline.duration.as_secs(), 13.0);
    }

    #[test]
    fn test_event_isu_code() {
        let jump = EventKind::Jump {
            rotations: Rotations::Triple,
            kind: JumpKind::Axel,
        };
        assert_eq!(jump.isu_code(), Some("3A".to_string()));

        let spin = EventKind::Spin {
            positions: vec![SpinPosition::Camel],
            level: Level::L4,
            flying: true,
            change_foot: false,
        };
        assert!(spin.isu_code().is_some());
    }

    #[test]
    fn test_events_in_range() {
        let mut timeline = Timeline::new("test");

        timeline.push_event(Event::new(1, EventKind::ProgramStart, TimeCode::from_secs(0.0)));
        timeline.push_event(Event::new(2, EventKind::Jump {
            rotations: Rotations::Triple,
            kind: JumpKind::Lutz,
        }, TimeCode::from_secs(10.0)));
        timeline.push_event(Event::new(3, EventKind::Jump {
            rotations: Rotations::Triple,
            kind: JumpKind::Flip,
        }, TimeCode::from_secs(30.0)));

        let range = timeline.events_in_range(TimeCode::from_secs(5.0), TimeCode::from_secs(25.0));
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].id, 2);
    }
}
