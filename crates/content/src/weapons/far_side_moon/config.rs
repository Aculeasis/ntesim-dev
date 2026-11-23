pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::FarSideMoon {
        crit_rate_bonus,
        skill_damage_per_stack,
        basic_attack_damage_bonus,
        lakshana_damage_bonus,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_prism") {
        return Err(format!(
            "actor {i} Far Side of the Moon proc requires equipped weapon:fork_prism"
        ));
    }
    for value in [
        *crit_rate_bonus,
        *skill_damage_per_stack,
        *basic_attack_damage_bonus,
        *lakshana_damage_bonus,
    ] {
        ExactFraction::from_ratio(value)?;
    }
    if *duration_us == 0 {
        return Err("Far Side of the Moon requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "far_side_moon",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    let action = ctx.action;
    let t = ctx.time_us;
    let i = ctx.action_index;
    if action.input_role == Some(crate::simulation::metadata::InputActionRole::RedirectSkill) {
        t.checked_add(*duration_us)
            .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    }
    Ok(())
}
