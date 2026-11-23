pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::GoldWool {
        lakshana_bonus,
        crit_damage_bonus,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_GoldWool") {
        return Err(format!(
            "actor {i} Gold Wool proc requires equipped weapon:fork_GoldWool"
        ));
    }
    ExactFraction::from_ratio(*lakshana_bonus)?;
    ExactFraction::from_ratio(*crit_damage_bonus)?;
    if *duration_us == 0 {
        return Err("Gold Wool requires positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "gold_wool",
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
    if matches!(
        action.input_role,
        Some(
            crate::simulation::metadata::InputActionRole::RedirectSkill
                | crate::simulation::metadata::InputActionRole::Ultimate
        )
    ) {
        t.checked_add(*duration_us)
            .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    }
    Ok(())
}
