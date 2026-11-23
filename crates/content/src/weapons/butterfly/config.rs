pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::ButterflyDamage {
        anima_bonus,
        attachment_bonus,
        ult_attachment_bonus,
        ult_duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_Butterfly")
    {
        return Err(format!(
            "actor {i} Reality Refuge proc requires equipped weapon:fork_Butterfly"
        ));
    }
    ExactFraction::from_ratio(*anima_bonus)?;
    ExactFraction::from_ratio(*attachment_bonus)?;
    ExactFraction::from_ratio(*ult_attachment_bonus)?;
    if *ult_duration_us == 0 {
        return Err("Reality Refuge requires positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "butterfly",
        duration_us: *ult_duration_us,
    })
}
