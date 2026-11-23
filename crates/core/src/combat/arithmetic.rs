use super::*;

pub fn defense_base(attacker_level: u32) -> f64 {
    600.0 + 6.0 * (attacker_level as f64)
}

/// Mitigate an already-scaled integer packet using the shared exact defense rule.
/// Settlement uses the content packet policy: nearest integer, ties upward.
pub fn mitigate_integer_damage(
    base_damage: u64,
    kind: DamageKind,
    target_defense: u64,
    attacker_level: u32,
) -> Result<u64, String> {
    use super::exact::{ExactFraction, defense_factor};
    if !kind.uses_defense() {
        return Ok(base_damage);
    }
    let zero = ExactFraction::new(0, 1)?;
    let factor = defense_factor(
        attacker_level,
        ExactFraction::new(target_defense as u128, 1)?,
        zero,
        zero,
    )?;
    ExactFraction::new(base_damage as u128, 1)?
        .mul(factor)?
        .round_u64()
}

/// Computes the defense factor:
/// base / (base + target_defense * (1 - shred) * (1 - ignore)).
/// For Mental and True damage, defense factor is 1.0.
pub fn defense_factor(
    attacker_level: u32,
    target_defense: f64,
    defense_shred: f64,
    defense_ignore: f64,
    kind: DamageKind,
) -> Result<f64, CombatError> {
    if !kind.uses_defense() {
        return Ok(1.0);
    }
    if attacker_level == 0 {
        return Err(CombatError::ZeroAttackerLevel);
    }
    if !target_defense.is_finite() || target_defense < 0.0 {
        return Err(CombatError::NegativeDefense(format!("{target_defense}")));
    }
    if !defense_shred.is_finite() || !(0.0..=1.0).contains(&defense_shred) {
        return Err(CombatError::DefenseShredOutOfRange(format!(
            "{defense_shred}"
        )));
    }
    if !defense_ignore.is_finite() || !(0.0..=1.0).contains(&defense_ignore) {
        return Err(CombatError::DefenseIgnoreOutOfRange(format!(
            "{defense_ignore}"
        )));
    }
    let base = defense_base(attacker_level);
    let effective_def = target_defense * (1.0 - defense_shred) * (1.0 - defense_ignore);
    let denom = base + effective_def;
    if denom <= 0.0 {
        return Err(CombatError::NonFiniteParameter(
            "defense denominator <= 0".into(),
        ));
    }
    Ok(base / denom)
}

pub fn resistance_factor(
    resistance: f64,
    model: NegativeResistanceModel,
) -> Result<f64, CombatError> {
    if !resistance.is_finite() {
        return Err(CombatError::NonFiniteParameter("resistance".into()));
    }
    if resistance >= 0.0 {
        Ok((1.0 - resistance).max(0.0))
    } else {
        match model {
            NegativeResistanceModel::StandardRational => {
                let denom = 1.0 - resistance;
                if denom <= 0.0 {
                    return Err(CombatError::NonFiniteParameter(
                        "resistance denominator <= 0".into(),
                    ));
                }
                Ok(1.0 - resistance / denom)
            }
            NegativeResistanceModel::LinearScaled => Ok(1.0 - resistance / 1.10),
        }
    }
}

pub fn crit_factor_expected(
    crit_rate: f64,
    crit_damage: f64,
    rule: CritRule,
) -> Result<f64, CombatError> {
    if !crit_rate.is_finite() {
        return Err(CombatError::InvalidCritRate(format!("{crit_rate}")));
    }
    if !crit_damage.is_finite() {
        return Err(CombatError::InvalidCritDamage(format!("{crit_damage}")));
    }
    match rule {
        CritRule::None => Ok(1.0),
        CritRule::Fixed50 => Ok(1.0 + crit_damage * 0.5),
        CritRule::Fixed100 => Ok(1.0 + crit_damage * 1.0),
        CritRule::Normal => {
            let effective_rate = crit_rate.clamp(0.0, 1.0);
            Ok(1.0 + crit_damage * effective_rate)
        }
    }
}

