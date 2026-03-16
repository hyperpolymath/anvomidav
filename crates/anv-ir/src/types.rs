// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! Core types for the IR.

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// Time code in seconds from program start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TimeCode(OrderedFloat<f64>);

impl TimeCode {
    /// Create a new time code from seconds.
    pub fn from_secs(secs: f64) -> Self {
        Self(OrderedFloat(secs))
    }

    /// Create a new time code from minutes and seconds.
    pub fn from_mins_secs(mins: u32, secs: u32) -> Self {
        Self::from_secs(f64::from(mins) * 60.0 + f64::from(secs))
    }

    /// Get the time in seconds.
    pub fn as_secs(&self) -> f64 {
        self.0.into_inner()
    }

    /// Get minutes component.
    pub fn minutes(&self) -> u32 {
        (self.as_secs() / 60.0) as u32
    }

    /// Get seconds component (0-59).
    pub fn seconds(&self) -> u32 {
        (self.as_secs() % 60.0) as u32
    }

    /// Zero time code.
    pub const ZERO: Self = Self(OrderedFloat(0.0));
}

impl Default for TimeCode {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add<Duration> for TimeCode {
    type Output = TimeCode;

    fn add(self, rhs: Duration) -> Self::Output {
        TimeCode::from_secs(self.as_secs() + rhs.as_secs())
    }
}

impl Sub for TimeCode {
    type Output = Duration;

    fn sub(self, rhs: TimeCode) -> Self::Output {
        Duration::from_secs(self.as_secs() - rhs.as_secs())
    }
}

impl fmt::Display for TimeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:02}", self.minutes(), self.seconds())
    }
}

/// Duration in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Duration(OrderedFloat<f64>);

impl Duration {
    /// Create a duration from seconds.
    pub fn from_secs(secs: f64) -> Self {
        Self(OrderedFloat(secs))
    }

    /// Get the duration in seconds.
    pub fn as_secs(&self) -> f64 {
        self.0.into_inner()
    }

    /// Zero duration.
    pub const ZERO: Self = Self(OrderedFloat(0.0));

    /// Standard element durations (approximate).
    pub fn jump_duration() -> Self {
        Self::from_secs(3.0)
    }

    pub fn spin_duration() -> Self {
        Self::from_secs(10.0)
    }

    pub fn step_sequence_duration() -> Self {
        Self::from_secs(25.0)
    }

    pub fn lift_duration() -> Self {
        Self::from_secs(8.0)
    }

    pub fn throw_duration() -> Self {
        Self::from_secs(4.0)
    }

    pub fn twist_duration() -> Self {
        Self::from_secs(3.0)
    }

    pub fn death_spiral_duration() -> Self {
        Self::from_secs(6.0)
    }

    pub fn choreographic_duration() -> Self {
        Self::from_secs(5.0)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Self::Output {
        Duration::from_secs(self.as_secs() + rhs.as_secs())
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}s", self.as_secs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timecode_from_mins_secs() {
        let tc = TimeCode::from_mins_secs(2, 30);
        assert_eq!(tc.as_secs(), 150.0);
        assert_eq!(tc.minutes(), 2);
        assert_eq!(tc.seconds(), 30);
    }

    #[test]
    fn test_timecode_display() {
        let tc = TimeCode::from_mins_secs(1, 5);
        assert_eq!(format!("{}", tc), "1:05");
    }

    #[test]
    fn test_timecode_arithmetic() {
        let t1 = TimeCode::from_secs(10.0);
        let t2 = TimeCode::from_secs(5.0);
        let dur = Duration::from_secs(3.0);

        assert_eq!((t1 - t2).as_secs(), 5.0);
        assert_eq!((t2 + dur).as_secs(), 8.0);
    }

    #[test]
    fn test_duration_add() {
        let d1 = Duration::from_secs(5.0);
        let d2 = Duration::from_secs(3.0);
        assert_eq!((d1 + d2).as_secs(), 8.0);
    }
}
