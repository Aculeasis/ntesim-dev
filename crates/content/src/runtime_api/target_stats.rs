use crate::simulation::{ExactFraction, state::TargetRuntime};

pub(crate) fn target_stat_modifier(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    target_index: usize,
    now_us: u64,
    stat: &str,
) -> Result<ExactFraction, String> {
    let stats = buffs
        .get_stat_modifiers_for_target(&target_index.to_string(), now_us)
        .map_err(|e| format!("target stat query failed: {e:?}"))?;
    ExactFraction::from_ratio(
        stats
            .get(stat)
            .copied()
            .unwrap_or(crate::formula::Ratio::zero()),
    )
}

pub(crate) fn target_defense(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    target: &TargetRuntime,
    target_index: usize,
    now_us: u64,
) -> Result<ExactFraction, String> {
    let reduction = target_stat_modifier(buffs, target_index, now_us, "def_percent_reduction")?;
    let reduction = ExactFraction::new(target.base_defense.unwrap_or(target.defense).into(), 1)?
        .mul(reduction)?;
    let total = ExactFraction::new(target.defense.into(), 1)?;
    let threshold = u128::from(target.defense)
        .checked_mul(reduction.denominator)
        .ok_or("target DEF comparison overflow")?;
    if reduction.numerator >= threshold {
        ExactFraction::new(0, 1)
    } else {
        total.sub(reduction)
    }
}
