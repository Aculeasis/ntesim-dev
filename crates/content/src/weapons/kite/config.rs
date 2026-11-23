pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::KiteAtk {
        atk_bonus,
        lakshana_bonus,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_Kite") {
        return Err(format!(
            "actor {i} Kite proc requires equipped weapon:fork_Kite"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    ExactFraction::from_ratio(*lakshana_bonus)?;
    if *duration_us == 0 {
        return Err("Watch Your Heads! requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "kite",
        duration_us: *duration_us,
    })
}
