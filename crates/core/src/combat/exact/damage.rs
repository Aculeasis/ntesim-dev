use super::factors::{defense_factor, direct_damage, resistance_factor};
use super::fraction::ExactFraction;

#[derive(Clone, Copy, Debug)]
pub struct ExactDirectDamageRequest {
    pub kind: crate::combat::DamageKind,
    pub attacker_level: u32,
    pub stat: ExactFraction,
    pub ratio: ExactFraction,
    pub damage_bonus: ExactFraction,
    pub target_defense: ExactFraction,
    pub defense_shred: ExactFraction,
    pub defense_ignore: ExactFraction,
    pub target_resistance: ExactFraction,
    pub resistance_reduction: ExactFraction,
    pub negative_resistance_model: crate::combat::NegativeResistanceModel,
    pub final_multiplier: ExactFraction,
    pub special_multiplier: ExactFraction,
    pub crit_bonus: ExactFraction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExactDirectDamageBreakdown {
    pub defense_factor: ExactFraction,
    pub resistance_factor: ExactFraction,
    pub non_crit: ExactFraction,
    pub critical: ExactFraction,
    pub crit_eligible: bool,
}

/// Uses the same damage-kind zone policy as `evaluate_direct_damage`.
/// Sampling and HP settlement remain separate transitions owned by the caller.
pub fn evaluate_direct_damage(
    req: &ExactDirectDamageRequest,
) -> Result<ExactDirectDamageBreakdown, String> {
    let one = ExactFraction::new(1, 1)?;
    let standard = req.kind.uses_standard_modifiers();
    let defense = if req.kind.uses_defense() {
        defense_factor(
            req.attacker_level,
            req.target_defense,
            req.defense_shred,
            req.defense_ignore,
        )?
    } else {
        one
    };
    let resistance = if standard {
        resistance_factor(
            req.target_resistance,
            req.resistance_reduction,
            req.negative_resistance_model,
        )?
    } else {
        one
    };
    let bonus = if standard {
        req.damage_bonus
    } else {
        ExactFraction::new(0, 1)?
    };
    let final_multiplier = if standard { req.final_multiplier } else { one };
    let non_crit = direct_damage(
        req.stat,
        req.ratio,
        bonus,
        defense
            .mul(resistance)?
            .mul(final_multiplier)?
            .mul(req.special_multiplier)?,
        None,
    )?;
    let critical = if standard {
        non_crit.mul(one.add(req.crit_bonus)?)?
    } else {
        non_crit
    };
    Ok(ExactDirectDamageBreakdown {
        defense_factor: defense,
        resistance_factor: resistance,
        non_crit,
        critical,
        crit_eligible: standard,
    })
}
