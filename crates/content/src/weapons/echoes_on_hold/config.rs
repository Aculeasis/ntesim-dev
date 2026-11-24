use crate::gear::validation::{ConfigContext, DeadlineContext, ProcValidation};
use crate::simulation::metadata::InputActionRole;
use crate::simulation::{ExactFraction, GearProcConfig};

pub(crate) fn validate_config(
    proc: &GearProcConfig,
    ctx: &ConfigContext<'_>,
) -> Result<ProcValidation, String> {
    let GearProcConfig::EchoesOnHold {
        atk_bonus,
        team_def_ignore,
        wearer_def_ignore,
        cosmos_penetration,
        duration_us,
    } = proc
    else {
        return Err("Echoes on Hold validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    if !actor.weapon_id.eq_ignore_ascii_case(super::WEAPON_ID) {
        return Err(format!(
            "actor {i} Echoes on Hold proc requires equipped weapon:fork_elyms"
        ));
    }
    for value in [
        atk_bonus,
        team_def_ignore,
        wearer_def_ignore,
        cosmos_penetration,
    ] {
        let frac = ExactFraction::from_ratio(*value)?;
        if frac.is_zero() || frac.numerator > frac.denominator {
            return Err("Echoes on Hold requires positive ratios <= 1".into());
        }
    }
    if *duration_us == 0 {
        return Err("Echoes on Hold requires a positive duration".into());
    }
    Ok(ProcValidation {
        kind: "echoes_on_hold",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    let t = ctx.time_us;
    let i = ctx.action_index;
    if ctx.action.input_role == Some(InputActionRole::RedirectSkill) {
        t.checked_add(*duration_us)
            .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    }
    Ok(())
}
