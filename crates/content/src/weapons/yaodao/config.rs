pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::YaodaoParry { parry_damage_ratio } = proc else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_yaodao") {
        return Err(format!(
            "actor {i} Crimson Hexblade proc requires equipped weapon:fork_yaodao"
        ));
    }
    if ExactFraction::from_ratio(*parry_damage_ratio)?.is_zero() {
        return Err("Crimson Hexblade requires a positive parry damage ratio".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "yaodao_parry",
        duration_us: 0,
    })
}
