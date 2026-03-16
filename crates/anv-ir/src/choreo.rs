// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Choreography IR: high-level representation of a complete skating program.
//!
//! This module provides the [`Choreography`] type, which is the primary IR node
//! for a fully lowered skating program. It bridges the gap between the flat
//! [`Timeline`](crate::timeline::Timeline) (a sequence of events) and the
//! structured representation needed for visualization, analysis, and export.
//!
//! # Design
//!
//! A `Choreography` groups events into [`PlacedElement`]s that carry spatial
//! and temporal context, linked by [`Transition`]s that describe movement
//! between elements. Each placed element can be decomposed into phases
//! (entry, execution, exit) via [`ElementPhase`].
//!
//! ```text
//! Choreography
//!   ├── metadata (program info, discipline, rink)
//!   ├── segments[]
//!   │     ├── placed_elements[]
//!   │     │     ├── element (EventKind from timeline)
//!   │     │     ├── phases[] (entry → execution → exit)
//!   │     │     ├── spatial placement (position, heading, path)
//!   │     │     └── temporal placement (start, duration, beat)
//!   │     └── transitions[] (between elements)
//!   │           ├── path (waypoints on rink)
//!   │           ├── movement quality
//!   │           └── edge work
//!   └── music_map (beat ↔ time mapping)
//! ```

use crate::path::IcePath;
use crate::rink::{Heading, Position, RinkDimensions};
use crate::timeline::EventKind;
use crate::types::{Duration, TimeCode};
use anv_core::skating::Edge;
use serde::{Deserialize, Serialize};

/// Unique identifier for a placed element within a choreography.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId(pub u32);

/// Unique identifier for a transition within a choreography.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransitionId(pub u32);

// =============================================================================
// Choreography (top-level IR node)
// =============================================================================

/// A fully lowered choreography: the primary IR for a skating program.
///
/// This is the output of the choreography lowering pass, which enriches the
/// flat [`Timeline`](crate::timeline::Timeline) with spatial paths, element
/// decomposition, and transition details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choreography {
    /// Program metadata.
    pub metadata: ProgramMetadata,

    /// Rink geometry for this program.
    pub rink: RinkDimensions,

    /// Choreography segments (short program, free skate, etc.).
    pub segments: Vec<ChoreoSegment>,

    /// Optional music synchronization map.
    pub music_map: Option<MusicMap>,

    /// Total program duration.
    pub total_duration: Duration,
}

impl Choreography {
    /// Create a new choreography with the given metadata.
    pub fn new(metadata: ProgramMetadata) -> Self {
        Self {
            metadata,
            rink: RinkDimensions::OLYMPIC,
            segments: Vec::new(),
            music_map: None,
            total_duration: Duration::ZERO,
        }
    }

    /// Count all placed elements across all segments.
    pub fn element_count(&self) -> usize {
        self.segments.iter().map(|s| s.elements.len()).sum()
    }

    /// Count all transitions across all segments.
    pub fn transition_count(&self) -> usize {
        self.segments.iter().map(|s| s.transitions.len()).sum()
    }

    /// Get a placed element by its ID.
    pub fn element_by_id(&self, id: ElementId) -> Option<&PlacedElement> {
        self.segments
            .iter()
            .flat_map(|s| s.elements.iter())
            .find(|e| e.id == id)
    }

