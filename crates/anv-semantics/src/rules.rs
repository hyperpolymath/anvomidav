// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! ISU rules definitions for figure skating.
//!
//! This module defines the rules for different skating disciplines and segment types
//! according to ISU regulations.

use anv_syntax::ast::SegmentKind;

/// Skating discipline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Discipline {
    /// Men's singles.
    MenSingles,
    /// Ladies' singles.
    LadiesSingles,
    /// Pairs skating.
    Pairs,
    /// Ice dance.
    IceDance,
}

impl Discipline {
    /// Returns the segment rules for this discipline and segment kind.
    pub fn segment_rules(&self, kind: SegmentKind) -> SegmentRules {
        match (self, kind) {
            // Men's Singles
            (Discipline::MenSingles, SegmentKind::Short) => SegmentRules::men_short(),
            (Discipline::MenSingles, SegmentKind::Free) => SegmentRules::men_free(),

            // Ladies' Singles
            (Discipline::LadiesSingles, SegmentKind::Short) => SegmentRules::ladies_short(),
            (Discipline::LadiesSingles, SegmentKind::Free) => SegmentRules::ladies_free(),

            // Pairs
            (Discipline::Pairs, SegmentKind::Short) => SegmentRules::pairs_short(),
            (Discipline::Pairs, SegmentKind::Free) => SegmentRules::pairs_free(),

            // Ice Dance
            (Discipline::IceDance, SegmentKind::Rhythm) => SegmentRules::rhythm_dance(),
            (Discipline::IceDance, SegmentKind::Free) => SegmentRules::free_dance(),
            (Discipline::IceDance, SegmentKind::Pattern) => SegmentRules::pattern_dance(),
            (Discipline::IceDance, SegmentKind::Short) => SegmentRules::rhythm_dance(), // Use rhythm for short

            // Exhibition (no strict rules)
            (_, SegmentKind::Exhibition) => SegmentRules::exhibition(),

            // Pattern dance for non-ice-dance (shouldn't happen, but handle gracefully)
            (_, SegmentKind::Pattern) => SegmentRules::default(),

            // Rhythm for non-ice-dance
            (_, SegmentKind::Rhythm) => SegmentRules::default(),
        }
    }
}

/// Rules for a specific segment type.
#[derive(Debug, Clone, Default)]
pub struct SegmentRules {
    /// Minimum duration in seconds.
    pub min_duration: Option<u32>,
    /// Maximum duration in seconds.
    pub max_duration: Option<u32>,

    /// Maximum number of jump elements.
    pub max_jumps: Option<u32>,
    /// Maximum number of spin elements.
    pub max_spins: Option<u32>,
    /// Required step sequence count.
    pub step_sequences: Option<u32>,

    /// Maximum number of triple/quad jumps.
    pub max_triple_quads: Option<u32>,
    /// Maximum axels allowed.
    pub max_axels: Option<u32>,

    // Pairs-specific
    /// Required lifts.
    pub required_lifts: Option<u32>,
    /// Required throws.
    pub required_throws: Option<u32>,
    /// Required twists.
    pub required_twists: Option<u32>,
    /// Required death spirals.
    pub required_death_spirals: Option<u32>,

    // Ice dance-specific
    /// Required pattern repetitions.
    pub pattern_repetitions: Option<u32>,
    /// Required twizzles.
    pub required_twizzles: Option<u32>,

    /// Required elements list.
    pub required_elements: Vec<RequiredElement>,
}

/// A required element specification.
#[derive(Debug, Clone)]
pub struct RequiredElement {
    /// Element type.
    pub element_type: RequiredElementType,
    /// Minimum count.
    pub min_count: u32,
    /// Maximum count (None = unlimited).
    pub max_count: Option<u32>,
    /// Description.
    pub description: String,
}

/// Types of required elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredElementType {
    Jump,
    Spin,
    StepSequence,
    ChoreographicSequence,
    Lift,
    Throw,
    Twist,
    DeathSpiral,
    Pattern,
    Twizzle,
}

