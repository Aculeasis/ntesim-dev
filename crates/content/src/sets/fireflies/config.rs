pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::FirefliesCritDamage {
        crit_damage_per_stack,
        max_stacks,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if actor.set_id.as_deref() != Some("set:Suit2") {
        return Err(format!(
            "actor {i} Fireflies proc requires equipped set:Suit2"
        ));
    }
    ExactFraction::from_ratio(*crit_damage_per_stack)?;
    if !(1..=64).contains(max_stacks) || *duration_us == 0 {
        return Err("Fireflies requires 1..=64 stacks and positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "fireflies",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    let t = ctx.time_us;
    let hit_delay_us = ctx.hit_delay_us;
    let i = ctx.action_index;
    t.checked_add(hit_delay_us)
        .and_then(|hit_t| hit_t.checked_add(*duration_us))
        .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    Ok(())
}