    /// Get all elements in chronological order across all segments.
    pub fn all_elements(&self) -> Vec<&PlacedElement> {
        let mut elements: Vec<_> = self
            .segments
            .iter()
            .flat_map(|s| s.elements.iter())
            .collect();
        elements.sort_by(|a, b| {
            a.placement
                .start
                .as_secs()
                .partial_cmp(&b.placement.start.as_secs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        elements
    }

    /// Get all transitions in chronological order across all segments.
    pub fn all_transitions(&self) -> Vec<&Transition> {
        let mut transitions: Vec<_> = self
            .segments
            .iter()
            .flat_map(|s| s.transitions.iter())
            .collect();
        transitions.sort_by(|a, b| {
            a.start
                .as_secs()
                .partial_cmp(&b.start.as_secs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        transitions
    }

    /// Compute the total ice coverage distance (sum of all transition paths).
    pub fn total_distance(&self) -> f64 {
        self.segments
            .iter()
            .flat_map(|s| s.transitions.iter())
            .map(|t| t.path.total_distance())
            .sum()
    }

    /// Export the choreography to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

// =============================================================================
// Program Metadata
// =============================================================================

/// Metadata describing a skating program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramMetadata {
    /// Program name.
    pub name: String,

    /// Skating discipline.
    pub discipline: Discipline,

    /// Segment type (short, free, etc.).
    pub segment_type: Option<String>,

    /// Skater name(s).
    pub skaters: Vec<String>,

    /// Season/competition context.
    pub season: Option<String>,
}

/// Skating discipline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Discipline {
    /// Men's singles.
    MenSingles,
    /// Women's singles.
    WomenSingles,
    /// Pairs.
    Pairs,
    /// Ice dance.
    IceDance,
}

// =============================================================================
// Choreography Segment
// =============================================================================

/// A segment within a choreography (e.g., short program, free skate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoreoSegment {
    /// Segment name.
    pub name: String,

    /// Segment kind (short, free, rhythm, exhibition).
    pub kind: String,

    /// Placed elements within this segment.
    pub elements: Vec<PlacedElement>,

    /// Transitions between elements.
    pub transitions: Vec<Transition>,

    /// Start time of this segment.
    pub start: TimeCode,

    /// End time of this segment.
    pub end: TimeCode,
}

impl ChoreoSegment {
    /// Create a new empty segment.
    pub fn new(name: impl Into<String>, kind: impl Into<String>, start: TimeCode) -> Self {
        Self {
            name: name.into(),
            kind: kind.into(),
            elements: Vec::new(),
            transitions: Vec::new(),
            start,
            end: start,
        }
    }

    /// Get the segment duration.
    pub fn duration(&self) -> Duration {
        self.end - self.start
    }

    /// Get the element that follows a given element in this segment.
    pub fn next_element(&self, id: ElementId) -> Option<&PlacedElement> {
        let idx = self.elements.iter().position(|e| e.id == id)?;
        self.elements.get(idx + 1)
    }

    /// Get the transition leading into a given element.
    pub fn transition_into(&self, element_id: ElementId) -> Option<&Transition> {
        self.transitions.iter().find(|t| t.to_element == element_id)
    }

    /// Get the transition leaving a given element.
    pub fn transition_from(&self, element_id: ElementId) -> Option<&Transition> {
        self.transitions
            .iter()
            .find(|t| t.from_element == Some(element_id))
    }
}

// =============================================================================
// Placed Element
// =============================================================================

/// A technical element placed in time and space on the rink.
///
/// This is the core IR node for a skating element. It combines the abstract
/// element (what kind of jump/spin/step) with concrete spatial and temporal
/// placement (where and when on the rink).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedElement {
    /// Unique element ID.
    pub id: ElementId,

    /// The technical element kind (from timeline IR).
    pub element: EventKind,

    /// ISU element code (e.g., "3A", "CCoSp4", "StSq4").
    pub isu_code: Option<String>,

    /// User-defined label.
    pub label: Option<String>,

    /// Spatial and temporal placement.
    pub placement: ElementPlacement,

    /// Decomposition into execution phases.
    pub phases: Vec<ElementPhase>,

    /// Which skater performs this (for pairs/dance; None for singles).
    pub skater: Option<SkaterRef>,
}

impl PlacedElement {
    /// Create a new placed element.
    pub fn new(id: ElementId, element: EventKind, placement: ElementPlacement) -> Self {
        Self {
            id,
            element,
            isu_code: None,
            label: None,
            placement,
            phases: Vec::new(),
            skater: None,
        }
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

    /// Builder: set phases.
    pub fn with_phases(mut self, phases: Vec<ElementPhase>) -> Self {
        self.phases = phases;
        self
    }

    /// Builder: set skater.
    pub fn with_skater(mut self, skater: SkaterRef) -> Self {
        self.skater = Some(skater);
        self
    }

    /// Get end time.
    pub fn end_time(&self) -> TimeCode {
        self.placement.start + self.placement.duration
    }

    /// Get end position (exit point of the last phase, or the main position).
    pub fn end_position(&self) -> Position {
        self.phases
            .last()
            .map(|p| p.end_position)
            .unwrap_or(self.placement.position)
    }

    /// Get end heading.
    pub fn end_heading(&self) -> Heading {
        self.phases
            .last()
            .map(|p| p.end_heading)
            .unwrap_or(self.placement.heading)
    }

    /// Generate default phases for this element kind.
    pub fn generate_default_phases(&mut self) {
        self.phases = default_phases_for(&self.element, &self.placement);
    }
}

// =============================================================================
// Element Placement
// =============================================================================

/// Spatial and temporal placement of an element on the rink.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPlacement {
    /// Start time.
    pub start: TimeCode,

    /// Total element duration.
    pub duration: Duration,

    /// Center position on ice.
    pub position: Position,

    /// Starting heading/facing direction.
    pub heading: Heading,

    /// Optional ending position (for elements with travel, e.g. step sequences).
    pub end_position: Option<Position>,

    /// Optional ending heading.
    pub end_heading: Option<Heading>,

    /// Music beat alignment (if synced to music).
    pub beat: Option<u32>,
}

impl ElementPlacement {
    /// Create a new placement.
    pub fn new(start: TimeCode, duration: Duration, position: Position) -> Self {
        Self {
            start,
            duration,
            position,
            heading: Heading::default(),
            end_position: None,
            end_heading: None,
            beat: None,
        }
    }

    /// Builder: set heading.
    pub fn with_heading(mut self, heading: Heading) -> Self {
        self.heading = heading;
        self
    }

    /// Builder: set end position.
    pub fn with_end_position(mut self, pos: Position) -> Self {
        self.end_position = Some(pos);
        self
    }

    /// Builder: set beat alignment.
    pub fn with_beat(mut self, beat: u32) -> Self {
        self.beat = Some(beat);
        self
    }

    /// Get the end time.
    pub fn end_time(&self) -> TimeCode {
        self.start + self.duration
    }
}

// =============================================================================
// Element Phase (technical decomposition)
// =============================================================================

/// A phase of element execution.
///
/// Technical elements are decomposed into sequential phases:
/// - **Entry**: approach, including edges and steps leading in
/// - **Execution**: the element itself (airborne for jumps, rotating for spins)
/// - **Exit**: landing, exit edges, and recovery
///
/// For jumps: entry (approach + takeoff) -> execution (airborne) -> exit (landing)
/// For spins: entry (wind-up) -> execution (spinning) -> exit (check-out)
/// For steps: the step sequence is a single extended execution phase, but can
///            be decomposed into sub-phases for each pattern section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPhase {
    /// Phase kind.
    pub kind: PhaseKind,

    /// Phase start time.
    pub start: TimeCode,

    /// Phase duration.
    pub duration: Duration,

    /// Starting position.
    pub start_position: Position,

    /// Ending position.
    pub end_position: Position,

    /// Starting heading.
    pub start_heading: Heading,

    /// Ending heading.
    pub end_heading: Heading,

    /// Entry/exit edge (if applicable).
    pub edge: Option<Edge>,

    /// Phase-specific data.
    pub data: PhaseData,
}

/// Kind of execution phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseKind {
    /// Approach and preparation.
    Entry,
    /// Core element execution.
    Execution,
    /// Landing/exit and recovery.
    Exit,
    /// Position change within a spin.
    PositionChange,
    /// Foot change within a spin.
    FootChange,
}

/// Phase-specific data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseData {
    /// Jump entry: approach speed and takeoff edge.
    JumpEntry {
        /// Approach speed in m/s.
        approach_speed: f64,
        /// Takeoff edge.
        takeoff_edge: Edge,
    },
    /// Jump airborne: peak height and rotation.
    JumpAirborne {
        /// Peak height in meters.
        peak_height: f64,
        /// Total rotation in degrees.
        rotation_degrees: f64,
    },
    /// Jump landing.
    JumpLanding {
        /// Landing edge.
        landing_edge: Edge,
        /// Flow out distance in meters.
        flow_out: f64,
    },
    /// Spin rotation.
    SpinRotation {
        /// Revolutions in this phase.
        revolutions: f64,
        /// Spin position.
        position: String,
    },
    /// Step sequence section.
    StepSection {
        /// Pattern direction in this section.
        direction: String,
        /// Edges used.
        edges: Vec<Edge>,
    },
    /// Lift hold.
    LiftHold {
        /// Hold position description.
        hold_position: String,
        /// Height above ice in meters.
        height: f64,
    },
    /// Generic / no specific data.
    Generic,
}

