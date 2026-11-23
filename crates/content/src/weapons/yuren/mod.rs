use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsYurenV1;

pub(crate) fn grant_time_shield_bonus(
    actor: &crate::simulation::GameActorConfig,
    current_hp: u64,
    max_hp: u64,
    granter_index: usize,
    recipient_index: usize,
) -> Result<crate::simulation::ExactFraction, String> {
    use crate::simulation::{ExactFraction, GearProcConfig};

    let Some(GearProcConfig::YurenHp { shield_bonus, .. }) = actor
        .gear_procs
        .iter()
        .find(|proc| matches!(proc, GearProcConfig::YurenHp { .. }))
    else {
        return ExactFraction::new(0, 1);
    };
    if granter_index != recipient_index {
        return Err("Rainman shield-strength grant to a foreign recipient is unsupported by the frozen wearer-only description".into());
    }
    if current_hp == 0 {
        return ExactFraction::new(0, 1);
    }
    if max_hp == 0 {
        return Err("Rainman shield grant has a living wearer with zero effective Max HP".into());
    }
    if u128::from(current_hp) * 2 <= u128::from(max_hp) {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::from_ratio(*shield_bonus)
}