pub fn crit_factor_stochastic(
    crit_rate: f64,
    crit_damage: f64,
    rule: CritRule,
    random_draw: f64,
) -> Result<(f64, bool), CombatError> {
    if !random_draw.is_finite() || !(0.0..1.0).contains(&random_draw) {
        return Err(CombatError::NonFiniteParameter(
            "random_draw must be in [0,1)".into(),
        ));
    }
    if !crit_rate.is_finite() {
        return Err(CombatError::InvalidCritRate(format!("{crit_rate}")));
    }
    if !crit_damage.is_finite() {
        return Err(CombatError::InvalidCritDamage(format!("{crit_damage}")));
    }
    match rule {
        CritRule::None => Ok((1.0, false)),
        CritRule::Fixed50 => {
            let is_crit = random_draw < 0.5;
            Ok((if is_crit { 1.0 + crit_damage } else { 1.0 }, is_crit))
        }
        CritRule::Fixed100 => Ok((1.0 + crit_damage, true)),
        CritRule::Normal => {
            let effective_rate = crit_rate.clamp(0.0, 1.0);
            let is_crit = random_draw < effective_rate;
            Ok((if is_crit { 1.0 + crit_damage } else { 1.0 }, is_crit))
        }
    }
}

fn validate_cycle_intensity(intensity: f64) -> Result<(), CombatError> {
    if !intensity.is_finite() || intensity < 0.0 {
        return Err(CombatError::NonFiniteParameter(
            "cycle intensity must be finite and nonnegative".into(),
        ));
    }
    Ok(())
}

pub fn cycle_intensity_zone(intensity: f64) -> Result<f64, CombatError> {
    validate_cycle_intensity(intensity)?;
    Ok(1.0 + intensity / 600.0)
}

pub fn cycle_effect_zone(intensity: f64) -> Result<f64, CombatError> {
    validate_cycle_intensity(intensity)?;
    Ok(1.0 + 0.2 * (intensity / (intensity + 180.0)))
}

pub fn stain_factor(intensity: f64, base: f64) -> Result<f64, CombatError> {
    if !base.is_finite() || base < 0.0 {
        return Err(CombatError::NonFiniteParameter(
            "stain base multiplier must be finite and nonnegative".into(),
        ));
    }
    let factor = base * cycle_effect_zone(intensity)?;
    if !factor.is_finite() {
        return Err(CombatError::NonFiniteParameter(
            "stain factor overflow".into(),
        ));
    }
    Ok(factor)
}

pub fn hexed_fraction(intensity: f64, base: f64) -> Result<f64, CombatError> {
    Ok(stain_factor(intensity, base)? - 1.0)
}

pub fn quantize(raw: f64, rounding: Rounding) -> u64 {
    if raw <= 0.0 {
        return 0;
    }
    match rounding {
        Rounding::Floor => raw.floor() as u64,
        Rounding::Ceil => raw.ceil() as u64,
        Rounding::NearestTiesEven => raw.round_ties_even() as u64,
    }
}

pub(crate) fn checked_quantize(raw: f64, rounding: Rounding) -> Result<u64, CombatError> {
    if !raw.is_finite() || raw < 0.0 {
        return Err(CombatError::NonFiniteParameter(
            "nonnegative finite output required".into(),
        ));
    }
    // u64::MAX rounds to 2^64 as f64; that boundary cannot be represented by u64.
    let rounded = match rounding {
        Rounding::Floor => raw.floor(),
        Rounding::Ceil => raw.ceil(),
        Rounding::NearestTiesEven => raw.round_ties_even(),
    };
    if rounded >= 18_446_744_073_709_551_616.0 {
        return Err(CombatError::QuantizationOverflow);
    }
    Ok(rounded as u64)
}
