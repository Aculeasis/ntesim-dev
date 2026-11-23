pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::RishiAtk {
        atk_bonus,
        eclipse_duration_us,
        defeat_energy,
        max_defeats,
        arc_cooldown_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_rishi") {
        return Err(format!(
            "actor {i} Eclipse proc requires equipped weapon:fork_rishi"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    if *eclipse_duration_us == 0 {
        return Err(format!(
            "actor {i} Eclipse requires a positive eclipse field duration"
        ));
    }
    if *defeat_energy == 0 {
        return Err(format!("actor {i} Eclipse requires positive defeat energy"));
    }
    if *max_defeats == 0 {
        return Err(format!("actor {i} Eclipse requires a positive defeat cap"));
    }
    if *arc_cooldown_us == 0 {
        return Err(format!(
            "actor {i} Eclipse requires a positive arc cooldown"
        ));
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "rishi_atk",
        duration_us: *eclipse_duration_us,
    })
}
