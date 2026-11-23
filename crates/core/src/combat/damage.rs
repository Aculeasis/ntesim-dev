use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectHitSpec {
    #[serde(default)]
    pub base: Option<f64>,
    #[serde(default)]
    pub ratio: Option<f64>,
    #[serde(default = "default_stat")]
    pub stat: String,
    #[serde(default)]
    pub flat: f64,
    #[serde(default = "default_element")]
    pub element: String,
    #[serde(default = "default_damage_kind")]
    pub kind: DamageKind,
    #[serde(default = "default_crit_rule")]
    pub crit_rule: CritRule,
    #[serde(default)]
    pub action_type: Option<String>,
    #[serde(default)]
    pub damage_bonus: f64,
    #[serde(default)]
    pub crit_damage_bonus: f64,
    #[serde(default = "default_one")]
    pub final_multiplier: f64,
    #[serde(default = "default_one")]
    pub special_multiplier: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DirectDamageBreakdown {
    pub base_damage: f64,
    pub scaling_stat_name: String,
    pub scaling_stat_value: f64,
    pub ratio: f64,
    pub flat: f64,
    pub damage_bonus_total: f64,
    pub damage_bonus_multiplier: f64,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub crit_factor: f64,
    pub crit_hit: Option<bool>,
    pub defense_factor: f64,
    pub effective_enemy_defense: f64,
    pub resistance_factor: f64,
    pub vulnerability_multiplier: f64,
    pub final_multiplier: f64,
    pub special_multiplier: f64,
    pub raw_damage: f64,
    pub quantized_damage: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EvaluationCritMode {
    Expected,
    Stochastic(f64), // random draw in [0.0, 1.0)
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectDamageEvaluationRequest<'a> {
    pub attacker_level: u32,
    pub attacker_stats: &'a DynamicStats,
    pub target_defense: f64,
    pub target_resistance: f64,
    pub target_vulnerability: f64,
    pub hit: &'a DirectHitSpec,
    pub crit_mode: EvaluationCritMode,
    pub rounding: Rounding,
    pub negative_res_model: NegativeResistanceModel,
}

pub fn evaluate_direct_damage(
    req: &DirectDamageEvaluationRequest<'_>,
) -> Result<DirectDamageBreakdown, CombatError> {
    let hit = req.hit;
    let attacker_stats = req.attacker_stats;
    let rounding = req.rounding;
    for (name, value) in [
        ("base", hit.base.unwrap_or(0.0)),
        ("ratio", hit.ratio.unwrap_or(1.0)),
        ("flat", hit.flat),
        ("final_multiplier", hit.final_multiplier),
        ("special_multiplier", hit.special_multiplier),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(CombatError::NonFiniteParameter(name.into()));
        }
    }

    if !hit.kind.uses_standard_modifiers() {
        let stat_val = attacker_stats
            .get_stat(&hit.stat)
            .ok_or_else(|| CombatError::MissingStat(hit.stat.clone()))?;
        if !stat_val.is_finite() {
            return Err(CombatError::NonFiniteParameter(hit.stat.clone()));
        }
        let base_dmg = if let Some(b) = hit.base {
            b
        } else {
            hit.ratio.unwrap_or(1.0) * stat_val + hit.flat
        };
        let raw = base_dmg * hit.special_multiplier;
        let quantized = arithmetic::checked_quantize(raw, rounding)?;
        return Ok(DirectDamageBreakdown {
            base_damage: base_dmg,
            scaling_stat_name: hit.stat.clone(),
            scaling_stat_value: stat_val,
            ratio: hit.ratio.unwrap_or(1.0),
            flat: hit.flat,
            damage_bonus_total: 0.0,
            damage_bonus_multiplier: 1.0,
            crit_rate: 0.0,
            crit_damage: 0.0,
            crit_factor: 1.0,
            crit_hit: None,
            defense_factor: 1.0,
            effective_enemy_defense: 0.0,
            resistance_factor: 1.0,
            vulnerability_multiplier: 1.0,
            final_multiplier: 1.0,
            special_multiplier: hit.special_multiplier,
            raw_damage: raw,
            quantized_damage: quantized,
        });
    }

    let stat_val = attacker_stats
        .get_stat(&hit.stat)
        .ok_or_else(|| CombatError::MissingStat(hit.stat.clone()))?;
    if !stat_val.is_finite() {
        return Err(CombatError::NonFiniteParameter(hit.stat.clone()));
    }
    let ratio = hit.ratio.unwrap_or(1.0);
    let base_damage = if let Some(b) = hit.base {
        b
    } else {
        ratio * stat_val + hit.flat
    };

    let mut total_dmg_bonus = attacker_stats.damage_bonus + hit.damage_bonus;
    let damage_channel = match hit.kind {
        DamageKind::Mental => "Mental",
        _ => &hit.element,
    };
    if let Some(el_bonus) = attacker_stats.elemental_bonuses.get(damage_channel) {
        total_dmg_bonus += el_bonus;
    }
    if let Some(act) = &hit.action_type
        && let Some(act_bonus) = attacker_stats.action_type_bonuses.get(act)
    {
        total_dmg_bonus += act_bonus;
    }
    let dmg_bonus_mult = 1.0 + total_dmg_bonus;
    if !dmg_bonus_mult.is_finite() || dmg_bonus_mult < 0.0 {
        return Err(CombatError::NegativeDamageBonus(format!(
            "{total_dmg_bonus}"
        )));
    }

    let total_crit_damage = attacker_stats.crit_damage + hit.crit_damage_bonus;
    let (crit_fac, crit_hit) = match req.crit_mode {
        EvaluationCritMode::Expected => (
            crit_factor_expected(attacker_stats.crit_rate, total_crit_damage, hit.crit_rule)?,
            None,
        ),
        EvaluationCritMode::Stochastic(draw) => {
            let (fac, hit_bool) = crit_factor_stochastic(
                attacker_stats.crit_rate,
                total_crit_damage,
                hit.crit_rule,
                draw,
            )?;
            (fac, Some(hit_bool))
        }
    };

    let def_fac = defense_factor(
        req.attacker_level,
        req.target_defense,
        attacker_stats.defense_shred,
        attacker_stats.defense_ignore,
        hit.kind,
    )?;
    let effective_def = req.target_defense
        * (1.0 - attacker_stats.defense_shred)
        * (1.0 - attacker_stats.defense_ignore);

    let res_fac = resistance_factor(req.target_resistance, req.negative_res_model)?;

    let vuln_mult = 1.0 + attacker_stats.vulnerability + req.target_vulnerability;
    if !vuln_mult.is_finite() || vuln_mult < 0.0 {
        return Err(CombatError::NonFiniteParameter(
            "vulnerability multiplier".into(),
        ));
    }

    let raw_damage = base_damage
        * dmg_bonus_mult
        * crit_fac
        * def_fac
        * res_fac
        * vuln_mult
        * hit.final_multiplier
        * hit.special_multiplier;

    let quantized = arithmetic::checked_quantize(raw_damage, rounding)?;

    Ok(DirectDamageBreakdown {
        base_damage,
        scaling_stat_name: hit.stat.clone(),
        scaling_stat_value: stat_val,
        ratio,
        flat: hit.flat,
        damage_bonus_total: total_dmg_bonus,
        damage_bonus_multiplier: dmg_bonus_mult,
        crit_rate: attacker_stats.crit_rate,
        crit_damage: total_crit_damage,
        crit_factor: crit_fac,
        crit_hit,
        defense_factor: def_fac,
        effective_enemy_defense: effective_def,
        resistance_factor: res_fac,
        vulnerability_multiplier: vuln_mult,
        final_multiplier: hit.final_multiplier,
        special_multiplier: hit.special_multiplier,
        raw_damage,
        quantized_damage: quantized,
    })
}
