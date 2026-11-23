//! Consumed contact/recovery coordinates and reached portions of owner pause curves.
use super::{Boundary, Kind, Plan};
use crate::runtime_api::CastContext;
use ntesim_core::kernel::timing::HitlagRatePoint;
pub(super) fn select(
    ctx: &mut CastContext<'_>,
    stage: usize,
    cyclone: Option<u8>,
) -> Result<Plan, String> {
    if super::super::air::accepted(ctx.request)
        && let Some(plan) = super::super::air::select(ctx)?
    {
        return Ok(plan);
    }
    if super::super::secondary::accepted(ctx.request)
        && let Some(plan) = super::super::secondary::select(ctx)?
    {
        return Ok(plan);
    }
    if let Some(n) = cyclone {
        let p = super::super::cyclone::plan(n);
        return Ok(Plan {
            kind: Kind::Cyclone(n),
            hits: p.hits,
            recovery: p.recovery,
            jump: [1_582_164, 1_833_353, 1_248_400][usize::from(n - 1)],
            switch: p.recovery,
            notifies: p.notifies,
        });
    }
    if ctx.ability_id == "GA_Hathor_Skill" {
        if let Some(p) = super::super::held::plan(ctx)? {
            let release = ctx.request.hold_inputs[&ctx.command_index]
                .duration_us
                .min(5_000_000);
            return Ok(Plan {
                kind: Kind::Skill,
                hits: p.hits,
                recovery: p.recovery,
                jump: release
                    + if release <= 1_000_000 {
                        766_680
                    } else {
                        1_833_415
                    },
                switch: p.recovery,
                notifies: p.notifies,
            });
        }
        return Ok(Plan {
            kind: Kind::Skill,
            hits: SKILL.to_vec(),
            recovery: 1_433_378,
            jump: 1_057_816,
            switch: 1_433_378,
            notifies: vec![
                (283_333, Boundary::TapPower),
                (300_073, Boundary::SkillCooldown),
            ],
        });
    }
    if ctx.ability_id == "GA_Hathor_UltraSkill" {
        return Ok(Plan {
            kind: Kind::Burst,
            hits: BURST.to_vec(),
            recovery: super::super::burst::RECOVERY,
            jump: super::super::burst::RECOVERY,
            switch: 4_522_841,
            notifies: vec![(1_499_991, Boundary::Emergency)],
        });
    }
    if ctx.ability_id != "GA_Hathor_Melee" {
        return Err("Hathor clock has no action binding".into());
    }
    Ok(Plan {
        kind: Kind::Normal(stage),
        hits: NORMAL[stage].to_vec(),
        recovery: RECOVERY[stage],
        jump: [316_916, 869_900, 716_561, 949_947, 1_500_294][stage],
        switch: RECOVERY[stage],
        notifies: vec![],
    })
}
pub(super) type Hit = (&'static str, u64);
pub(super) const NORMAL: [&[Hit]; 5] = [
    &[("GE_Player_Hathor_Melee1_Damage", 216746)],
    &[
        ("GE_Player_Hathor_Melee2_Damage", 199964),
        ("GE_Player_Hathor_Melee2_1_Damage", 383195),
        ("GE_Player_Hathor_Melee2_1_Damage", 483464),
        ("GE_Player_Hathor_Melee2_1_Damage", 633209),
        ("GE_Player_Hathor_Melee2_1_Damage", 750064),
    ],
    &[
        ("GE_Player_Hathor_Melee3_Damage", 299698),
        ("GE_Player_Hathor_Melee3_Damage", 383747),
    ],
    &[
        ("GE_Player_Hathor_Melee4_Damage", 199797),
        ("GE_Player_Hathor_Melee4_1_Damage", 416574),
        ("GE_Player_Hathor_Melee4_2_Damage", 799878),
    ],
    &[
        ("GE_Player_Hathor_Melee5_Damage", 399196),
        ("GE_Player_Hathor_Melee5_1_Damage", 940983),
    ],
];
pub(super) const RECOVERY: [u64; 5] = [1100977, 1417339, 1250544, 1750748, 2250971];
pub(super) const SKILL: &[Hit] = &[
    ("GE_Player_Hathor_Skill1_Damage", 516245),
    ("GE_Player_Hathor_Skill1_Damage", 699503),
    ("GE_Player_Hathor_Skill11_Damage", 916369),
];
pub(super) const BURST: &[Hit] = &[
    ("GE_Player_Hathor_UltraSkill1_Damage", 2_483_287),
    ("GE_Player_Hathor_UltraSkill1_Damage", 2_883_287),
    ("GE_Player_Hathor_UltraSkill2_Damage", 3_816_348),
];
const SKILL_CURVE: &[HitlagRatePoint] = &[
    HitlagRatePoint {
        offset_us: 0,
        rate_ppm: 50000,
    },
    HitlagRatePoint {
        offset_us: 16666,
        rate_ppm: 0,
    },
    HitlagRatePoint {
        offset_us: 66666,
        rate_ppm: 0,
    },
    HitlagRatePoint {
        offset_us: 83333,
        rate_ppm: 500000,
    },
];
const BRANCH_CURVE: &[HitlagRatePoint] = &[
    HitlagRatePoint {
        offset_us: 0,
        rate_ppm: 300_000,
    },
    HitlagRatePoint {
        offset_us: 16666,
        rate_ppm: 30_000,
    },
    HitlagRatePoint {
        offset_us: 100000,
        rate_ppm: 30_000,
    },
    HitlagRatePoint {
        offset_us: 216666,
        rate_ppm: 1000000,
    },
];
const LAST: &[HitlagRatePoint] = &[
    HitlagRatePoint {
        offset_us: 0,
        rate_ppm: 75000,
    },
    HitlagRatePoint {
        offset_us: 16666,
        rate_ppm: 0,
    },
    HitlagRatePoint {
        offset_us: 66666,
        rate_ppm: 0,
    },
    HitlagRatePoint {
        offset_us: 83333,
        rate_ppm: 10000,
    },
    HitlagRatePoint {
        offset_us: 100000,
        rate_ppm: 100000,
    },
    HitlagRatePoint {
        offset_us: 116666,
        rate_ppm: 300000,
    },
    HitlagRatePoint {
        offset_us: 150000,
        rate_ppm: 900000,
    },
];
pub(super) enum Pause {
    Flat {
        duration: u64,
        delay: u64,
        rate: u32,
    },
    Curve(&'static [HitlagRatePoint], u64),
}
pub(super) fn pause(effect: &str) -> Option<Pause> {
    Some(match effect {
        // Native constant-rate windows; preserve the real interval for explicit
        // interruption and later actions admitted before that interval ends.
        "GE_Player_Hathor_QTE1_Damage" => Pause::Flat {
            duration: 400_000,
            delay: 0,
            rate: 200_000,
        },
        "GE_Player_Hathor_QTE2_Damage" => Pause::Flat {
            duration: 33_333,
            delay: 0,
            rate: 10_000,
        },
        // Their configured duration reaches only the constant-zero curve portion.
        "GE_Player_Hathor_Melee1_Damage"
        | "GE_Player_Hathor_Melee2_Damage"
        | "GE_Player_Hathor_Melee3_Damage" => Pause::Flat {
            duration: 33333,
            delay: 16666,
            rate: 0,
        },
        "GE_Player_Hathor_Melee2_1_Damage" => Pause::Flat {
            duration: 16666,
            delay: 0,
            rate: 0,
        },
        "GE_Player_Hathor_Melee4_Damage" | "GE_Player_Hathor_Melee4_1_Damage" => Pause::Flat {
            duration: 33333,
            delay: 0,
            rate: 0,
        },
        // The maximum reached native rate1.389485e-8 rounds to0 integer PPM.
        "GE_Player_Hathor_Melee4_2_Damage" => Pause::Flat {
            duration: 50000,
            delay: 0,
            rate: 0,
        },
        "GE_Player_Hathor_Melee5_Damage" => Pause::Flat {
            duration: 16666,
            delay: 0,
            rate: 10000,
        },
        "GE_Player_Hathor_Melee5_1_Damage" => Pause::Curve(LAST, 133333),
        "GE_Player_Hathor_Branch_Damage" => Pause::Curve(BRANCH_CURVE, 83_333),
        "GE_Player_Hathor_Skill1_Damage" => Pause::Curve(SKILL_CURVE, 33333),
        "GE_Player_Hathor_Skill11_Damage" => Pause::Curve(SKILL_CURVE, 83333),
        "GE_Player_Hathor_Skill5_Damage" => Pause::Flat {
            duration: 33333,
            delay: 0,
            rate: 30000,
        },
        "GE_Player_Hathor_Skill6_Damage" => Pause::Flat {
            duration: 16666,
            delay: 0,
            rate: 0,
        },
        "GE_Player_Hathor_Skill9_Damage" => Pause::Flat {
            duration: 50000,
            delay: 0,
            rate: 10000,
        },
        "GE_Player_Hathor_Skill10_Damage" => Pause::Flat {
            duration: 83333,
            delay: 0,
            rate: 60000,
        },
        "GE_Player_Hathor_Skill2_Damage"
        | "GE_Player_Hathor_Skill3_Damage"
        | "GE_Player_Hathor_Skill4_Damage" => Pause::Flat {
            duration: 16666,
            delay: 0,
            rate: 10000,
        },
        _ => return None,
    })
}