// =============================================================================
// Transition (between elements)
// =============================================================================

/// A transition connecting two elements on the rink.
///
/// Transitions represent the skating between technical elements: the connecting
/// steps, edges, and movements that carry the skater from one element to the
/// next. Transition quality is a major component of the program component score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Unique transition ID.
    pub id: TransitionId,

    /// Element this transition starts from (None for program opening).
    pub from_element: Option<ElementId>,

    /// Element this transition leads into.
    pub to_element: ElementId,

    /// Start time.
    pub start: TimeCode,

    /// Duration.
    pub duration: Duration,

    /// Path on the ice surface.
    pub path: IcePath,

    /// Movement quality classification.
    pub quality: TransitionQuality,

    /// Edge work during the transition.
    pub edges: Vec<TransitionEdge>,

    /// Whether this transition contains connecting steps/movements
    /// (important for GOE evaluation of the following element).
    pub has_connecting_steps: bool,
}

impl Transition {
    /// Create a new transition.
    pub fn new(
        id: TransitionId,
        to_element: ElementId,
        start: TimeCode,
        duration: Duration,
        path: IcePath,
    ) -> Self {
        Self {
            id,
            from_element: None,
            to_element,
            start,
            duration,
            path,
            quality: TransitionQuality::Stroking,
            edges: Vec::new(),
            has_connecting_steps: false,
        }
    }

