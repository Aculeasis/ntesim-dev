use super::*;

#[derive(Clone, Debug)]
pub struct ExactBreakDamageRequest {
    pub base_break: exact::ExactFraction,
    pub intensity: exact::ExactFraction,
    pub original_cap: exact::ExactFraction,
    pub attacker_level: u32,
    pub enemy_defense: exact::ExactFraction,
    pub resistance: exact::ExactFraction,
    pub bonus: exact::ExactFraction,
    pub shred: exact::ExactFraction,
    pub ignore: exact::ExactFraction,
    pub resistance_reduction: exact::ExactFraction,
    pub special: exact::ExactFraction,
}

pub fn exact_break_damage(req: &ExactBreakDamageRequest) -> Result<exact::ExactFraction, String> {
    use exact::ExactFraction as F;
    if req.base_break.is_zero() || req.original_cap.is_zero() {
        return Err("Break base and original capacity must be positive".into());
    }
    let zone = F::new(1, 1)?
        .add(req.intensity.div(F::new(300, 1)?)?)?
        .add(req.bonus)?;
    req.base_break
        .mul(zone)?
        .mul(req.original_cap.div(F::new(3, 1)?)?)?
        .mul(exact::defense_factor(
            req.attacker_level,
            req.enemy_defense,
            req.shred,
            req.ignore,
        )?)?
        .mul(exact::resistance_factor(
            req.resistance,
            req.resistance_reduction,
            NegativeResistanceModel::StandardRational,
        )?)?
        .mul(req.special)
}

#[cfg(test)]
mod exact_tests {
    use super::*;
    use exact::ExactFraction as F;
    #[test]
    fn break_zone_uses_original_capacity_and_additive_bi_without_ordinary_stats() {
        let mut request = ExactBreakDamageRequest {
            base_break: F::new(3603, 1).unwrap(),
            intensity: F::new(0, 1).unwrap(),
            original_cap: F::new(30, 1).unwrap(),
            attacker_level: 80,
            enemy_defense: F::new(1080, 1).unwrap(),
            resistance: F::new(0, 1).unwrap(),
            bonus: F::new(1, 5).unwrap(),
            shred: F::new(0, 1).unwrap(),
            ignore: F::new(0, 1).unwrap(),
            resistance_reduction: F::new(0, 1).unwrap(),
            special: F::new(1, 1).unwrap(),
        };
        assert_eq!(
            exact_break_damage(&request).unwrap(),
            F::new(21618, 1).unwrap()
        );
        request.intensity = F::new(300, 1).unwrap();
        assert_eq!(
            exact_break_damage(&request).unwrap(),
            F::new(39633, 1).unwrap()
        );
        request.intensity = F::new(0, 1).unwrap();
        request.bonus = F::new(1, 2).unwrap();
        assert_eq!(
            exact_break_damage(&request).unwrap(),
            F::new(54045, 2).unwrap()
        );
        request.original_cap = F::new(0, 1).unwrap();
        assert!(exact_break_damage(&request).is_err());
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BreakDamageRequest {
    pub base_break: f64,
    pub intensity: f64,
    pub original_cap: f64,
    pub attacker_level: u32,
    pub enemy_defense: f64,
    pub resistance: f64,
    #[serde(default)]
    pub extra_bonus: f64,
    #[serde(default)]
    pub shred: f64,
    #[serde(default)]
    pub ignore: f64,
    #[serde(default = "default_one")]
    pub special: f64,
    #[serde(default)]
    pub negative_res_model: NegativeResistanceModel,
}

pub fn break_damage(req: &BreakDamageRequest) -> Result<f64, CombatError> {
    for (name, value) in [
        ("base_break", req.base_break),
        ("intensity", req.intensity),
        ("original_cap", req.original_cap),
        ("special", req.special),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(CombatError::NegativeBreakParameter(name.into()));
        }
    }
    if req.base_break <= 0.0 || req.original_cap <= 0.0 {
        return Err(CombatError::NegativeBreakParameter(
            "base_break and original_cap must be positive".into(),
        ));
    }
    let def_fac = defense_factor(
        req.attacker_level,
        req.enemy_defense,
        req.shred,
        req.ignore,
        DamageKind::Elemental,
    )?;
    let res_fac = resistance_factor(req.resistance, req.negative_res_model)?;
    let intensity_mult = 1.0 + req.intensity / 300.0 + req.extra_bonus;
    let cap_mult = req.original_cap / 3.0;
    let damage = req.base_break * intensity_mult * cap_mult * def_fac * res_fac * req.special;
    if !intensity_mult.is_finite() || intensity_mult < 0.0 || !damage.is_finite() {
        return Err(CombatError::NonFiniteParameter("break damage".into()));
    }
    Ok(damage)
}
