//! Finite action selection, separate from scheduling/cancellation.
use super::*;
pub(super) fn select(
    ctx: &mut CastContext<'_>,
    charged: bool,
    continued: bool,
) -> Result<Plan, String> {
    let secondary = if super::super::secondary::accepted(ctx.request) {
        super::super::secondary::select(ctx)?
    } else {
        None
    };
    Ok(if let Some(plan) = secondary {
        plan
    } else {
        match ctx.ability_id {
            "GA_Haniel_Melee" => {
                let state = ctx.actor.character.haniel_mut()?;
                if !continued {
                    state.normal_stage = 0;
                }
                let stage = state.normal_stage;
                if charged && stage != 0 {
                    return Err(
                    "Haniel Cannon charge requires Normal1/entry rather than a queued later stage"
                        .into(),
                );
                }
                state.normal_stage = if charged { 0 } else { (stage + 1) % 5 };
                let entry = matches!(
                    ctx.request.actions.get(ctx.command_index),
                    Some(GameActionCommand::SwitchActor { .. })
                );
                (
                    if charged {
                        Kind::Charge
                    } else if entry {
                        Kind::Entry
                    } else {
                        Kind::Normal(stage)
                    },
                    NORMAL[stage].to_vec(),
                    if charged {
                        1_211_743
                    } else if entry {
                        896_381
                    } else {
                        RECOVERY[stage]
                    },
                    0,
                )
            }
            SKILL => (
                Kind::Skill,
                vec![
                    ("GE_Player_Haniel_Skill1_Damage", 345_126),
                    ("GE_Player_Haniel_Skill1_Damage", 719_512),
                ],
                1_309_354,
                0,
            ),
            BURST => (
                Kind::Burst,
                vec![("GE_Player_Haniel_Ultraskill1_Damage", 3_843_514)],
                4_240_809,
                0,
            ),
            _ => return Err("Haniel ordinary clock has no binding for this action".into()),
        }
    })
}