    /// Builder: set from_element.
    pub fn with_from(mut self, from: ElementId) -> Self {
        self.from_element = Some(from);
        self
    }

    /// Builder: set quality.
    pub fn with_quality(mut self, quality: TransitionQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Builder: set connecting steps flag.
    pub fn with_connecting_steps(mut self, has_steps: bool) -> Self {
        self.has_connecting_steps = has_steps;
        self
    }

    /// Get end time.
    pub fn end_time(&self) -> TimeCode {
        self.start + self.duration
    }
}

/// Movement quality of a transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionQuality {
    /// Simple stroking/crossovers.
    Stroking,
    /// Basic edge work.
    EdgeWork,
    /// Complex footwork.
    Footwork,
    /// Choreographic movements (spirals, spreads, etc.).
    Choreographic,
    /// Difficult connecting steps.
    DifficultSteps,
}

/// An edge used during a transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionEdge {
    /// The edge.
    pub edge: Edge,
    /// Start time within the transition (relative).
    pub start_offset: Duration,
    /// Duration on this edge.
    pub duration: Duration,
    /// Approximate position on the rink.
    pub position: Position,
}

// =============================================================================
// Skater Reference (pairs/dance)
// =============================================================================

/// Reference to a skater in a pairs/dance program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkaterRef {
    /// Skater name or identifier.
    pub name: String,
    /// Role in the partnership.
    pub role: SkaterRole,
}

/// Role in a pairs/dance partnership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkaterRole {
    /// Lead partner (traditionally the man).
    Lead,
    /// Follow partner (traditionally the woman).
    Follow,
    /// Solo element (both skaters in unison, or single skater).
    Solo,
}

// =============================================================================
// Music Map
// =============================================================================

/// Mapping between musical time (beats, measures) and absolute time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicMap {
    /// Tempo in beats per minute.
    pub bpm: f64,

    /// Time signature numerator (e.g., 4 for 4/4 time).
    pub beats_per_measure: u32,

    /// Tempo changes throughout the program.
    pub tempo_changes: Vec<TempoChange>,

    /// Specific cue points (musical accents, phrase boundaries).
    pub cue_points: Vec<CuePoint>,
}

impl MusicMap {
    /// Create a new music map with constant tempo.
    pub fn new(bpm: f64, beats_per_measure: u32) -> Self {
        Self {
            bpm,
            beats_per_measure,
            tempo_changes: Vec::new(),
            cue_points: Vec::new(),
        }
    }

    /// Convert a beat number to a time code.
    pub fn beat_to_time(&self, beat: u32) -> TimeCode {
        // Find the tempo at the given beat
        let secs_per_beat = 60.0 / self.effective_bpm_at_beat(beat);
        TimeCode::from_secs(f64::from(beat) * secs_per_beat)
    }

    /// Get the effective BPM at a given beat.
    fn effective_bpm_at_beat(&self, beat: u32) -> f64 {
        // Walk through tempo changes to find the active one
        let mut current_bpm = self.bpm;
        for change in &self.tempo_changes {
            if change.at_beat <= beat {
                current_bpm = change.new_bpm;
            } else {
                break;
            }
        }
        current_bpm
    }
}

/// A tempo change point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoChange {
    /// Beat number where the change occurs.
    pub at_beat: u32,
    /// New tempo in BPM.
    pub new_bpm: f64,
}

/// A musical cue point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuePoint {
    /// Time of the cue.
    pub time: TimeCode,
    /// Beat number.
    pub beat: u32,
    /// Description (e.g., "chorus", "accent", "phrase boundary").
    pub label: String,
}

// =============================================================================
// Default Phase Generation
// =============================================================================

