use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Represents an actor's selected awakening configuration.
///
/// # NTE Mechanics: Individual Nodes vs. Resonance Milestones
///
/// In *Neverness to Ever* (NTE), character awakenings follow a two-tier system:
///
/// 1. **Individual Awakening Nodes (`Effect1`..`Effect6`)**:
///    Each character has 6 distinct nodes (often referred to as A1..A6). These are
///    **identity-gated**: a mechanic tied to node `N` only activates if index `N` is
///    explicitly selected. Checked via [`has_effect(N)`](Self::has_effect).
///
/// 2. **Resonance Milestone Bonuses (`resonance_3` and `resonance_6`)**:
///    These are threshold/milestone bonuses triggered by the **total count** of unlocked
///    awakenings, NOT by selecting node 3 or 6 specifically:
///    - `resonance_3` activates when **at least 3** awakening effects are selected (`count >= 3`).
///      For almost all characters, this grants +1 level to Melee, Skill, and UltraSkill.
///    - `resonance_6` activates when **all 6** awakening effects are selected (`count >= 6`).
///      This grants a character-specific milestone passive (such as team Crit DMG or DEF ignore).
///
/// ### Examples
/// - `awakening=[3]` selects ONLY `Effect3`: `has_effect(3)` is `true`, but `has_resonance_3()` is `false` (`count == 1 < 3`).
/// - `awakening=[1, 2, 4]` selects nodes 1, 2, and 4: `has_effect(3)` is `false`, but `has_resonance_3()` is `true` (`count == 3 >= 3`).
/// - `awakening=3` is prefix sugar for `awakening=[1, 2, 3]`: here both `has_effect(3)` and `has_resonance_3()` are `true`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AwakeningSelection(Vec<u8>);

impl AwakeningSelection {
    pub fn prefix(level: u32) -> Self {
        assert!(level <= 6, "awakening prefix level must be in 0..=6");
        Self((1..=level).map(|index| index as u8).collect())
    }

    pub fn selected(effects: impl IntoIterator<Item = u8>) -> Result<Self, String> {
        let mut effects = effects.into_iter().collect::<Vec<_>>();
        if effects.len() > 6 || effects.iter().any(|effect| !(1..=6).contains(effect)) {
            return Err("awakening effects must be distinct indices in 1..=6".into());
        }
        effects.sort_unstable();
        if effects.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err("awakening effects must be distinct indices in 1..=6".into());
        }
        Ok(Self(effects))
    }

    /// Checks if a specific individual awakening node (1..=6) is selected.
    ///
    /// This is identity-gated: `has_effect(3)` only returns true if node 3 was explicitly chosen.
    pub fn has_effect(&self, effect: u8) -> bool {
        self.0.binary_search(&effect).is_ok()
    }

    /// Total count of selected awakening effects (0..=6).
    pub fn count(&self) -> u32 {
        self.0.len() as u32
    }

    /// Returns true if the `resonance_3` milestone bonus is active (at least 3 effects selected).
    ///
    /// In NTE mechanics, `resonance_3` is a milestone bonus unlocked by the TOTAL COUNT
    /// of selected awakenings (`count >= 3`), NOT by selecting node 3 (`has_effect(3)`).
    #[inline]
    pub fn has_resonance_3(&self) -> bool {
        self.count() >= 3
    }

    /// Returns true if the `resonance_6` milestone bonus is active (all 6 effects selected).
    ///
    /// In NTE mechanics, `resonance_6` is a milestone bonus unlocked by the TOTAL COUNT
    /// of selected awakenings (`count >= 6`), NOT by selecting node 6 (`has_effect(6)`).
    #[inline]
    pub fn has_resonance_6(&self) -> bool {
        self.count() >= 6
    }

    pub fn is_prefix(&self) -> bool {
        self.0.iter().copied().eq(1..=self.count() as u8)
    }
}

impl Serialize for AwakeningSelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawAwakening {
    Rank(u32),
    List(Vec<u8>),
}

impl<'de> Deserialize<'de> for AwakeningSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawAwakening::deserialize(deserializer)?;
        match raw {
            RawAwakening::Rank(level) => {
                if level > 6 {
                    return Err(serde::de::Error::custom(format!(
                        "awakening level {level} exceeds maximum 6"
                    )));
                }
                Ok(Self::prefix(level))
            }
            RawAwakening::List(effects) => {
                Self::selected(effects).map_err(serde::de::Error::custom)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AwakeningSelection;

    #[test]
    fn selection_deserializes_into_sorted_unique_effect_indices() {
        let selected: AwakeningSelection = serde_json::from_str("[6,3,1]").unwrap();
        assert_eq!(serde_json::to_string(&selected).unwrap(), "[1,3,6]");
        assert!(selected.has_effect(6));
        assert_eq!(selected.count(), 3);
        assert!(!selected.is_prefix());

        // Integer rank deserialization
        let rank3: AwakeningSelection = serde_json::from_str("3").unwrap();
        assert_eq!(rank3.count(), 3);
        assert!(rank3.is_prefix());
        assert_eq!(serde_json::to_string(&rank3).unwrap(), "[1,2,3]");

        let rank0: AwakeningSelection = serde_json::from_str("0").unwrap();
        assert_eq!(rank0.count(), 0);

        for invalid in [
            "[1,1]",
            "[0]",
            "[7]",
            "[1,2,3,4,5,6,1]",
            "[1.5]",
            "[true]",
            "7",
            "-1",
        ] {
            assert!(
                serde_json::from_str::<AwakeningSelection>(invalid).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn resonance_milestones_depend_on_count_not_effect_identity() {
        // Sparse selection [3]: has Effect3, but does NOT activate resonance_3 or resonance_6
        let single_three = AwakeningSelection::selected([3]).unwrap();
        assert!(single_three.has_effect(3));
        assert!(!single_three.has_effect(1));
        assert!(!single_three.has_resonance_3());
        assert!(!single_three.has_resonance_6());

        // Sparse selection [1, 2, 4]: does NOT have Effect3, but DOES activate resonance_3 (count == 3)
        let three_without_three = AwakeningSelection::selected([1, 2, 4]).unwrap();
        assert!(!three_without_three.has_effect(3));
        assert!(three_without_three.has_resonance_3());
        assert!(!three_without_three.has_resonance_6());

        // Selection [6]: has Effect6, but does NOT activate resonance_6 (count == 1)
        let single_six = AwakeningSelection::selected([6]).unwrap();
        assert!(single_six.has_effect(6));
        assert!(!single_six.has_resonance_6());

        // Full selection [1, 2, 3, 4, 5, 6]: activates both resonance_3 and resonance_6
        let full = AwakeningSelection::prefix(6);
        assert!(full.has_resonance_3());
        assert!(full.has_resonance_6());
    }
}
