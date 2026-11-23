use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShieldSpec {
    pub ratio: f64,
    #[serde(default = "default_stat")]
    pub stat: String,
    #[serde(default)]
    pub flat: f64,
    #[serde(default)]
    pub shield_bonus: f64,
    #[serde(default = "default_rounding")]
    pub rounding: Rounding,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShieldBreakdown {
    pub base_amount: f64,
    pub shield_bonus_multiplier: f64,
    pub raw_shield: f64,
    pub effective_shield: u64,
}

pub fn evaluate_shield(
    shielder_stats: &DynamicStats,
    spec: &ShieldSpec,
) -> Result<ShieldBreakdown, CombatError> {
    if !spec.ratio.is_finite() || !spec.flat.is_finite() || spec.ratio < 0.0 || spec.flat < 0.0 {
        return Err(CombatError::NonFiniteParameter(
            "shield ratio and flat must be nonnegative".into(),
        ));
    }
    let stat_val = shielder_stats
        .get_stat(&spec.stat)
        .ok_or_else(|| CombatError::MissingStat(spec.stat.clone()))?;
    let base = spec.ratio * stat_val + spec.flat;
    let shield_mult = 1.0 + shielder_stats.shield_bonus + spec.shield_bonus;
    if !shield_mult.is_finite() || shield_mult < 0.0 {
        return Err(CombatError::NonFiniteParameter(
            "shield bonus multiplier".into(),
        ));
    }
    let raw = base * shield_mult;
    let effective = arithmetic::checked_quantize(raw, spec.rounding)?;

    Ok(ShieldBreakdown {
        base_amount: base,
        shield_bonus_multiplier: shield_mult,
        raw_shield: raw,
        effective_shield: effective,
    })
}
