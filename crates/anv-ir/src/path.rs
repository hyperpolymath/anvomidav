// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Ice surface path representation.
//!
//! Paths describe the trajectory of a skater across the rink surface. They are
//! composed of [`Waypoint`]s connected by curve segments (lines, arcs, or
//! Bezier curves). Paths are used both in transitions between elements and
//! within elements that involve travel (step sequences, lifts).
//!
//! # Coordinate System
//!
//! The coordinate system follows the rink module convention:
//! - Origin (0, 0) at center of rink
//! - X-axis: long axis (positive = toward judges)
//! - Y-axis: short axis (positive = toward kiss & cry)
//! - Units: meters

use crate::rink::{Heading, Position};
use serde::{Deserialize, Serialize};

// =============================================================================
// Ice Path
// =============================================================================

/// A path on the ice surface, represented as a sequence of waypoints.
///
/// The path is a polyline with optional curve information between waypoints.
/// For visualization, intermediate points can be interpolated along curves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcePath {
    /// Ordered waypoints along the path.
    pub waypoints: Vec<Waypoint>,

    /// Whether the path forms a closed loop.
    pub closed: bool,
}

impl IcePath {
    /// Create an empty path.
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            closed: false,
        }
    }

    /// Create a straight-line path between two points.
    pub fn straight(from: Position, to: Position) -> Self {
        Self {
            waypoints: vec![
                Waypoint::new(from).with_heading(Heading::from_degrees(
                    (to.y() - from.y()).atan2(to.x() - from.x()).to_degrees(),
                )),
                Waypoint::new(to).with_heading(Heading::from_degrees(
                    (to.y() - from.y()).atan2(to.x() - from.x()).to_degrees(),
                )),
            ],
            closed: false,
        }
    }

    /// Create a circular arc path.
    ///
    /// Generates waypoints along an arc from `start_angle` to `end_angle`
    /// (in degrees) around the given `center` with the given `radius`.
    pub fn arc(
        center: Position,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        segments: usize,
    ) -> Self {
        let segments = segments.max(2);
        let angle_step = (end_angle - start_angle) / (segments - 1) as f64;

        let waypoints = (0..segments)
            .map(|i| {
                let angle_deg = start_angle + angle_step * i as f64;
                let angle_rad = angle_deg.to_radians();
                let x = center.x() + radius * angle_rad.cos();
                let y = center.y() + radius * angle_rad.sin();
                // Tangent heading is perpendicular to the radius
                let tangent = angle_deg + 90.0;
                Waypoint::new(Position::new(x, y)).with_heading(Heading::from_degrees(tangent))
            })
            .collect();

        Self {
            waypoints,
            closed: (end_angle - start_angle).abs() >= 360.0,
        }
    }

    /// Create a serpentine (S-curve) path across the rink.
    ///
    /// Generates waypoints along a sinusoidal curve from `from` to `to`
    /// with the given number of half-periods (`waves`).
    pub fn serpentine(from: Position, to: Position, amplitude: f64, waves: usize) -> Self {
        let segments = waves.max(1) * 8; // 8 points per half-period
        let dx = to.x() - from.x();
        let dy = to.y() - from.y();
        let length = (dx * dx + dy * dy).sqrt();

        // Direction vectors
        let dir_x = dx / length;
        let dir_y = dy / length;
        // Perpendicular
        let perp_x = -dir_y;
        let perp_y = dir_x;

        let waypoints = (0..=segments)
            .map(|i| {
                let t = i as f64 / segments as f64;
                let wave_offset =
                    amplitude * (t * waves as f64 * std::f64::consts::PI).sin();

                let x = from.x() + dx * t + perp_x * wave_offset;
                let y = from.y() + dy * t + perp_y * wave_offset;

                // Approximate tangent heading
                let heading_angle = (dir_y + perp_y * amplitude * waves as f64
                    * std::f64::consts::PI
                    * (t * waves as f64 * std::f64::consts::PI).cos()
                    / segments as f64)
                    .atan2(
                        dir_x
                            + perp_x * amplitude * waves as f64
                                * std::f64::consts::PI
                                * (t * waves as f64 * std::f64::consts::PI).cos()
                                / segments as f64,
                    )
                    .to_degrees();

                Waypoint::new(Position::new(x, y)).with_heading(Heading::from_degrees(heading_angle))
            })
            .collect();

        Self {
            waypoints,
            closed: false,
        }
    }

    /// Add a waypoint to the path.
    pub fn push(&mut self, waypoint: Waypoint) {
        self.waypoints.push(waypoint);
    }

    /// Get the number of waypoints.
    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    /// Check if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }

    /// Get the first waypoint.
    pub fn start(&self) -> Option<&Waypoint> {
        self.waypoints.first()
    }

    /// Get the last waypoint.
    pub fn end(&self) -> Option<&Waypoint> {
        self.waypoints.last()
    }

    /// Get the start position.
    pub fn start_position(&self) -> Option<Position> {
        self.start().map(|w| w.position)
    }

    /// Get the end position.
    pub fn end_position(&self) -> Option<Position> {
        self.end().map(|w| w.position)
    }

    /// Calculate the total path distance (sum of segment lengths).
    pub fn total_distance(&self) -> f64 {
        self.waypoints
            .windows(2)
            .map(|pair| pair[0].position.distance_to(&pair[1].position))
            .sum()
    }

    /// Get the position at a fractional distance along the path (0.0 = start, 1.0 = end).
    ///
    /// Uses linear interpolation between waypoints.
    pub fn position_at(&self, t: f64) -> Option<Position> {
        if self.waypoints.is_empty() {
            return None;
        }
        if self.waypoints.len() == 1 {
            return Some(self.waypoints[0].position);
        }

        let t = t.clamp(0.0, 1.0);
        let total = self.total_distance();
        if total < 1e-9 {
            return Some(self.waypoints[0].position);
        }

        let target_dist = t * total;
        let mut accumulated = 0.0;

        for pair in self.waypoints.windows(2) {
            let seg_dist = pair[0].position.distance_to(&pair[1].position);
            if accumulated + seg_dist >= target_dist {
                let local_t = if seg_dist < 1e-9 {
                    0.0
                } else {
                    (target_dist - accumulated) / seg_dist
                };
                let x = pair[0].position.x() + local_t * (pair[1].position.x() - pair[0].position.x());
                let y = pair[0].position.y() + local_t * (pair[1].position.y() - pair[0].position.y());
                return Some(Position::new(x, y));
            }
            accumulated += seg_dist;
        }

        self.end_position()
    }

    /// Sample the path at N evenly-spaced points.
    pub fn sample(&self, n: usize) -> Vec<Position> {
        if n == 0 {
            return Vec::new();
        }
        if n == 1 {
            return self.start_position().into_iter().collect();
        }

        (0..n)
            .filter_map(|i| {
                let t = i as f64 / (n - 1) as f64;
                self.position_at(t)
            })
            .collect()
    }

    /// Compute the bounding box of the path: (min_x, min_y, max_x, max_y).
    pub fn bounding_box(&self) -> Option<(f64, f64, f64, f64)> {
        if self.waypoints.is_empty() {
            return None;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for wp in &self.waypoints {
            min_x = min_x.min(wp.position.x());
            min_y = min_y.min(wp.position.y());
            max_x = max_x.max(wp.position.x());
            max_y = max_y.max(wp.position.y());
        }

        Some((min_x, min_y, max_x, max_y))
    }
}

