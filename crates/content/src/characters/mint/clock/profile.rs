//! Consumed selected source coordinates, rounded once to nearest microsecond.
pub(super) type Hit = (&'static str, u64);
pub(super) const NORMAL: [&[Hit]; 5] = [
    &[
        ("GE_Player_Mint_Melee1_Damage_New", 154636),
        ("GE_Player_Mint_Melee2_Damage_New", 309723),
    ],
    &[("GE_Player_Mint_Melee3_Damage_New", 89514)],
    &[
        ("GE_Player_Mint_Melee4_Damage_New", 241903),
        ("GE_Player_Mint_Melee5_Damage_New", 378755),
    ],
    &[("GE_Player_Mint_Melee6_Damage_New", 149935)],
    &[
        ("GE_Player_Mint_Melee7_Damage_New", 259318),
        ("GE_Player_Mint_Melee8_Damage_New", 600077),
    ],
];
pub(super) const SKILL: &[Hit] = &[
    ("GE_Player_Mint_Skill1_Damage_New", 200178),
    ("GE_Player_Mint_Skill1_Damage_New", 515924),
];
pub(super) const BURST: &[Hit] = &[
    ("GE_Player_Mint_UtraSkill1_Damage_New", 1615911),
    ("GE_Player_Mint_UtraSkill2_Damage_New", 1754891),
    ("GE_Player_Mint_UtraSkill3_Damage_New", 1937954),
    ("GE_Player_Mint_UtraSkill4_Damage_New", 2143918),
    ("GE_Player_Mint_UtraSkill5_Damage_New", 2260660),
    ("GE_Player_Mint_UtraSkill6_Damage_New", 2427269),
    ("GE_Player_Mint_UtraSkill7_Damage_New", 3499036),
];
pub(super) fn recovery(kind: super::Kind) -> u64 {
    match kind {
        super::Kind::Normal(stage) => [832327, 665744, 1183458, 716001, 986150][stage],
        super::Kind::Skill => 868323,
        super::Kind::Burst => 3907992,
        super::Kind::Hold => 5_410_000 + 616_639,
        super::Kind::End => 616_639,
        super::Kind::Entry => 776779,
        super::Kind::Approach => 831319,
        super::Kind::Plunge => 333451,
        super::Kind::Riposte => 443505,
        super::Kind::Support => 1366703,
        super::Kind::Dodge(back) => {
            if back {
                998398
            } else {
                1004085
            }
        }
    }
}
pub(super) fn pause(effect: &str) -> (u64, u32, u64) {
    let duration = match effect {
        "GE_Player_Mint_Melee1_Damage_New" => 16700,
        "GE_Player_Mint_Melee3_Damage_New" => 30000,
        "GE_Player_Mint_Melee5_Damage_New" => 50000,
        "GE_Player_Mint_Melee6_Damage_New" | "GE_Player_Mint_Melee8_Damage_New" => 60000,
        "GE_Player_Mint_Skill1_Damage_New" | super::super::hold::EVADE1 => 50000,
        super::super::riposte::PACKET => 300000,
        super::secondary::SUPPORT_PACKET => return (400000, 200000, 100000),
        _ => 0,
    };
    (duration, 10000, 0)
}
