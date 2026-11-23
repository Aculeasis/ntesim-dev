use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealingSpec {
    pub ratio: f64,
    #[serde(default = "default_stat")]
    pub stat: String,
    #[serde(default)]
    pub flat: f64,
    #[serde(default)]
    pub outgoing_bonus: f64,
    #[serde(default)]
    pub incoming_bonus: f64,
    #[serde(default = "default_rounding")]
    pub rounding: Rounding,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HealingBreakdown {
    pub base_amount: f64,
    pub outgoing_bonus_multiplier: f64,
    pub incoming_bonus_multiplier: f64,
    pub raw_healing: f64,
    pub effective_healing: u64,
    pub overheal: u64,
}

pub fn evaluate_healing(
    healer_stats: &DynamicStats,
    target_current_hp: u64,
    target_max_hp: u64,
    spec: &HealingSpec,
) -> Result<HealingBreakdown, CombatError> {
    if target_max_hp == 0 {
        return Err(CombatError::ZeroHealthCapacity);
    }
    if target_current_hp > target_max_hp {
        return Err(CombatError::InvalidHealthState);
    }
    if target_current_hp == 0 {
        return Err(CombatError::DeadRecipient);
    }
    if !spec.ratio.is_finite() || !spec.flat.is_finite() || spec.ratio < 0.0 || spec.flat < 0.0 {
        return Err(CombatError::NonFiniteParameter(
            "healing ratio and flat must be nonnegative".into(),
        ));
    }
    let stat_val = healer_stats
        .get_stat(&spec.stat)
        .ok_or_else(|| CombatError::MissingStat(spec.stat.clone()))?;
    let base = spec.ratio * stat_val + spec.flat;
    let out_mult = 1.0 + healer_stats.healing_bonus + spec.outgoing_bonus;
    // Recipient bonuses are explicit in the packet; caster received-healing stats
    // must never alter another recipient's healing.
    let in_mult = 1.0 + spec.incoming_bonus;
    if !out_mult.is_finite() || !in_mult.is_finite() || out_mult < 0.0 || in_mult < 0.0 {
        return Err(CombatError::NonFiniteParameter(
            "healing bonus multiplier".into(),
        ));
    }
    let raw = base * out_mult * in_mult;
    let quantized = arithmetic::checked_quantize(raw, spec.rounding)?;

    let mut vitals = crate::state::VitalState::new(
        target_current_hp,
        target_max_hp,
        crate::state::ShieldOrder::OldestFirst,
    )
    .map_err(|_| CombatError::InvalidHealthState)?;
    let settled = vitals
        .heal(quantized)
        .map_err(|_| CombatError::DeadRecipient)?;

    Ok(HealingBreakdown {
        base_amount: base,
        outgoing_bonus_multiplier: out_mult,
        incoming_bonus_multiplier: in_mult,
        raw_healing: raw,
        effective_healing: settled.applied,
        overheal: settled.excess,
    })
}