/// Generate default execution phases for an element.
fn default_phases_for(element: &EventKind, placement: &ElementPlacement) -> Vec<ElementPhase> {
    match element {
        EventKind::Jump { kind, .. } => {
            let entry_edge = kind.required_entry_edge();
            let exit_edge = kind.standard_exit_edge();
            default_jump_phases(placement, entry_edge, exit_edge)
        }
        EventKind::Spin { positions, .. } => {
            default_spin_phases(placement, positions.len())
        }
        EventKind::StepSequence { .. } => {
            default_step_phases(placement)
        }
        EventKind::Lift { .. } => {
            default_lift_phases(placement)
        }
        EventKind::Throw { kind, .. } => {
            let entry_edge = kind.required_entry_edge();
            let exit_edge = kind.standard_exit_edge();
            default_throw_phases(placement, entry_edge, exit_edge)
        }
        EventKind::DeathSpiral { edge, .. } => {
            default_death_spiral_phases(placement, *edge)
        }
        _ => Vec::new(),
    }
}

/// Default phases for a jump element.
fn default_jump_phases(
    placement: &ElementPlacement,
    entry_edge: Edge,
    exit_edge: Edge,
) -> Vec<ElementPhase> {
    let total = placement.duration.as_secs();
    // Jump timing: ~40% entry, ~20% airborne, ~40% exit
    let entry_dur = total * 0.4;
    let air_dur = total * 0.2;
    let exit_dur = total * 0.4;

    let pos = placement.position;
    let heading = placement.heading;

    vec![
        ElementPhase {
            kind: PhaseKind::Entry,
            start: placement.start,
            duration: Duration::from_secs(entry_dur),
            start_position: pos,
            end_position: pos,
            start_heading: heading,
            end_heading: heading,
            edge: Some(entry_edge),
            data: PhaseData::JumpEntry {
                approach_speed: 5.0, // ~5 m/s typical
                takeoff_edge: entry_edge,
            },
        },
        ElementPhase {
            kind: PhaseKind::Execution,
            start: placement.start + Duration::from_secs(entry_dur),
            duration: Duration::from_secs(air_dur),
            start_position: pos,
            end_position: pos,
            start_heading: heading,
            end_heading: heading.rotate(360.0), // Simplified: one full rotation per single
            edge: None,
            data: PhaseData::JumpAirborne {
                peak_height: 0.5, // ~0.5m typical for triples
                rotation_degrees: 360.0,
            },
        },
        ElementPhase {
            kind: PhaseKind::Exit,
            start: placement.start + Duration::from_secs(entry_dur + air_dur),
            duration: Duration::from_secs(exit_dur),
            start_position: pos,
            end_position: Position::new(pos.x() + 2.0, pos.y()), // ~2m flow-out
            start_heading: heading,
            end_heading: heading,
            edge: Some(exit_edge),
            data: PhaseData::JumpLanding {
                landing_edge: exit_edge,
                flow_out: 2.0,
            },
        },
    ]
}

/// Default phases for a spin element.
fn default_spin_phases(placement: &ElementPlacement, position_count: usize) -> Vec<ElementPhase> {
    let total = placement.duration.as_secs();
    let pos = placement.position;
    let heading = placement.heading;

    if position_count <= 1 {
        // Simple spin: entry + execution + exit
        let entry_dur = total * 0.15;
        let exec_dur = total * 0.7;
        let exit_dur = total * 0.15;

        vec![
            ElementPhase {
                kind: PhaseKind::Entry,
                start: placement.start,
                duration: Duration::from_secs(entry_dur),
                start_position: pos,
                end_position: pos,
                start_heading: heading,
                end_heading: heading,
                edge: Some(Edge::LBO),
                data: PhaseData::Generic,
            },
            ElementPhase {
                kind: PhaseKind::Execution,
                start: placement.start + Duration::from_secs(entry_dur),
                duration: Duration::from_secs(exec_dur),
                start_position: pos,
                end_position: pos,
                start_heading: heading,
                end_heading: heading,
                edge: None,
                data: PhaseData::SpinRotation {
                    revolutions: 8.0,
                    position: "single".to_string(),
                },
            },
            ElementPhase {
                kind: PhaseKind::Exit,
                start: placement.start + Duration::from_secs(entry_dur + exec_dur),
                duration: Duration::from_secs(exit_dur),
                start_position: pos,
                end_position: pos,
                start_heading: heading,
                end_heading: heading,
                edge: None,
                data: PhaseData::Generic,
            },
        ]
    } else {
        // Combination spin: entry + N position phases + exit
        let entry_dur = total * 0.1;
        let exit_dur = total * 0.1;
        let execution_total = total * 0.8;
        let per_position = execution_total / position_count as f64;

        let mut phases = vec![ElementPhase {
            kind: PhaseKind::Entry,
            start: placement.start,
            duration: Duration::from_secs(entry_dur),
            start_position: pos,
            end_position: pos,
            start_heading: heading,
            end_heading: heading,
            edge: Some(Edge::LBO),
            data: PhaseData::Generic,
        }];

        let mut time_offset = entry_dur;
        for i in 0..position_count {
            let phase_kind = if i == 0 {
                PhaseKind::Execution
            } else {
                PhaseKind::PositionChange
            };

            phases.push(ElementPhase {
                kind: phase_kind,
                start: placement.start + Duration::from_secs(time_offset),
                duration: Duration::from_secs(per_position),
                start_position: pos,
                end_position: pos,
                start_heading: heading,
                end_heading: heading,
                edge: None,
                data: PhaseData::SpinRotation {
                    revolutions: 3.0,
                    position: format!("position_{}", i + 1),
                },
            });
            time_offset += per_position;
        }

        phases.push(ElementPhase {
            kind: PhaseKind::Exit,
            start: placement.start + Duration::from_secs(time_offset),
            duration: Duration::from_secs(exit_dur),
            start_position: pos,
            end_position: pos,
            start_heading: heading,
            end_heading: heading,
            edge: None,
            data: PhaseData::Generic,
        });

        phases
    }
}

