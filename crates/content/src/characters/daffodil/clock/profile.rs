//! Reached source coordinates, rounded once to nearest microsecond.
use super::Kind;
pub(crate) type Hit = (&'static str, u64);
pub(super) const NORMAL: [&[Hit]; 5] = [
    &[("GE_Player_Daffodill_Melee1_Damage", 185_308)],
    &[
        ("GE_Player_Daffodill_Melee2_Damage", 124_620),
        ("GE_Player_Daffodill_Melee2_1_Damage", 345_453),
    ],
    &[
        ("GE_Player_Daffodill_Melee3_Damage", 186_126),
        ("GE_Player_Daffodill_Melee3_Damage", 228_422),
        ("GE_Player_Daffodill_Melee3_1_Damage", 355_357),
        ("GE_Player_Daffodill_Melee3_1_Damage", 397_732),
        ("GE_Player_Daffodill_Melee3_2_Damage", 569_242),
        ("GE_Player_Daffodill_Melee3_3_Damage", 907_714),
    ],
    &[
        ("GE_Player_Daffodill_Melee4_Damage", 252_337),
        ("GE_Player_Daffodill_Melee4_1_Damage", 490_139),
        ("GE_Player_Daffodill_Melee4_2_Damage", 638_915),
    ],
    &[
        ("GE_Player_Daffodill_Melee5_Damage", 250_153),
        ("GE_Player_Daffodill_Melee6_Damage", 300_007),
        ("GE_Player_Daffodill_Melee6_Damage", 424_944),
        ("GE_Player_Daffodill_Melee5_Damage", 483_563),
        ("GE_Player_Daffodill_Melee6_Damage", 534_406),
        ("GE_Player_Daffodill_Melee6_Damage", 597_252),
        ("GE_Player_Daffodill_Melee5_1_Damage", 800_387),
    ],
];
pub(super) const SKILL: &[Hit] = &[
    ("GE_Player_Daffodill_Skill1_Damage", 196_675),
    ("GE_Player_Daffodill_Skill1_Damage", 331_903),
    ("GE_Player_Daffodill_Skill2_Damage", 670_029),
    ("GE_Player_Daffodill_Skill6_Damage", 749_890),
    ("GE_Player_Daffodill_Skill7_Damage", 832_625),
    ("GE_Player_Daffodill_Skill8_Damage", 916_519),
    ("GE_Player_Daffodill_Skill9_Damage", 1_000_193),
    ("GE_Player_Daffodill_Skill5_Damage", 1_105_589),
];
pub(super) const ECHOES: &[Hit] = &[
    ("GE_Player_Daffodill_Skill3_Damage", 118_959),
    ("GE_Player_Daffodill_Skill3_Damage", 195_011),
    ("GE_Player_Daffodill_Skill3_Damage", 283_757),
    ("GE_Player_Daffodill_Skill3_Damage", 393_685),
    ("GE_Player_Daffodill_Skill3_Damage", 471_799),
    ("GE_Player_Daffodill_Skill3_Damage", 545_992),
    ("GE_Player_Daffodill_Skill4_Damage", 784_395),
];
pub(super) const BURST: &[Hit] = &[
    ("GE_Player_Daffodill_UltraSkill1_Damage", 2_849_375),
    ("GE_Player_Daffodill_UltraSkill3_Damage", 2_906_067),
    ("GE_Player_Daffodill_UltraSkill3_Damage", 2_960_202),
    ("GE_Player_Daffodill_UltraSkill2_Damage", 3_049_960),
    ("GE_Player_Daffodill_UltraSkill6_Damage", 3_100_386),
    ("GE_Player_Daffodill_UltraSkill6_Damage", 3_149_607),
    ("GE_Player_Daffodill_UltraSkill4_Damage", 3_775_980),
    ("GE_Player_Daffodill_UltraSkill4_Damage", 3_836_060),
    ("GE_Player_Daffodill_UltraSkill4_Damage", 3_891_205),
    ("GE_Player_Daffodill_UltraSkill4_Damage", 3_952_318),
    ("GE_Player_Daffodill_UltraSkill4_Damage", 4_019_527),
    ("GE_Player_Daffodill_UltraSkill4_Damage", 4_097_983),
    ("GE_Player_Daffodill_UltraSkill4_Damage", 4_154_141),
    ("GE_Player_Daffodill_UltraSkill5_Damage", 4_657_557),
];
pub(super) fn recovery(kind: Kind) -> u64 {
    match kind {
        Kind::Normal(stage) => [1_003_012, 705_447, 1_607_759, 1_379_811, 1_161_335][stage],
        Kind::Skill(false) => 1_210_492,
        Kind::Skill(true) => 988_628,
        Kind::Burst => 5_088_978,
        Kind::Branch => 1_433_323,
        Kind::Plunge => 281_249,
        Kind::Riposte => 874_201,
        Kind::Support => 900_691,
        Kind::Step => super::super::phantom::STEP_RECOVERY_US,
        Kind::SupportParry => 900_691,
        Kind::Dodge(back) => {
            if back {
                1_007_068
            } else {
                1_014_541
            }
        }
    }
}
// The selected owner clock adds duration*(1-rate) at each connected request.
// These integrated lost-progress intervals pause only this finite montage.
pub(super) fn pause(effect: &str) -> u64 {
    match effect {
        "GE_Player_Daffodill_Melee2_1_Damage" => 9_900,
        "GE_Player_Daffodill_Melee3_3_Damage" => 57_600,
        "GE_Player_Daffodill_Melee5_1_Damage" => 76_800,
        "GE_Player_Daffodill_Branch3_Damage" => 118_800,
        "GE_Player_Daffodill_Skill5_Damage" => 47_500,
        "GE_Player_Daffodill_Skill4_Damage" => 118_800,
        "GE_Player_Daffodill_AirAttack_Damage" => 39_600,
        "GE_Player_Daffodill_PerfectEvadeAttack_Damage" => 297_000,
        "GE_Player_Daffodill_QTE_Damage" => 349_650,
        "GE_Player_Daffodill_EntryAttack1_Damage" => 59_400,
        "GE_Player_Daffodill_EntryAttack4_Damage" => 80_000,
        _ => 0,
    }
}
