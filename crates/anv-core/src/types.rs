// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Core numeric and dimensional types.

use derive_more::{Add, Display, From, Into, Mul, Sub};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

/// Time point in seconds from program start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Add, Sub, From, Into, Display, Serialize, Deserialize)]
#[display(fmt = "{}", "_0.0")]
pub struct Time(pub OrderedFloat<f64>);

impl Time {
    pub const ZERO: Time = Time(OrderedFloat(0.0));

    pub fn new(seconds: f64) -> Self {
        Time(OrderedFloat(seconds))
    }

    pub fn as_secs(&self) -> f64 {
        self.0.into_inner()
    }

    /// Parse time from mm:ss or mm:ss.ms format
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.as_slice() {
            [mins, secs] => {
                let m: f64 = mins.parse().ok()?;
                let s: f64 = secs.parse().ok()?;
                Some(Time::new(m * 60.0 + s))
            }
            [hours, mins, secs] => {
                let h: f64 = hours.parse().ok()?;
                let m: f64 = mins.parse().ok()?;
                let s: f64 = secs.parse().ok()?;
                Some(Time::new(h * 3600.0 + m * 60.0 + s))
            }
            _ => None,
        }
    }

    /// Format as mm:ss.ms
    pub fn format_mmss(&self) -> String {
        let total_secs = self.as_secs();
        let mins = (total_secs / 60.0).floor() as u32;
        let secs = total_secs % 60.0;
        if secs.fract() == 0.0 {
            format!("{}:{:02}", mins, secs as u32)
        } else {
            format!("{}:{:05.2}", mins, secs)
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Duration in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Add, Sub, Mul, From, Into, Display, Serialize, Deserialize)]
#[display(fmt = "{}s", "_0.0")]
pub struct Duration(pub OrderedFloat<f64>);

impl Duration {
    pub const ZERO: Duration = Duration(OrderedFloat(0.0));

    pub fn new(seconds: f64) -> Self {
        Duration(OrderedFloat(seconds))
    }

    pub fn from_millis(ms: u64) -> Self {
        Duration::new(ms as f64 / 1000.0)
    }

    pub fn as_secs(&self) -> f64 {
        self.0.into_inner()
    }

    pub fn as_millis(&self) -> u64 {
        (self.as_secs() * 1000.0) as u64
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Angle in radians.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Add, Sub, Mul, From, Into, Serialize, Deserialize)]
pub struct Angle(pub OrderedFloat<f64>);

impl Angle {
    pub const ZERO: Angle = Angle(OrderedFloat(0.0));
    pub const PI: Angle = Angle(OrderedFloat(std::f64::consts::PI));
    pub const TWO_PI: Angle = Angle(OrderedFloat(std::f64::consts::TAU));

    pub fn from_radians(rad: f64) -> Self {
        Angle(OrderedFloat(rad))
    }

    pub fn from_degrees(deg: f64) -> Self {
        Angle::from_radians(deg.to_radians())
    }

    pub fn as_radians(&self) -> f64 {
        self.0.into_inner()
    }

    pub fn as_degrees(&self) -> f64 {
        self.as_radians().to_degrees()
    }

    /// Normalize to [0, 2π)
    pub fn normalize(&self) -> Self {
        let rad = self.as_radians().rem_euclid(std::f64::consts::TAU);
        Angle::from_radians(rad)
    }
}

impl Default for Angle {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::fmt::Display for Angle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}°", self.as_degrees())
    }
}

/// 2D position on the ice surface.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate (along long axis, positive toward judges)
    pub x: f64,
    /// Y coordinate (along short axis, positive toward right)
    pub y: f64,
}

impl Position {
    pub const ORIGIN: Position = Position { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }

    pub fn distance(&self, other: &Position) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn translate(&self, dx: f64, dy: f64) -> Self {
        Position::new(self.x + dx, self.y + dy)
    }

    pub fn rotate(&self, angle: Angle, center: &Position) -> Self {
        let cos = angle.as_radians().cos();
        let sin = angle.as_radians().sin();
        let dx = self.x - center.x;
        let dy = self.y - center.y;
        Position::new(
            center.x + dx * cos - dy * sin,
            center.y + dx * sin + dy * cos,
        )
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::ORIGIN
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

/// 3D position (includes height for jumps).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position3 {
    pub const ORIGIN: Position3 = Position3 { x: 0.0, y: 0.0, z: 0.0 };

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Position3 { x, y, z }
    }

    pub fn from_2d(pos: Position, z: f64) -> Self {
        Position3 { x: pos.x, y: pos.y, z }
    }

    pub fn to_2d(&self) -> Position {
        Position::new(self.x, self.y)
    }

    pub fn distance(&self, other: &Position3) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
    }
}

impl Default for Position3 {
    fn default() -> Self {
        Self::ORIGIN
    }
}

/// 2D velocity vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
}

impl Velocity {
    pub const ZERO: Velocity = Velocity { vx: 0.0, vy: 0.0 };

    pub fn new(vx: f64, vy: f64) -> Self {
        Velocity { vx, vy }
    }

    pub fn from_polar(speed: f64, angle: Angle) -> Self {
        Velocity::new(
            speed * angle.as_radians().cos(),
            speed * angle.as_radians().sin(),
        )
    }

    pub fn speed(&self) -> f64 {
        (self.vx.powi(2) + self.vy.powi(2)).sqrt()
    }

    pub fn direction(&self) -> Angle {
        Angle::from_radians(self.vy.atan2(self.vx))
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Beat position in musical time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Beat {
    /// Measure number (1-indexed)
    pub measure: u32,
    /// Beat within measure (1-indexed)
    pub beat: u32,
}

impl Beat {
    pub fn new(measure: u32, beat: u32) -> Self {
        Beat { measure, beat }
    }

    /// Convert to absolute beat number (0-indexed)
    pub fn to_absolute(&self, beats_per_measure: u32) -> u32 {
        (self.measure - 1) * beats_per_measure + (self.beat - 1)
    }

    /// Convert from absolute beat number
    pub fn from_absolute(abs_beat: u32, beats_per_measure: u32) -> Self {
        Beat {
            measure: abs_beat / beats_per_measure + 1,
            beat: abs_beat % beats_per_measure + 1,
        }
    }
}

impl std::fmt::Display for Beat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.measure, self.beat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_time_parse() {
        assert_eq!(Time::parse("1:30"), Some(Time::new(90.0)));
        assert_eq!(Time::parse("2:45.5"), Some(Time::new(165.5)));
        assert_eq!(Time::parse("0:00"), Some(Time::new(0.0)));
    }

    #[test]
    fn test_time_format() {
        assert_eq!(Time::new(90.0).format_mmss(), "1:30");
        assert_eq!(Time::new(165.5).format_mmss(), "2:45.50");
    }

    #[test]
    fn test_angle_normalize() {
        let a = Angle::from_degrees(450.0);
        assert!((a.normalize().as_degrees() - 90.0).abs() < 0.001);
    }

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0.0, 0.0);
        let p2 = Position::new(3.0, 4.0);
        assert!((p1.distance(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_beat_conversion() {
        let beat = Beat::new(3, 2);
        assert_eq!(beat.to_absolute(4), 9); // (3-1)*4 + (2-1) = 9
        assert_eq!(Beat::from_absolute(9, 4), beat);
    }
}
