//! Native notify progress in nearest microseconds (`Animation/Skill/*.json` via `action-damage.md`).

pub(super) type Hit = (&'static str, u64);

pub(crate) const MELEE1_HITS: &[Hit] = &[
    ("GE_Player_Elyms_Melee1_Damage", 131_303),
    ("GE_Player_Elyms_Melee1_Damage", 307_749),
];
pub(crate) const MELEE2_HITS: &[Hit] = &[
    ("GE_Player_Elyms_Melee2_Damage", 163_746),
    ("GE_Player_Elyms_Melee2_Damage", 261_461),
];
pub(crate) const MELEE3_HITS: &[Hit] = &[
    ("GE_Player_Elyms_Melee3_Damage", 337_874),
    ("GE_Player_Elyms_Melee3_Damage", 624_525),
    ("GE_Player_Elyms_Melee3_1_Damage", 835_709),
];
pub(crate) const MELEE4_HITS: &[Hit] = &[
    ("GE_Player_Elyms_Melee4_Damage", 86_976),
    ("GE_Player_Elyms_Melee4_1_Damage", 194_800),
    ("GE_Player_Elyms_Melee4_Damage", 307_586),
    ("GE_Player_Elyms_Melee4_1_Damage", 411_490),
    ("GE_Player_Elyms_Melee4_Damage", 516_182),
];
pub(crate) const MELEE5_HITS: &[Hit] = &[("GE_Player_Elyms_Melee5_Damage", 605_117)];
pub(crate) const MELEE5_ADDE_PROGRESS: u64 = 632_180;
pub(crate) const NORMAL_RECOVERY: [u64; 5] = [657_555, 788_579, 2_018_442, 2_104_604, 1_305_152];

pub(crate) const MELEEB1_HITS: &[Hit] = &[
    ("GE_Player_Elyms_MeleeB1_Damage", 164_788),
    ("GE_Player_Elyms_MeleeB1_1_Damage", 355_115),
];
pub(crate) const MELEEB2_HITS: &[Hit] = &[
    ("GE_Player_Elyms_MeleeB2_Damage", 165_958),
    ("GE_Player_Elyms_MeleeB2_1_Damage", 232_924),
    ("GE_Player_Elyms_MeleeB2_Damage", 301_134),
    ("GE_Player_Elyms_MeleeB2_1_Damage", 365_986),
];
pub(crate) const MELEEB3_HITS: &[Hit] = &[
    ("GE_Player_Elyms_MeleeB3_Damage", 148_066),
    ("GE_Player_Elyms_MeleeB3_Damage", 247_854),
    ("GE_Player_Elyms_MeleeB3_Damage", 380_905),
    ("GE_Player_Elyms_MeleeB3_1_Damage", 665_314),
];
pub(crate) const MELEEB3_ADDE_PROGRESS: u64 = 617_396;
pub(crate) const CROSSTALK_RECOVERY: [u64; 3] = [1_001_050, 1_060_205, 1_508_954];

pub(crate) const BRANCH_HITS: &[Hit] = &[
    ("GE_Player_Elyms_Branch1_Damage", 70_101),
    ("GE_Player_Elyms_Branch2_Damage", 180_677),
    ("GE_Player_Elyms_Branch1_Damage", 314_019),
    ("GE_Player_Elyms_Branch3_Damage", 533_002),
];
pub(crate) const BRANCH_RECOVERY: u64 = 649_976;

pub(crate) const AIM_HITS: &[Hit] = &[("GE_Player_Elyms_Aim_Damage", 48_313)];
pub(crate) const AIM_RECOVERY: u64 = 146_353;

pub(crate) const PLUNGE_HIT: u64 = 50_169;
pub(crate) const PLUNGE_RECOVERY: u64 = 766_767;

pub(crate) const RIPOSTE_HIT: u64 = 182_751;
pub(crate) const RIPOSTE_RECOVERY: u64 = 651_818;

pub(crate) const QTE_HIT: u64 = 747_546;
pub(crate) const QTE_SWITCH_LOCK: u64 = 1_240_934;
pub(crate) const QTE_RECOVERY: u64 = 1_246_321;

pub(crate) const SKILL1_HITS: &[Hit] = &[
    ("GE_Player_Elyms_Skill1_Damage", 163_746),
    ("GE_Player_Elyms_Skill1_1_Damage", 461_007),
    ("GE_Player_Elyms_Skill1_1_Damage", 523_651),
    ("GE_Player_Elyms_Skill1_1_Damage", 625_732),
    ("GE_Player_Elyms_Skill1_1_Damage", 711_157),
    ("GE_Player_Elyms_Skill1_2_Damage", 821_274),
];
pub(crate) const SKILL1_ADDE_PROGRESS: u64 = 818_769;
pub(crate) const SKILL1_RECOVERY: u64 = 1_309_221;

pub(crate) const SKILL2_HITS: &[Hit] = &[
    ("GE_Player_Elyms_Skill2_Damage", 97_422),
    ("GE_Player_Elyms_Skill2_Damage", 288_095),
    ("GE_Player_Elyms_Skill2_Damage", 469_234),
    ("GE_Player_Elyms_Skill2_Damage", 664_674),
    ("GE_Player_Elyms_Skill2_Damage", 844_009),
    ("GE_Player_Elyms_Skill2_1_Damage", 1_186_104),
];
pub(crate) const SKILL2_MANAGE_PROGRESS: u64 = 605_117;
pub(crate) const SKILL2_SWITCH_LOCK: u64 = 1_669_059;
pub(crate) const SKILL2_RECOVERY: u64 = 1_669_059;

pub(crate) const SKILL3_HITS: &[Hit] = &[
    ("GE_Player_Elyms_Skill3_Damage", 97_422),
    ("GE_Player_Elyms_Skill3_Damage", 288_095),
    ("GE_Player_Elyms_Skill3_Damage", 469_234),
    ("GE_Player_Elyms_Skill3_Damage", 664_674),
    ("GE_Player_Elyms_Skill3_Damage", 844_009),
    ("GE_Player_Elyms_Skill3_1_Damage", 1_186_104),
];
pub(crate) const SKILL3_EXTRA_HIT: u64 = 1_186_104;
pub(crate) const SKILL3_SWITCH_LOCK: u64 = 1_666_238;
pub(crate) const SKILL3_RECOVERY: u64 = 1_666_238;

pub(crate) const BURSTA_HITS: &[Hit] = &[
    ("GE_Player_Elyms_UltraSkillA1_Damage", 3_124_098),
    ("GE_Player_Elyms_UltraSkillA1_Damage", 3_328_750),
    ("GE_Player_Elyms_UltraSkillA1_Damage", 3_544_312),
    ("GE_Player_Elyms_UltraSkillA2_Damage", 4_616_667),
];
pub(crate) const BURSTA_SWITCH_LOCK: u64 = 5_499_266;
pub(crate) const BURSTA_RECOVERY: u64 = 5_569_568;

pub(crate) const BURSTB_HITS: &[Hit] = &[
    ("GE_Player_Elyms_UltraSkillB1_Damage", 3_126_049),
    ("GE_Player_Elyms_UltraSkillB1_Damage", 3_328_750),
    ("GE_Player_Elyms_UltraSkillB1_Damage", 3_544_312),
    ("GE_Player_Elyms_UltraSkillB2_Damage", 4_628_571),
    ("GE_Player_Elyms_UltraSkillB2_Damage", 4_741_408),
    ("GE_Player_Elyms_UltraSkillB2_Damage", 4_845_375),
    ("GE_Player_Elyms_UltraSkillB2_Damage", 4_987_917),
    ("GE_Player_Elyms_UltraSkillB3_Damage", 5_166_667),
];
pub(crate) const BURSTB_SWITCH_LOCK: u64 = 5_499_266;
pub(crate) const BURSTB_RECOVERY: u64 = 5_566_667;

pub(crate) const XIETONGA_FIRST_HIT: u64 = 254_680;
pub(crate) const XIETONGB_FIRST_HIT: u64 = 326_896;

pub(crate) const EVADE_FORWARD_RECOVERY: u64 = 431_326;
pub(crate) const EVADE_BACK_RECOVERY: u64 = 499_987;

pub(super) fn pause(effect: &str) -> Option<(u64, u64, u32)> {
    // (delay_us, duration_real_us, dilated_rate_ppm) -> lost progress matches action-damage.md §4.2
    match effect {
        "GE_Player_Elyms_Branch3_Damage" => Some((0, 20_000, 10_000)),
        "GE_Player_Elyms_Melee5_Damage" => Some((0, 30_000, 10_000)),
        "GE_Player_Elyms_QTE_Damage" => Some((16_700, 20_000, 100_000)),
        "GE_Player_Elyms_UltraSkillA2_Damage" => Some((0, 50_000, 30_000)),
        _ => None,
    }
}