/// Default phases for a step sequence.
fn default_step_phases(placement: &ElementPlacement) -> Vec<ElementPhase> {
    let total = placement.duration.as_secs();
    let pos = placement.position;
    let end_pos = placement
        .end_position
        .unwrap_or(Position::new(-pos.x(), -pos.y())); // Default: traverse rink
    let heading = placement.heading;

    // Step sequences are a single extended execution
    vec![
        ElementPhase {
            kind: PhaseKind::Entry,
            start: placement.start,
            duration: Duration::from_secs(total * 0.05),
            start_position: pos,
            end_position: pos,
            start_heading: heading,
            end_heading: heading,
            edge: None,
            data: PhaseData::Generic,
        },
        ElementPhase {
            kind: PhaseKind::Execution,
            start: placement.start + Duration::from_secs(total * 0.05),
            duration: Duration::from_secs(total * 0.9),
            start_position: pos,
            end_position: end_pos,
            start_heading: heading,
            end_heading: heading.rotate(180.0),
            edge: None,
            data: PhaseData::StepSection {
                direction: "full_ice".to_string(),
                edges: vec![],
            },
        },
        ElementPhase {
            kind: PhaseKind::Exit,
            start: placement.start + Duration::from_secs(total * 0.95),
            duration: Duration::from_secs(total * 0.05),
            start_position: end_pos,
            end_position: end_pos,
            start_heading: heading.rotate(180.0),
            end_heading: heading.rotate(180.0),
            edge: None,
            data: PhaseData::Generic,
        },
    ]
}

/// Default phases for a lift element.
fn default_lift_phases(placement: &ElementPlacement) -> Vec<ElementPhase> {
    let total = placement.duration.as_secs();
    let pos = placement.position;
    let heading = placement.heading;

    // Lift: 20% entry/liftoff, 60% hold, 20% dismount
    vec![
        ElementPhase {
            kind: PhaseKind::Entry,
            start: placement.start,
            duration: Duration::from_secs(total * 0.2),
            start_position: pos,
            end_position: pos,
            start_heading: heading,
            end_heading: heading,
            edge: None,
            data: PhaseData::Generic,
        },
        ElementPhase {
            kind: PhaseKind::Execution,
            start: placement.start + Duration::from_secs(total * 0.2),
            duration: Duration::from_secs(total * 0.6),
            start_position: pos,
            end_position: Position::new(pos.x() + 5.0, pos.y()),
            start_heading: heading,
            end_heading: heading,
            edge: None,
            data: PhaseData::LiftHold {
                hold_position: "overhead".to_string(),
                height: 2.0,
            },
        },
        ElementPhase {
            kind: PhaseKind::Exit,
            start: placement.start + Duration::from_secs(total * 0.8),
            duration: Duration::from_secs(total * 0.2),
            start_position: Position::new(pos.x() + 5.0, pos.y()),
            end_position: Position::new(pos.x() + 7.0, pos.y()),
            start_heading: heading,
            end_heading: heading,
            edge: None,
            data: PhaseData::Generic,
        },
    ]
}

