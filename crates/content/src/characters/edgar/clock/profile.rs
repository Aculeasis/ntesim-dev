//! Selected source seconds rounded to microseconds at event boundaries.
use super::*;
pub(crate) type Hit = (&'static str, u64);
const NORMAL: [&[Hit]; 5] = [
    &[("GE_Player_Edgar_Melee1_Damage", 183_333)],
    &[("GE_Player_Edgar_Melee2_Damage", 283_333)],
    &[
        ("GE_Player_Edgar_Melee3_Damage", 200_357),
        ("GE_Player_Edgar_Melee3_Damage", 433_734),
    ],
    &[("GE_Player_Edgar_Melee4_Damage", 567_506)],
    &[("GE_Player_Edgar_Melee5_Damage", 734_407)],
];
pub(super) fn plan(
    ctx: &mut CastContext<'_>,
    continued: bool,
) -> Result<(Kind, Vec<Hit>, u64), String> {
    if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str())
        != super::super::primary_packet(ctx.ability_id)
    {
        return Err("Edgar full kit requires canonical action packets".into());
    }
    let (kind, hits) = match ctx.ability_id {
        "GA_Edgar_Melee" => {
            if matches!(
                ctx.request.actions.get(ctx.command_index),
                Some(crate::simulation::GameActionCommand::SwitchActor { .. })
            ) {
                return Ok((
                    Kind::Appear,
                    vec![("GE_Player_Edgar_Melee1_Damage", 283_289)],
                    0,
                ));
            }
            let state = ctx.actor.character.edgar_mut()?;
            if !continued {
                state.normal_stage = 0;
            }
            let stage = state.normal_stage;
            state.normal_stage = (stage + 1) % 5;
            (Kind::Normal(stage), NORMAL[stage].to_vec())
        }
        SKILL => {
            let release = ctx
                .request
                .hold_inputs
                .get(&ctx.command_index)
                .map(|h| h.duration_us.min(4_900_000))
                .filter(|t| *t >= 800_000);
            let hits = if let Some(release) = release {
                (0..7)
                    .map(|k| (SKILL_PACKET, 821_980 + k * 600_000))
                    .filter(|(_, t)| *t < release)
                    .collect()
            } else {
                vec![(SKILL_PACKET, 866_099)]
            };
            ctx.actor.ability_cooldowns.remove(SKILL);
            (Kind::Skill(release), hits)
        }
        BURST => {
            ctx.actor.ability_cooldowns.insert(
                BURST.into(),
                ctx.time_us
                    .checked_add(20_000_000)
                    .ok_or("Edgar Burst cooldown overflow")?,
            );
            super::super::effects::burst_refund(ctx)?;
            (
                Kind::Burst,
                vec![("GE_Player_Edgar_UltraSkill_Damage", 2_750_956)],
            )
        }
        _ => return Err("Edgar full kit has no supported action".into()),
    };
    Ok((kind, hits, 0))
}
pub(super) fn recovery(kind: Kind) -> u64 {
    match kind {
        Kind::Normal(stage) => [1_096_206, 950_352, 1_166_324, 1_276_495, 1_335_250][stage],
        Kind::Appear => 1_108_982,
        Kind::Skill(Some(release)) => release + 430_891,
        Kind::Skill(None) => 1_350_100,
        Kind::Burst => 3_398_164,
        Kind::Plunge => 198_548,
        Kind::Riposte => 901_881,
        Kind::Support => 1_393_653,
        Kind::Dodge(back) => {
            if back {
                1_000_811
            } else {
                1_004_912
            }
        }
    }
}
pub(super) fn pause(effect: &str) -> (u64, u64) {
    match effect {
        "GE_Player_Edgar_Melee1_Damage" | "GE_Player_Edgar_Melee2_Damage" => (29_700, 0),
        "GE_Player_Edgar_Melee3_Damage" => (60_300, 40_000),
        "GE_Player_Edgar_Melee4_Damage" | "GE_Player_Edgar_Melee5_Damage" => (29_700, 33_000),
        "GE_Player_Edgar_PerfectEvadeAttack_Damage" => (297_000, 0),
        "GE_Player_Edgar_QTE_Damage" => (210_000, 0),
        _ => (0, 0),
    }
}
pub(super) fn gate(kind: Kind, input: &str) -> Option<u64> {
    Some(match (kind, input) {
        (Kind::Normal(stage), "jump") => [200_726, 313_329, 225_131, 591_627, 766_480][stage],
        (Kind::Appear, "jump") => 315_733,
        (Kind::Skill(None), "jump") => 928_624,
        (Kind::Skill(Some(release)), "jump") => release,
        (Kind::Plunge, "jump") => 82_917,
        (Kind::Riposte, "jump") => 0,
        (Kind::Dodge(back), "jump") => {
            if back {
                0
            } else {
                82_544
            }
        }
        (Kind::Dodge(back), SKILL) => {
            if back {
                428_208
            } else {
                328_189
            }
        }
        (_, "jump" | "dash" | "GA_Edgar_Melee" | SKILL | BURST) => recovery(kind),
        _ => return None,
    })
}
pub(super) fn schedule_controls(
    ctx: &mut CastContext<'_>,
    clock: &mut Clock,
    kind: Kind,
) -> Result<(), String> {
    let coordinate = ctx
        .native_array_index
        .ok_or("Edgar native talent missing")?
        .checked_add(1)
        .ok_or("Edgar talent overflow")?;
    match kind {
        Kind::Skill(release) => {
            let groups = clock.active.as_ref().unwrap().groups.clone();
            for (group, progress) in groups {
                schedule(ctx, progress, ActionEvent::Pulse { group })?;
            }
            if let Some(release) = release {
                schedule(ctx, 800_000, ActionEvent::LoopEntered)?;
                for k in 0..7 {
                    let progress = 821_711 + k * 600_000;
                    if progress < release {
                        schedule(ctx, progress, ActionEvent::Heal { coordinate })?;
                    }
                }
                let extra = additional(release - 800_000, ctx.config.awakening.has_effect(2))?;
                schedule(
                    ctx,
                    release + 96_741,
                    ActionEvent::Cooldown { additional: extra },
                )?;
            } else {
                schedule(ctx, 867_115, ActionEvent::Heal { coordinate })?;
                schedule(ctx, 849_585, ActionEvent::Cooldown { additional: 0 })?;
            }
        }
        Kind::Burst => {
            schedule(ctx, 2_750_956, ActionEvent::FieldSpawn { coordinate })?;
            ctx.actor.switch_action_locked_until_us = ctx
                .time_us
                .checked_add(3_326_474)
                .ok_or("Edgar Burst switch lock overflow")?;
        }
        Kind::Support => {
            ctx.actor.switch_action_locked_until_us = ctx
                .time_us
                .checked_add(1_336_149)
                .ok_or("Edgar Support switch lock overflow")?;
        }
        _ => {}
    }
    Ok(())
}
