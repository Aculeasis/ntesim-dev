pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::MindRoyaleEnergy {
        energy_grant,
        cooldown_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    let limits = ctx.limits;
    if actor.weapon_id != "weapon:fork_lingganzhongjiezhe" {
        return Err(format!(
            "actor {i} Mind Royale proc requires equipped weapon:fork_lingganzhongjiezhe"
        ));
    }
    if *energy_grant > limits.max_pool_capacity {
        return Err("Mind Royale energy grant exceeds execution limit".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "mind_royale",
        duration_us: *cooldown_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    cooldown_us: &u64,
) -> Result<(), String> {
    let action = ctx.action;
    let t = ctx.time_us;
    let i = ctx.action_index;
    if action.input_role == Some(crate::simulation::metadata::InputActionRole::RedirectSkill) {
        t.checked_add(*cooldown_us)
            .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    }
    Ok(())
}