/// Default phases for a throw jump.
fn default_throw_phases(
    placement: &ElementPlacement,
    entry_edge: Edge,
    exit_edge: Edge,
) -> Vec<ElementPhase> {
    let total = placement.duration.as_secs();
    let pos = placement.position;
    let heading = placement.heading;

    vec![
        ElementPhase {
            kind: PhaseKind::Entry,
            start: placement.start,
            duration: Duration::from_secs(total * 0.3),
            start_position: pos,
            end_position: pos,
            start_heading: heading,
            end_heading: heading,
            edge: Some(entry_edge),
            data: PhaseData::JumpEntry {
                approach_speed: 4.0,
                takeoff_edge: entry_edge,
            },
        },
        ElementPhase {
            kind: PhaseKind::Execution,
            start: placement.start + Duration::from_secs(total * 0.3),
            duration: Duration::from_secs(total * 0.3),
            start_position: pos,
            end_position: Position::new(pos.x() + 4.0, pos.y()),
            start_heading: heading,
            end_heading: heading,
            edge: None,
            data: PhaseData::JumpAirborne {
                peak_height: 0.7,
                rotation_degrees: 360.0,
            },
        },
        ElementPhase {
            kind: PhaseKind::Exit,
            start: placement.start + Duration::from_secs(total * 0.6),
            duration: Duration::from_secs(total * 0.4),
            start_position: Position::new(pos.x() + 4.0, pos.y()),
            end_position: Position::new(pos.x() + 6.0, pos.y()),
            start_heading: heading,
            end_heading: heading,
            edge: Some(exit_edge),
            data: PhaseData::JumpLanding {
                landing_edge: exit_edge,
                flow_out: 3.0,
            },
        },
    ]
}