impl Default for IcePath {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Waypoint
// =============================================================================

/// A point along an ice path with optional metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    /// Position on the ice surface.
    pub position: Position,

    /// Heading at this point (direction of travel).
    pub heading: Option<Heading>,

    /// Speed at this point in m/s.
    pub speed: Option<f64>,

    /// Edge at this point.
    pub edge: Option<anv_core::skating::Edge>,

    /// Curve kind to the next waypoint.
    pub curve_to_next: CurveKind,
}

impl Waypoint {
    /// Create a new waypoint at a position.
    pub fn new(position: Position) -> Self {
        Self {
            position,
            heading: None,
            speed: None,
            edge: None,
            curve_to_next: CurveKind::Linear,
        }
    }

    /// Builder: set heading.
    pub fn with_heading(mut self, heading: Heading) -> Self {
        self.heading = Some(heading);
        self
    }

    /// Builder: set speed.
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Builder: set edge.
    pub fn with_edge(mut self, edge: anv_core::skating::Edge) -> Self {
        self.edge = Some(edge);
        self
    }

    /// Builder: set curve to next waypoint.
    pub fn with_curve(mut self, curve: CurveKind) -> Self {
        self.curve_to_next = curve;
        self
    }
}

// =============================================================================
// Curve Kind
// =============================================================================

/// Kind of curve segment between two waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CurveKind {
    /// Straight line.
    Linear,

    /// Circular arc with the given radius (positive = left turn, negative = right turn).
    Arc {
        radius: f64,
    },

    /// Cubic Bezier curve (control points are offsets from the waypoints).
    Bezier {
        /// Control point 1: offset from the start waypoint.
        cp1_dx: f64,
        cp1_dy: f64,
        /// Control point 2: offset from the end waypoint.
        cp2_dx: f64,
        cp2_dy: f64,
    },
}

impl Default for CurveKind {
    fn default() -> Self {
        CurveKind::Linear
    }
}

// =============================================================================
// Path Generators for Step Sequences
// =============================================================================

