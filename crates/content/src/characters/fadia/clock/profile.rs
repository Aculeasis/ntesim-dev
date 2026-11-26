//! Reached ordinary montage coordinates and the consumed portions of source rate curves.
use ntesim_core::kernel::timing::HitlagRatePoint;
pub(super) type Hit = (&'static str, u64);
pub(super) const NORMAL: [&[Hit]; 5] = [
    &[("GE_Player_Fadia_Melee1_Damage", 316667)],
    &[
        ("GE_Player_Fadia_Melee2_Damage", 350000),
        ("GE_Player_Fadia_Melee3_Damage", 750000),
    ],
    &[("GE_Player_Fadia_Melee4_Damage", 583333)],
    &[
        ("GE_Player_Fadia_Melee5_Damage", 266667),
        ("GE_Player_Fadia_Melee6_Damage", 566667),
    ],
    &[
        ("GE_Player_Fadia_Melee7_Damage", 313870),
        ("GE_Player_Fadia_Melee8_Damage", 916135),
    ],
];
pub(super) const RECOVERY: [u64; 5] = [1084427, 1672247, 1452898, 1533110, 1616355];
pub(super) const SKILL: &[Hit] = &[
    ("GE_Player_Fadia_Skill3_Damage", 215663),
    ("GE_Player_Fadia_Skill1_Damage", 728626),
    ("GE_Player_Fadia_Skill2_Damage", 906368),
];
pub(super) const BURST: &[Hit] = &[
    ("GE_Player_Fadia_UltraSkill1_Damage", 2707353),
    ("GE_Player_Fadia_UltraSkill2_Damage", 3101401),
    ("GE_Player_Fadia_UltraSkill2_Damage", 3226261),
    ("GE_Player_Fadia_UltraSkill2_Damage", 3352828),
    ("GE_Player_Fadia_UltraSkill2_Damage", 3483887),
    ("GE_Player_Fadia_UltraSkill3_Damage", 3893480),
];
pub(super) const LILITH: [&[Hit]; 5] = [
    &[("GE_Player_Fadia_UltraSkillMelee1_Damage", 166271)],
    &[
        ("GE_Player_Fadia_UltraSkillMelee2_Damage", 99251),
        ("GE_Player_Fadia_UltraSkillMelee3_Damage", 449960),
    ],
    &[
        ("GE_Player_Fadia_UltraSkillMelee4_Damage", 199372),
        ("GE_Player_Fadia_UltraSkillMelee4_Damage", 283247),
        ("GE_Player_Fadia_UltraSkillMelee5_Damage", 466929),
    ],
    &[
        ("GE_Player_Fadia_UltraSkillMelee6_Damage", 367187),
        ("GE_Player_Fadia_UltraSkillMelee6_Damage", 467940),
        ("GE_Player_Fadia_UltraSkillMelee6_Damage", 566083),
        ("GE_Player_Fadia_UltraSkillMelee6_Damage", 665742),
        ("GE_Player_Fadia_UltraSkillMelee6_Damage", 766782),
        ("GE_Player_Fadia_UltraSkillMelee7_Damage", 898644),
    ],
    &[("GE_Player_Fadia_UltraSkillMelee8_Damage", 329640)],
];
pub(super) const LILITH_RECOVERY: [u64; 5] = [1082411, 616251, 1316099, 1342477, 675917];
const CURVE1: &[HitlagRatePoint] = &[
    HitlagRatePoint {
        offset_us: 0,
        rate_ppm: 0,
    },
    HitlagRatePoint {
        offset_us: 60000,
        rate_ppm: 0,
    },
    HitlagRatePoint {
        offset_us: 120000,
        rate_ppm: 10000,
    },
    HitlagRatePoint {
        offset_us: 160000,
        rate_ppm: 1000000,
    },
];
const CURVE4: &[HitlagRatePoint] = &[
    HitlagRatePoint {
        offset_us: 0,
        rate_ppm: 10000,
    },
    HitlagRatePoint {
        offset_us: 130000,
        rate_ppm: 10000,
    },
    HitlagRatePoint {
        offset_us: 140000,
        rate_ppm: 1000000,
    },
];
pub(super) enum Pause {
    Flat {
        duration: u64,
        delay: u64,
        rate_ppm: u32,
    },
    Curve {
        duration: u64,
        points: &'static [HitlagRatePoint],
    },
}
pub(super) fn pause(effect: &str) -> Option<Pause> {
    Some(match effect {
        "GE_Player_Fadia_UltraSkillMelee1_Damage"
        | "GE_Player_Fadia_UltraSkillMelee2_Damage"
        | "GE_Player_Fadia_UltraSkillMelee3_Damage"
        | "GE_Player_Fadia_UltraSkillMelee6_Damage" => Pause::Flat {
            duration: 20000,
            delay: 0,
            rate_ppm: 10000,
        },
        "GE_Player_Fadia_UltraSkillMelee5_Damage" => Pause::Flat {
            duration: 100000,
            delay: 0,
            rate_ppm: 10000,
        },
        "GE_Player_Fadia_UltraSkillMelee7_Damage" => Pause::Flat {
            duration: 150000,
            delay: 60000,
            rate_ppm: 20000,
        },
        "GE_Player_Fadia_UltraSkillMelee8_Damage" => Pause::Flat {
            duration: 200000,
            delay: 60000,
            rate_ppm: 20000,
        },
        "GE_Player_Fadia_Skill3_Damage" => Pause::Flat {
            duration: 50000,
            delay: 0,
            rate_ppm: 10000,
        },
        "GE_Player_Fadia_Skill2_Damage" => Pause::Flat {
            duration: 100000,
            delay: 40000,
            rate_ppm: 10000,
        },
        "GE_Player_Fadia_Melee1_Damage" => Pause::Curve {
            duration: 130000,
            points: CURVE1,
        },
        // Their native curve segments are constant throughout these reached windows.
        "GE_Player_Fadia_Melee2_Damage" => Pause::Flat {
            duration: 100000,
            delay: 0,
            rate_ppm: 0,
        },
        "GE_Player_Fadia_Melee3_Damage" => Pause::Flat {
            duration: 100000,
            delay: 0,
            rate_ppm: 10000,
        },
        "GE_Player_Fadia_Melee4_Damage" => Pause::Curve {
            duration: 140000,
            points: CURVE4,
        },
        "GE_Player_Fadia_Melee5_Damage" => Pause::Curve {
            duration: 100000,
            points: CURVE1,
        },
        "GE_Player_Fadia_Melee6_Damage" => Pause::Flat {
            duration: 80000,
            delay: 0,
            rate_ppm: 0,
        },
        "GE_Player_Fadia_Melee7_Damage" => Pause::Flat {
            duration: 100000,
            delay: 90000,
            rate_ppm: 10000,
        },
        "GE_Player_Fadia_Melee8_Damage" => Pause::Flat {
            duration: 200000,
            delay: 60000,
            rate_ppm: 10000,
        },
        "GE_Player_Fadia_AirAttack_Damage" => Pause::Flat {
            duration: 100000,
            delay: 90000,
            rate_ppm: 10000,
        },
        "GE_Player_Fadia_PerfectEvadeAttack_Damage" => Pause::Flat {
            duration: 300000,
            delay: 0,
            rate_ppm: 10000,
        },
        "GE_Player_Fadia_QTE1_Damage" => Pause::Flat {
            duration: 250000,
            delay: 0,
            rate_ppm: 0,
        },
        _ => return None,
    })
}

pub(crate) struct Plan {
    pub hits: &'static [Hit],
    pub recovery: u64,
    pub skill: bool,
    pub enhanced: bool,
    pub a5: bool,
    pub jump: u64,
    pub skill_gate: u64,
    pub landing: u64,
}
pub(super) fn select(
    ctx: &mut crate::runtime_api::CastContext<'_>,
    full: bool,
) -> Result<Plan, String> {
    if full && let Some(plan) = super::super::secondary::select(ctx)? {
        return Ok(plan);
    }
    // Admission captured vitals before any gear request or self heal.
    let (hp, max_hp) = ctx.team_hp_snapshot[ctx.actor_index];
    let enhanced_burst =
        max_hp.saturating_sub(hp) > super::super::effects::threshold(ctx.config.character_level);
    let state = ctx.actor.character.fadia_mut()?;
    state.normal_stage = crate::characters::normal::after_gap(
        state.normal_stage,
        state.normal_started_us,
        ctx.time_us,
    );
    let mut enhanced = false;
    let mut a5 = false;
    let mut lilith = false;
    let (hits, recovery, skill, jump, skill_gate) = match ctx.ability_id {
        "GA_Fadia_Melee"
            if full
                && state
                    .lilith_until_us
                    .is_some_and(|until| ctx.time_us < until) =>
        {
            let stage = state.lilith_stage;
            state.lilith_stage += 1;
            if state.lilith_stage == 5 {
                state.lilith_until_us = None;
            }
            enhanced = state.high_q;
            a5 = ctx.config.awakening.has_effect(5);
            lilith = true;
            (
                LILITH[stage],
                LILITH_RECOVERY[stage],
                false,
                [216877, 500335, 516491, 1033592, 347204][stage],
                match stage {
                    0 => 349804,
                    2 => 773784,
                    _ => LILITH_RECOVERY[stage],
                },
            )
        }
        "GA_Fadia_Melee" => {
            let stage = state.normal_stage;
            state.normal_stage = (stage + 1) % 5;
            (
                NORMAL[stage],
                RECOVERY[stage],
                false,
                [368655, 380012, 631545, 314679, 966021][stage],
                [601789, 987857, 752737, 997778, 1616355][stage],
            )
        }
        "GA_Fadia_Skill" if full => {
            state.normal_stage = 0;
            state.lilith_until_us = None;
            (SKILL, 1246059, true, 977466, 1246059)
        }
        "GA_Fadia_UltraSkill" if full => {
            state.normal_stage = 0;
            state.lilith_stage = 0;
            state.lilith_until_us = Some(
                ctx.time_us
                    .checked_add(9420000)
                    .ok_or("Fadia Lilith deadline overflow")?,
            );
            state.high_q = enhanced_burst;
            enhanced = enhanced_burst;
            a5 = ctx.config.awakening.has_effect(5);
            (BURST, 4415921, false, 4415921, 4415921)
        }
        _ => return Err("Fadia action clock is not yet bound for this action".into()),
    };
    if lilith {
        let purchased = ctx
            .config
            .talent_levels
            .map(|levels| levels[2])
            .unwrap_or(ctx.config.purchased_skill_level);
        ctx.native_array_index = Some(
            (purchased + u32::from(ctx.config.awakening.has_resonance_3()) - 1).min(12) as usize,
        );
    }
    Ok(Plan {
        hits,
        recovery,
        skill,
        enhanced,
        a5,
        jump,
        skill_gate,
        landing: 0,
    })
}
