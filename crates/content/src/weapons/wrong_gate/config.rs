pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::WrongGate {
        atk_bonus,
        anima_damage_bonus,
        ally_damage_bonus,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_Door") {
        return Err(format!(
            "actor {i} The Wrong Gate proc requires equipped weapon:fork_Door"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    ExactFraction::from_ratio(*anima_damage_bonus)?;
    ExactFraction::from_ratio(*ally_damage_bonus)?;
    if *duration_us == 0 {
        return Err("The Wrong Gate requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "the_wrong_gate",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    let action = ctx.action;
    let t = ctx.time_us;
    let hit_delay_us = ctx.hit_delay_us;
    let i = ctx.action_index;
    if action.input_role == Some(crate::simulation::metadata::InputActionRole::Ultimate) {
        t.checked_add(hit_delay_us)
            .and_then(|hit_t| hit_t.checked_add(*duration_us))
            .ok_or_else(|| format!("action {i} The Wrong Gate deadline overflow"))?;
    }
    Ok(())
}