impl SegmentRules {
    /// Men's short program rules (ISU 2024).
    pub fn men_short() -> Self {
        Self {
            min_duration: Some(160), // 2:40
            max_duration: Some(170), // 2:50
            max_jumps: Some(3),
            max_spins: Some(3),
            step_sequences: Some(1),
            max_triple_quads: Some(3),
            max_axels: Some(1),
            required_elements: vec![
                RequiredElement {
                    element_type: RequiredElementType::Jump,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Double or triple Axel".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Jump,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Triple or quad jump with steps immediately preceding".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Jump,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Jump combination (triple-triple or quad-triple)".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Spin,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Flying spin".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Spin,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Camel spin or sit spin with only one change of foot".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Spin,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Spin combination with only one change of foot".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::StepSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Step sequence".into(),
                },
            ],
            ..Default::default()
        }
    }

    /// Men's free skate rules (ISU 2024).
    pub fn men_free() -> Self {
        Self {
            min_duration: Some(240), // 4:00
            max_duration: Some(270), // 4:30
            max_jumps: Some(7),
            max_spins: Some(3),
            step_sequences: Some(1),
            max_triple_quads: Some(7),
            max_axels: Some(2),
            required_elements: vec![
                RequiredElement {
                    element_type: RequiredElementType::Jump,
                    min_count: 7,
                    max_count: Some(7),
                    description: "Maximum 7 jump elements".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Spin,
                    min_count: 3,
                    max_count: Some(3),
                    description: "3 spins of different nature".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::StepSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Step sequence".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::ChoreographicSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Choreographic sequence".into(),
                },
            ],
            ..Default::default()
        }
    }

    /// Ladies' short program rules (ISU 2024).
    pub fn ladies_short() -> Self {
        Self {
            min_duration: Some(160), // 2:40
            max_duration: Some(170), // 2:50
            max_jumps: Some(3),
            max_spins: Some(3),
            step_sequences: Some(1),
            max_triple_quads: Some(3),
            max_axels: Some(1),
            required_elements: vec![
                RequiredElement {
                    element_type: RequiredElementType::Jump,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Double or triple Axel".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Jump,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Triple or quad jump with steps immediately preceding".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Jump,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Jump combination".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Spin,
                    min_count: 3,
                    max_count: Some(3),
                    description: "3 spins of different nature".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::StepSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Step sequence".into(),
                },
            ],
            ..Default::default()
        }
    }

    /// Ladies' free skate rules (ISU 2024).
    pub fn ladies_free() -> Self {
        Self {
            min_duration: Some(240), // 4:00
            max_duration: Some(270), // 4:30
            max_jumps: Some(7),
            max_spins: Some(3),
            step_sequences: Some(1),
            max_triple_quads: Some(7),
            max_axels: Some(2),
            required_elements: vec![
                RequiredElement {
                    element_type: RequiredElementType::Jump,
                    min_count: 7,
                    max_count: Some(7),
                    description: "Maximum 7 jump elements".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Spin,
                    min_count: 3,
                    max_count: Some(3),
                    description: "3 spins of different nature".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::StepSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Step sequence".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::ChoreographicSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Choreographic sequence".into(),
                },
            ],
            ..Default::default()
        }
    }

    /// Pairs short program rules (ISU 2024).
    pub fn pairs_short() -> Self {
        Self {
            min_duration: Some(160), // 2:40
            max_duration: Some(170), // 2:50
            max_jumps: Some(3),
            max_spins: Some(1),
            step_sequences: Some(1),
            required_lifts: Some(1),
            required_throws: Some(1),
            required_twists: Some(1),
            required_death_spirals: Some(1),
            required_elements: vec![
                RequiredElement {
                    element_type: RequiredElementType::Twist,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Twist lift".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Lift,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Lift (Group 3, 4, or 5)".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Throw,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Throw jump".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Jump,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Solo jump".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::StepSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Step sequence".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::DeathSpiral,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Death spiral".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Spin,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Pair spin combination".into(),
                },
            ],
            ..Default::default()
        }
    }

