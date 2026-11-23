pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::BitGame {
        off_field_atk_base,
        off_field_atk_per_stack,
        off_field_icd_us,
        on_field_psyche_base,
        on_field_psyche_per_stack,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_BitGame") {
        return Err(format!(
            "actor {i} Blow up the Crowd proc requires equipped weapon:fork_BitGame"
        ));
    }
    ExactFraction::from_ratio(*off_field_atk_base)?;
    ExactFraction::from_ratio(*off_field_atk_per_stack)?;
    ExactFraction::from_ratio(*on_field_psyche_base)?;
    ExactFraction::from_ratio(*on_field_psyche_per_stack)?;
    if *off_field_icd_us == 0 {
        return Err("Blow up the Crowd requires a positive off-field ICD".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "bit_game",
        duration_us: 0,
    })
}
