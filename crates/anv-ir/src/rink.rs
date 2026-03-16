// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Rink geometry and positioning.
//!
//! Standard ISU rink dimensions:
//! - Length: 60 meters (56-61m allowed)
//! - Width: 30 meters (26-30m allowed)
//! - Corner radius: 8.5 meters
//!
//! Coordinate system:
//! - Origin (0, 0) at center of rink
//! - X-axis: long axis (positive = toward judges)
//! - Y-axis: short axis (positive = toward kiss & cry)

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Position on the ice surface in meters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate (along long axis, -30 to +30).
    pub x: OrderedFloat<f64>,
    /// Y coordinate (along short axis, -15 to +15).
    pub y: OrderedFloat<f64>,
}

impl Position {
    /// Create a new position.
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: OrderedFloat(x),
            y: OrderedFloat(y),
        }
    }

    /// Center of the rink.
    pub const CENTER: Self = Self {
        x: OrderedFloat(0.0),
        y: OrderedFloat(0.0),
    };

    /// Distance from center.
    pub fn distance_from_center(&self) -> f64 {
        (self.x.into_inner().powi(2) + self.y.into_inner().powi(2)).sqrt()
    }

    /// Distance to another position.
    pub fn distance_to(&self, other: &Position) -> f64 {
        let dx = self.x.into_inner() - other.x.into_inner();
        let dy = self.y.into_inner() - other.y.into_inner();
        (dx.powi(2) + dy.powi(2)).sqrt()
    }

    /// Get X as f64.
    pub fn x(&self) -> f64 {
        self.x.into_inner()
    }

    /// Get Y as f64.
    pub fn y(&self) -> f64 {
        self.y.into_inner()
    }

    /// Named positions on the rink.
    pub fn center_ice() -> Self {
        Self::CENTER
    }

    pub fn judges_side() -> Self {
        Self::new(25.0, 0.0)
    }

    pub fn far_end() -> Self {
        Self::new(-25.0, 0.0)
    }

    pub fn near_corner_right() -> Self {
        Self::new(22.0, 11.0)
    }

    pub fn near_corner_left() -> Self {
        Self::new(22.0, -11.0)
    }

    pub fn far_corner_right() -> Self {
        Self::new(-22.0, 11.0)
    }

    pub fn far_corner_left() -> Self {
        Self::new(-22.0, -11.0)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::CENTER
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x(), self.y())
    }
}

/// Standard rink dimensions per ISU regulations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RinkDimensions {
    /// Length in meters (standard: 60m).
    pub length: f64,
    /// Width in meters (standard: 30m).
    pub width: f64,
    /// Corner radius in meters (standard: 8.5m).
    pub corner_radius: f64,
}

impl RinkDimensions {
    /// Standard ISU Olympic rink.
    pub const OLYMPIC: Self = Self {
        length: 60.0,
        width: 30.0,
        corner_radius: 8.5,
    };

    /// NHL-sized rink (smaller).
    pub const NHL: Self = Self {
        length: 60.96, // 200 feet
        width: 25.91,  // 85 feet
        corner_radius: 8.53,
    };

    /// Check if a position is within the rink bounds.
    pub fn contains(&self, pos: &Position) -> bool {
        let half_length = self.length / 2.0;
        let half_width = self.width / 2.0;

        let x = pos.x().abs();
        let y = pos.y().abs();

        // Simple rectangular check (ignoring corners for now)
        x <= half_length && y <= half_width
    }

    /// Get the perimeter length (approximate).
    pub fn perimeter(&self) -> f64 {
        // Two straight sides + two curved ends
        let straight = 2.0 * (self.length - 2.0 * self.corner_radius);
        let curved = 2.0 * std::f64::consts::PI * self.corner_radius;
        straight + curved
    }

    /// Get the area in square meters.
    pub fn area(&self) -> f64 {
        // Rectangle minus corner cutoffs plus corner arcs
        let rect = self.length * self.width;
        let corner_cutoff = 4.0 * (self.corner_radius.powi(2) - std::f64::consts::PI * self.corner_radius.powi(2) / 4.0);
        rect - corner_cutoff
    }
}

impl Default for RinkDimensions {
    fn default() -> Self {
        Self::OLYMPIC
    }
}

/// Heading/direction on the ice in degrees.
/// 0° = toward judges, 90° = toward right side, 180° = away from judges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Heading(OrderedFloat<f64>);

impl Heading {
    /// Create a heading from degrees.
    pub fn from_degrees(deg: f64) -> Self {
        // Normalize to 0-360
        let normalized = ((deg % 360.0) + 360.0) % 360.0;
        Self(OrderedFloat(normalized))
    }

    /// Create a heading from radians.
    pub fn from_radians(rad: f64) -> Self {
        Self::from_degrees(rad.to_degrees())
    }

    /// Get heading in degrees.
    pub fn as_degrees(&self) -> f64 {
        self.0.into_inner()
    }

    /// Get heading in radians.
    pub fn as_radians(&self) -> f64 {
        self.as_degrees().to_radians()
    }

    /// Toward judges (0°).
    pub const TOWARD_JUDGES: Self = Self(OrderedFloat(0.0));

    /// Away from judges (180°).
    pub const AWAY_FROM_JUDGES: Self = Self(OrderedFloat(180.0));

    /// Rotate by given degrees.
    pub fn rotate(&self, degrees: f64) -> Self {
        Self::from_degrees(self.as_degrees() + degrees)
    }
}

impl Default for Heading {
    fn default() -> Self {
        Self::TOWARD_JUDGES
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0.0, 0.0);
        let p2 = Position::new(3.0, 4.0);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_rink_contains() {
        let rink = RinkDimensions::OLYMPIC;
        assert!(rink.contains(&Position::CENTER));
        assert!(rink.contains(&Position::new(25.0, 10.0)));
        assert!(!rink.contains(&Position::new(35.0, 0.0)));
    }

    #[test]
    fn test_heading_normalize() {
        let h = Heading::from_degrees(450.0);
        assert_eq!(h.as_degrees(), 90.0);

        let h2 = Heading::from_degrees(-90.0);
        assert_eq!(h2.as_degrees(), 270.0);
    }

    #[test]
    fn test_heading_rotate() {
        let h = Heading::TOWARD_JUDGES;
        let rotated = h.rotate(90.0);
        assert_eq!(rotated.as_degrees(), 90.0);
    }
}