/// Generate a path for a step sequence based on its pattern type.
pub fn step_sequence_path(
    pattern: &crate::timeline::StepPattern,
    start: Position,
    heading: Heading,
) -> IcePath {
    match pattern {
        crate::timeline::StepPattern::Straight => {
            // Straight line across the rink, typically along the long axis
            let end = Position::new(
                start.x() + 40.0 * heading.as_radians().cos(),
                start.y() + 40.0 * heading.as_radians().sin(),
            );
            IcePath::straight(start, end)
        }
        crate::timeline::StepPattern::Circular => {
            // Circle around center ice, radius ~12m
            let center = Position::center_ice();
            let start_angle = (start.y() - center.y())
                .atan2(start.x() - center.x())
                .to_degrees();
            IcePath::arc(center, 12.0, start_angle, start_angle + 360.0, 36)
        }
        crate::timeline::StepPattern::Serpentine => {
            // S-curve across the rink
            let end = Position::new(-start.x(), -start.y());
            IcePath::serpentine(start, end, 8.0, 3)
        }
        crate::timeline::StepPattern::Diagonal => {
            // Diagonal across the rink
            let end = Position::new(-start.x(), -start.y());
            IcePath::straight(start, end)
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_straight_path() {
        let path = IcePath::straight(Position::new(0.0, 0.0), Position::new(10.0, 0.0));

        assert_eq!(path.len(), 2);
        assert!(!path.closed);
        assert!((path.total_distance() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_arc_path() {
        let path = IcePath::arc(Position::center_ice(), 10.0, 0.0, 360.0, 36);

        assert_eq!(path.len(), 36);
        assert!(path.closed);
        // Perimeter should be roughly 2*pi*10 = 62.8
        let dist = path.total_distance();
        assert!(
            (dist - 62.4).abs() < 1.0,
            "Arc perimeter was {} (expected ~62.4)",
            dist
        );
    }

    #[test]
    fn test_serpentine_path() {
        let path = IcePath::serpentine(
            Position::new(-20.0, 0.0),
            Position::new(20.0, 0.0),
            5.0,
            2,
        );

        assert!(path.len() > 2);
        assert!(!path.closed);
        // Should be longer than straight-line distance of 40m
        assert!(path.total_distance() > 40.0);
    }

    #[test]
    fn test_path_position_at() {
        let path = IcePath::straight(Position::new(0.0, 0.0), Position::new(10.0, 0.0));

        let mid = path.position_at(0.5).unwrap();
        assert!((mid.x() - 5.0).abs() < 0.001);
        assert!((mid.y() - 0.0).abs() < 0.001);

        let start = path.position_at(0.0).unwrap();
        assert!((start.x() - 0.0).abs() < 0.001);

        let end = path.position_at(1.0).unwrap();
        assert!((end.x() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_path_sample() {
        let path = IcePath::straight(Position::new(0.0, 0.0), Position::new(10.0, 0.0));
        let samples = path.sample(5);

        assert_eq!(samples.len(), 5);
        assert!((samples[0].x() - 0.0).abs() < 0.001);
        assert!((samples[2].x() - 5.0).abs() < 0.001);
        assert!((samples[4].x() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_path_bounding_box() {
        let path = IcePath::straight(Position::new(-5.0, -3.0), Position::new(10.0, 7.0));
        let (min_x, min_y, max_x, max_y) = path.bounding_box().unwrap();

        assert!((min_x - (-5.0)).abs() < 0.001);
        assert!((min_y - (-3.0)).abs() < 0.001);
        assert!((max_x - 10.0).abs() < 0.001);
        assert!((max_y - 7.0).abs() < 0.001);
    }

    #[test]
    fn test_empty_path() {
        let path = IcePath::new();
        assert!(path.is_empty());
        assert_eq!(path.total_distance(), 0.0);
        assert!(path.position_at(0.5).is_none());
        assert!(path.bounding_box().is_none());
    }

    #[test]
    fn test_waypoint_builder() {
        let wp = Waypoint::new(Position::new(1.0, 2.0))
            .with_heading(Heading::from_degrees(45.0))
            .with_speed(3.5)
            .with_edge(anv_core::skating::Edge::LFO)
            .with_curve(CurveKind::Arc { radius: 5.0 });

        assert_eq!(wp.position.x(), 1.0);
        assert!(wp.heading.is_some());
        assert_eq!(wp.speed, Some(3.5));
        assert_eq!(wp.edge, Some(anv_core::skating::Edge::LFO));
        assert!(matches!(wp.curve_to_next, CurveKind::Arc { radius } if (radius - 5.0).abs() < 0.001));
    }

    #[test]
    fn test_step_sequence_paths() {
        use crate::timeline::StepPattern;

        let start = Position::new(20.0, 10.0);
        let heading = Heading::from_degrees(180.0);

        let straight = step_sequence_path(&StepPattern::Straight, start, heading);
        assert!(straight.len() >= 2);
        assert!(straight.total_distance() > 0.0);

        let circular = step_sequence_path(&StepPattern::Circular, start, heading);
        assert!(circular.len() > 2);
        assert!(circular.closed);

        let serpentine = step_sequence_path(&StepPattern::Serpentine, start, heading);
        assert!(serpentine.len() > 2);
        assert!(!serpentine.closed);
    }
}
