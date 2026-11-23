pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::JiaojuanBreak {
        break_intensity_bonus,
        broken_damage_bonus,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_jiaojuan") {
        return Err(format!(
            "actor {i} Film - MANISH proc requires equipped weapon:fork_jiaojuan"
        ));
    }
    ExactFraction::from_ratio(*break_intensity_bonus)?;
    ExactFraction::from_ratio(*broken_damage_bonus)?;
    Ok(crate::gear::validation::ProcValidation {
        kind: "jiaojuan",
        duration_us: 0,
    })
}
