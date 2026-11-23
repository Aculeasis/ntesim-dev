//! Exact consumed Ultimate-energy distribution, separate from Cycle energy.
use crate::simulation::ExactFraction;
use ntesim_data::character::Element;

pub(super) struct Distribution {
    pub(super) owner: ExactFraction,
    peer: ExactFraction,
}

// Global 2026-10-04 DT_SkillTeamChargeRelationData: every one of the six
// source-element rows has six recipient elements with exact coefficient .6.
// Owner identity is the existing execution model, not an element-diagonal rule.
pub(super) const DISTRIBUTION: Distribution = Distribution {
    owner: ExactFraction {
        numerator: 1,
        denominator: 1,
    },
    peer: ExactFraction {
        numerator: 3,
        denominator: 5,
    },
};

pub(super) fn peer(source: Element, recipient: Element) -> ExactFraction {
    use Element::*;
    // Exhaustive typed admission makes a future element require a source review.
    match (source, recipient) {
        (
            Chaos | Cosmos | Incantation | Lakshana | Nature | Psyche,
            Chaos | Cosmos | Incantation | Lakshana | Nature | Psyche,
        ) => DISTRIBUTION.peer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_native_element_pairs_preserve_three_fifths_including_the_diagonal() {
        use Element::*;
        let elements = [Chaos, Cosmos, Incantation, Lakshana, Nature, Psyche];
        for source in elements {
            for recipient in elements {
                assert_eq!(peer(source, recipient), ExactFraction::new(3, 5).unwrap());
            }
        }
        assert_eq!(DISTRIBUTION.owner, ExactFraction::new(1, 1).unwrap());
        assert_ne!(peer(Cosmos, Cosmos), DISTRIBUTION.owner);
    }
}
