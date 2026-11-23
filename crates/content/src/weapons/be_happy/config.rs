pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::BeHappy {
        heal_ratio,
        cooldown_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_vine") {
        return Err(format!(
            "actor {i} Be Happy proc requires equipped weapon:fork_vine"
        ));
    }
    ExactFraction::from_ratio(*heal_ratio)?;
    if *cooldown_us == 0 {
        return Err("Be Happy requires a positive cooldown".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "be_happy",
        duration_us: *cooldown_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    cooldown_us: &u64,
) -> Result<(), String> {
    let t = ctx.time_us;
    let hit_delay_us = ctx.hit_delay_us;
    let i = ctx.action_index;
    t.checked_add(hit_delay_us)
        .and_then(|hit_t| hit_t.checked_add(*cooldown_us))
        .ok_or_else(|| format!("action {i} Be Happy deadline overflow"))?;
    Ok(())
}
