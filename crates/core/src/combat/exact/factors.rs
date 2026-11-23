use super::fraction::{ExactArithmeticError, ExactFraction};

pub fn try_defense_factor(
    level: u32,
    defense: ExactFraction,
    shred: ExactFraction,
    ignore: ExactFraction,
) -> Result<ExactFraction, ExactArithmeticError> {
    if level == 0 {
        return Err(ExactArithmeticError::ZeroAttackerLevel);
    }
    let one = ExactFraction::try_new(1, 1)?;
    if shred.numerator > shred.denominator || ignore.numerator > ignore.denominator {
        return Err(ExactArithmeticError::ShredOutOfRange);
    }
    let base = ExactFraction::try_new(600 + 6 * u128::from(level), 1)?;
    let effective = defense
        .try_mul(one.try_sub(shred)?)?
        .try_mul(one.try_sub(ignore)?)?;
    base.try_div(base.try_add(effective)?)
}

pub fn defense_factor(
    level: u32,
    defense: ExactFraction,
    shred: ExactFraction,
    ignore: ExactFraction,
) -> Result<ExactFraction, String> {
    try_defense_factor(level, defense, shred, ignore).map_err(Into::into)
}

pub fn try_resistance_factor(
    resistance: ExactFraction,
    reduction: ExactFraction,
    model: crate::combat::NegativeResistanceModel,
) -> Result<ExactFraction, ExactArithmeticError> {
    let one = ExactFraction::try_new(1, 1)?;
    let lhs = resistance
        .numerator
        .checked_mul(reduction.denominator)
        .ok_or(ExactArithmeticError::ResistanceComparisonOverflow)?;
    let rhs = reduction
        .numerator
        .checked_mul(resistance.denominator)
        .ok_or(ExactArithmeticError::ResistanceComparisonOverflow)?;
    if lhs >= rhs {
        let positive = resistance.try_sub(reduction)?;
        if positive.numerator >= positive.denominator {
            ExactFraction::try_new(0, 1)
        } else {
            one.try_sub(positive)
        }
    } else {
        let magnitude = reduction.try_sub(resistance)?;
        let denominator = match model {
            crate::combat::NegativeResistanceModel::StandardRational => one.try_add(magnitude)?,
            crate::combat::NegativeResistanceModel::LinearScaled => ExactFraction::try_new(11, 10)?,
        };
        one.try_add(magnitude.try_div(denominator)?)
    }
}

pub fn resistance_factor(
    resistance: ExactFraction,
    reduction: ExactFraction,
    model: crate::combat::NegativeResistanceModel,
) -> Result<ExactFraction, String> {
    try_resistance_factor(resistance, reduction, model).map_err(Into::into)
}

pub fn try_direct_damage(
    stat: ExactFraction,
    ratio: ExactFraction,
    bonus: ExactFraction,
    mitigation: ExactFraction,
    crit_bonus: Option<ExactFraction>,
) -> Result<ExactFraction, ExactArithmeticError> {
    let one = ExactFraction::try_new(1, 1)?;
    let result = stat
        .try_mul(ratio)?
        .try_mul(one.try_add(bonus)?)?
        .try_mul(mitigation)?;
    match crit_bonus {
        Some(crit) => result.try_mul(one.try_add(crit)?),
        None => Ok(result),
    }
}

pub fn direct_damage(
    stat: ExactFraction,
    ratio: ExactFraction,
    bonus: ExactFraction,
    mitigation: ExactFraction,
    crit_bonus: Option<ExactFraction>,
) -> Result<ExactFraction, String> {
    try_direct_damage(stat, ratio, bonus, mitigation, crit_bonus).map_err(Into::into)
}

pub fn try_exact_healing(
    base_heal: ExactFraction,
    healing_bonus: ExactFraction,
    incoming_healing_bonus: ExactFraction,
) -> Result<ExactFraction, ExactArithmeticError> {
    let one = ExactFraction::try_new(1, 1)?;
    base_heal
        .try_mul(one.try_add(healing_bonus)?)?
        .try_mul(one.try_add(incoming_healing_bonus)?)
}

pub fn exact_healing(
    base_heal: ExactFraction,
    healing_bonus: ExactFraction,
    incoming_healing_bonus: ExactFraction,
) -> Result<ExactFraction, String> {
    try_exact_healing(base_heal, healing_bonus, incoming_healing_bonus).map_err(Into::into)
}