/// Default phases for a death spiral.
fn default_death_spiral_phases(
    placement: &ElementPlacement,
    edge: Edge,
) -> Vec<ElementPhase> {
    let total = placement.duration.as_secs();
    let pos = placement.position;
    let heading = placement.heading;

    vec![
        ElementPhase {
            kind: PhaseKind::Entry,
            start: placement.start,
            duration: Duration::from_secs(total * 0.2),
            start_position: pos,
            end_position: pos,
            start_heading: heading,
            end_heading: heading,
            edge: Some(edge),
            data: PhaseData::Generic,
        },
        ElementPhase {
            kind: PhaseKind::Execution,
            start: placement.start + Duration::from_secs(total * 0.2),
            duration: Duration::from_secs(total * 0.6),
            start_position: pos,
            end_position: pos,
            start_heading: heading,
            end_heading: heading.rotate(360.0),
            edge: Some(edge),
            data: PhaseData::SpinRotation {
                revolutions: 2.0,
                position: "death_spiral".to_string(),
            },
        },
        ElementPhase {
            kind: PhaseKind::Exit,
            start: placement.start + Duration::from_secs(total * 0.8),
            duration: Duration::from_secs(total * 0.2),
            start_position: pos,
            end_position: Position::new(pos.x() + 2.0, pos.y()),
            start_heading: heading,
            end_heading: heading,
            edge: None,
            data: PhaseData::Generic,
        },
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rink::Position;
    use crate::types::{Duration, TimeCode};
    use anv_core::skating::{JumpKind, Level, Rotations, SpinPosition};
    use pretty_assertions::assert_eq;

    fn test_metadata() -> ProgramMetadata {
        ProgramMetadata {
            name: "Test Program".to_string(),
            discipline: Discipline::MenSingles,
            segment_type: Some("short".to_string()),
            skaters: vec!["Test Skater".to_string()],
            season: None,
        }
    }

    #[test]
    fn test_choreography_new() {
        let choreo = Choreography::new(test_metadata());
        assert_eq!(choreo.metadata.name, "Test Program");
        assert_eq!(choreo.element_count(), 0);
        assert_eq!(choreo.transition_count(), 0);
    }

    #[test]
    fn test_choreography_with_segment() {
        let mut choreo = Choreography::new(test_metadata());

        let mut segment = ChoreoSegment::new("sp", "short", TimeCode::ZERO);

        let placement = ElementPlacement::new(
            TimeCode::from_secs(5.0),
            Duration::from_secs(3.0),
            Position::new(10.0, 5.0),
        );

        let element = PlacedElement::new(
            ElementId(1),
            EventKind::Jump {
                rotations: Rotations::Triple,
                kind: JumpKind::Axel,
            },
            placement,
        )
        .with_isu_code("3A");

        segment.elements.push(element);
        segment.end = TimeCode::from_secs(8.0);
        choreo.segments.push(segment);

        assert_eq!(choreo.element_count(), 1);
        assert!(choreo.element_by_id(ElementId(1)).is_some());
        assert!(choreo.element_by_id(ElementId(99)).is_none());
    }

    #[test]
    fn test_placed_element_phases() {
        let placement = ElementPlacement::new(
            TimeCode::from_secs(10.0),
            Duration::jump_duration(),
            Position::new(0.0, 0.0),
        );

        let mut elem = PlacedElement::new(
            ElementId(1),
            EventKind::Jump {
                rotations: Rotations::Triple,
                kind: JumpKind::Lutz,
            },
            placement,
        );

        elem.generate_default_phases();
        assert_eq!(elem.phases.len(), 3);
        assert_eq!(elem.phases[0].kind, PhaseKind::Entry);
        assert_eq!(elem.phases[1].kind, PhaseKind::Execution);
        assert_eq!(elem.phases[2].kind, PhaseKind::Exit);

        // Entry should have Lutz takeoff edge (LBO)
        if let PhaseData::JumpEntry { takeoff_edge, .. } = &elem.phases[0].data {
            assert_eq!(*takeoff_edge, Edge::LBO);
        } else {
            panic!("Expected JumpEntry phase data");
        }
    }

    #[test]
    fn test_combination_spin_phases() {
        let placement = ElementPlacement::new(
            TimeCode::from_secs(60.0),
            Duration::spin_duration(),
            Position::new(-5.0, 8.0),
        );

        let mut elem = PlacedElement::new(
            ElementId(2),
            EventKind::Spin {
                positions: vec![SpinPosition::Camel, SpinPosition::Sit, SpinPosition::Upright],
                level: Level::L4,
                flying: false,
                change_foot: true,
            },
            placement,
        );

        elem.generate_default_phases();
        // Entry + 3 position phases + exit = 5
        assert_eq!(elem.phases.len(), 5);
        assert_eq!(elem.phases[0].kind, PhaseKind::Entry);
        assert_eq!(elem.phases[1].kind, PhaseKind::Execution); // first position
        assert_eq!(elem.phases[2].kind, PhaseKind::PositionChange);
        assert_eq!(elem.phases[3].kind, PhaseKind::PositionChange);
        assert_eq!(elem.phases[4].kind, PhaseKind::Exit);
    }

    #[test]
    fn test_segment_navigation() {
        let mut segment = ChoreoSegment::new("sp", "short", TimeCode::ZERO);

        let place1 = ElementPlacement::new(
            TimeCode::from_secs(5.0),
            Duration::from_secs(3.0),
            Position::new(0.0, 0.0),
        );
        let place2 = ElementPlacement::new(
            TimeCode::from_secs(12.0),
            Duration::from_secs(10.0),
            Position::new(5.0, 5.0),
        );

        segment.elements.push(PlacedElement::new(
            ElementId(1),
            EventKind::Jump {
                rotations: Rotations::Triple,
                kind: JumpKind::Axel,
            },
            place1,
        ));
        segment.elements.push(PlacedElement::new(
            ElementId(2),
            EventKind::Spin {
                positions: vec![SpinPosition::Camel],
                level: Level::L4,
                flying: true,
                change_foot: false,
            },
            place2,
        ));

        let next = segment.next_element(ElementId(1));
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, ElementId(2));

        let no_next = segment.next_element(ElementId(2));
        assert!(no_next.is_none());
    }

    #[test]
    fn test_music_map_beat_to_time() {
        let map = MusicMap::new(120.0, 4); // 120 BPM, 4/4 time
        let time = map.beat_to_time(4);
        // 4 beats at 120 BPM = 4 * (60/120) = 2.0 seconds
        assert!((time.as_secs() - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_choreography_json_export() {
        let mut choreo = Choreography::new(test_metadata());
        let segment = ChoreoSegment::new("sp", "short", TimeCode::ZERO);
        choreo.segments.push(segment);

        let json = choreo.to_json().expect("JSON export failed");
        assert!(json.contains("Test Program"));
        assert!(json.contains("MenSingles"));
    }

    #[test]
    fn test_transition() {
        use crate::path::IcePath;

        let path = IcePath::straight(Position::new(0.0, 0.0), Position::new(10.0, 5.0));

        let transition = Transition::new(
            TransitionId(1),
            ElementId(2),
            TimeCode::from_secs(8.0),
            Duration::from_secs(4.0),
            path,
        )
        .with_from(ElementId(1))
        .with_quality(TransitionQuality::Footwork)
        .with_connecting_steps(true);

        assert_eq!(transition.from_element, Some(ElementId(1)));
        assert_eq!(transition.to_element, ElementId(2));
        assert!(transition.has_connecting_steps);
        assert_eq!(transition.end_time().as_secs(), 12.0);
    }
}