    /// Pairs free skate rules (ISU 2024).
    pub fn pairs_free() -> Self {
        Self {
            min_duration: Some(240), // 4:00
            max_duration: Some(270), // 4:30
            max_jumps: Some(3),
            max_spins: Some(2),
            step_sequences: Some(1),
            required_lifts: Some(3),
            required_throws: Some(2),
            required_twists: Some(1),
            required_death_spirals: Some(1),
            required_elements: vec![
                RequiredElement {
                    element_type: RequiredElementType::Twist,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Twist lift".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Lift,
                    min_count: 3,
                    max_count: Some(3),
                    description: "3 different lifts".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Throw,
                    min_count: 2,
                    max_count: Some(2),
                    description: "2 throw jumps".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Jump,
                    min_count: 1,
                    max_count: Some(3),
                    description: "Solo or synchronized jumps".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::DeathSpiral,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Death spiral".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::StepSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Choreographic sequence".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Spin,
                    min_count: 2,
                    max_count: Some(2),
                    description: "Pair spin and solo spin".into(),
                },
            ],
            ..Default::default()
        }
    }

    /// Rhythm dance rules (ISU 2024).
    pub fn rhythm_dance() -> Self {
        Self {
            min_duration: Some(170), // 2:50
            max_duration: Some(180), // 3:00
            step_sequences: Some(1),
            required_twizzles: Some(1),
            required_elements: vec![
                RequiredElement {
                    element_type: RequiredElementType::Pattern,
                    min_count: 1,
                    max_count: Some(2),
                    description: "Pattern dance section".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Twizzle,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Twizzle sequence".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::StepSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Partial step sequence".into(),
                },
            ],
            ..Default::default()
        }
    }

    /// Free dance rules (ISU 2024).
    pub fn free_dance() -> Self {
        Self {
            min_duration: Some(240), // 4:00
            max_duration: Some(250), // 4:10
            required_twizzles: Some(1),
            required_lifts: Some(3),
            required_elements: vec![
                RequiredElement {
                    element_type: RequiredElementType::Lift,
                    min_count: 3,
                    max_count: Some(3),
                    description: "3 different lifts".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Twizzle,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Twizzle sequence".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::Spin,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Dance spin".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::StepSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Step sequence".into(),
                },
                RequiredElement {
                    element_type: RequiredElementType::ChoreographicSequence,
                    min_count: 1,
                    max_count: Some(1),
                    description: "Choreographic element".into(),
                },
            ],
            ..Default::default()
        }
    }

    /// Pattern dance rules.
    pub fn pattern_dance() -> Self {
        Self {
            pattern_repetitions: Some(2),
            required_elements: vec![RequiredElement {
                element_type: RequiredElementType::Pattern,
                min_count: 2,
                max_count: Some(2),
                description: "Required pattern sequences".into(),
            }],
            ..Default::default()
        }
    }

    /// Exhibition/gala rules (no strict requirements).
    pub fn exhibition() -> Self {
        Self::default()
    }
}

/// ISU rules configuration.
#[derive(Debug, Clone)]
pub struct ISURules {
    /// Season year (e.g., 2024 for 2024-2025 season).
    pub season: u32,
    /// Whether to enforce strict ISU compliance.
    pub strict: bool,
}

impl Default for ISURules {
    fn default() -> Self {
        Self {
            season: 2024,
            strict: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_men_short_rules() {
        let rules = SegmentRules::men_short();
        assert_eq!(rules.max_jumps, Some(3));
        assert_eq!(rules.max_spins, Some(3));
        assert_eq!(rules.step_sequences, Some(1));
    }

    #[test]
    fn test_pairs_short_rules() {
        let rules = SegmentRules::pairs_short();
        assert_eq!(rules.required_lifts, Some(1));
        assert_eq!(rules.required_throws, Some(1));
        assert_eq!(rules.required_death_spirals, Some(1));
    }

    #[test]
    fn test_rhythm_dance_rules() {
        let rules = SegmentRules::rhythm_dance();
        assert_eq!(rules.required_twizzles, Some(1));
        assert!(rules.required_elements.iter().any(|e| e.element_type == RequiredElementType::Pattern));
    }

    #[test]
    fn test_discipline_segment_rules() {
        let rules = Discipline::MenSingles.segment_rules(SegmentKind::Short);
        assert_eq!(rules.max_jumps, Some(3));

        let rules = Discipline::Pairs.segment_rules(SegmentKind::Free);
        assert_eq!(rules.required_lifts, Some(3));
    }
}
