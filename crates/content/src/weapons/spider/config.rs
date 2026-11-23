pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::SpiderTeamAtk {
        atk_bonus_per_stack,
        atk_bonus_max_stack,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_spider") {
        return Err(format!(
            "actor {i} Failing You, Heavy in My Heart proc requires equipped weapon:fork_spider"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus_per_stack)?;
    ExactFraction::from_ratio(*atk_bonus_max_stack)?;
    if *duration_us == 0 {
        return Err("Failing You, Heavy in My Heart requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "spider",
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
    if action.input_role == Some(crate::simulation::metadata::InputActionRole::Ultimate) {
        t.checked_add(*duration_us)
            .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    }
    Ok(())
}
