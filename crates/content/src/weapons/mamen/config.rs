pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::MamenFons {
        cosmos_bonus_per_stack,
        fons_per_stack,
        max_stacks,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_mamen") {
        return Err(format!(
            "actor {i} Contemplative Cat proc requires equipped weapon:fork_mamen"
        ));
    }
    if ExactFraction::from_ratio(*cosmos_bonus_per_stack)?.is_zero() {
        return Err(format!(
            "actor {i} Contemplative Cat requires a positive Cosmos bonus per stack"
        ));
    }
    if *fons_per_stack == 0 {
        return Err(format!(
            "actor {i} Contemplative Cat requires a positive fons_per_stack"
        ));
    }
    if *max_stacks == 0 {
        return Err(format!(
            "actor {i} Contemplative Cat requires a positive max_stacks"
        ));
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "mamen",
        duration_us: 0,
    })
}
