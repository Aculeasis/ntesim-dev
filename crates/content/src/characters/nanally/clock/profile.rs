//! Saved rate-one asset markers, rounded upward to real microsecond bounds.
pub(super) type Hit = (&'static str, u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Normal(usize),
    Aim,
    Support,
    Riposte,
    Skill,
    Burst,
    Plunge,
    Heavy(usize),
}
pub(super) struct Windows {
    pub recovery: u64,
    pub jump: u64,
    pub movement: u64,
    pub switch: u64,
    pub skill: u64,
}
pub(super) const NORMAL: [&[Hit]; 5] = [
    &[("GE_Player_Nanally_Melee1_Damage", 191_054)],
    &[("GE_Player_Nanally_Melee2_Damage", 209_023)],
    &[
        ("GE_Player_Nanally_Melee3_Damage", 232_926),
        ("GE_Player_Nanally_Melee4_Damage", 316_815),
        ("GE_Player_Nanally_Melee5_Damage", 508_139),
    ],
    &[
        ("GE_Player_Nanally_Melee6_Damage", 349_478),
        ("GE_Player_Nanally_Melee6_Damage", 399_464),
        ("GE_Player_Nanally_Melee6_Damage", 508_732),
        ("GE_Player_Nanally_Melee6_Damage", 630_868),
        ("GE_Player_Nanally_Melee6_Damage", 717_125),
        ("GE_Player_Nanally_Melee6_Damage", 770_502),
        ("GE_Player_Nanally_Melee6_Damage", 833_821),
    ],
    &[
        ("GE_Player_Nanally_Melee9_Damage", 249_362),
        ("GE_Player_Nanally_Melee10_Damage", 801_464),
    ],
];
pub(super) const AIM: &[Hit] = &[("GE_Player_Nanally_BranchPraojectile_Damage", 16_958)];
pub(super) const HEAVY: [&[Hit]; 3] = [
    &[
        ("GE_Player_Nanally_ExtraAtk1_Damage", 460_519),
        ("GE_Player_Nanally_ExtraAtk2_Damage", 514_386),
    ],
    &[
        ("GE_Player_Nanally_ExtraAtkIntro1_Damage", 229_293),
        ("GE_Player_Nanally_ExtraAtkIntro1_Damage", 440_842),
        ("GE_Player_Nanally_ExtraAtkIntro2_Damage", 620_193),
    ],
    &[("GE_Player_Nanally_ExtraAtkOutro1_Damage", 294_516)],
];

pub(super) fn hits(kind: Kind) -> &'static [Hit] {
    match kind {
        Kind::Normal(stage) => NORMAL[stage],
        Kind::Aim => AIM,
        Kind::Plunge => &[("GE_Player_Nanally_AirAttack1_Damage", 50_170)],
        Kind::Heavy(stage) => HEAVY[stage],
        Kind::Support => &[("GE_Player_Nanally_QTE1_Damage", 666_410)],
        Kind::Riposte => &[("GE_Player_Nanally_PerfectEvadeAttack_Damage", 267_562)],
        Kind::Skill => &[
            ("GE_Player_Nanally_Skill1_Damage", 429_637),
            ("GE_Player_Nanally_Skill1_Damage", 497_363),
            ("GE_Player_Nanally_Skill1_Damage", 563_001),
            ("GE_Player_Nanally_Skill1_Damage", 631_341),
            ("GE_Player_Nanally_Skill1_Damage", 696_485),
        ],
        Kind::Burst => &[
            ("GE_Player_Nanally_UltraSkill1_Damage", 2_300_192),
            ("GE_Player_Nanally_UltraSkill2_Damage", 2_382_032),
            ("GE_Player_Nanally_UltraSkill2_Damage", 2_468_449),
            ("GE_Player_Nanally_UltraSkill2_Damage", 2_547_640),
            ("GE_Player_Nanally_UltraSkill2_Damage", 2_634_465),
            ("GE_Player_Nanally_UltraSkill2_Damage", 2_716_783),
            ("GE_Player_Nanally_UltraSkill3_Damage", 3_015_465),
        ],
    }
}

pub(super) fn windows(kind: Kind) -> Windows {
    let (recovery, jump, movement) = match kind {
        Kind::Aim => (133_047, 133_047, 0),
        Kind::Support => (1_108_275, 1_108_275, 1_108_275),
        Kind::Riposte => (841_593, 0, 841_593),
        Kind::Skill => (1_164_219, 1_164_219, 1_512_797),
        Kind::Burst => (3_582_140, 3_582_140, 3_756_871),
        Kind::Plunge => (255_141, 123_558, 749_161),
        Kind::Heavy(stage) => (
            [866_014, 1_009_719, 1_182_105][stage],
            [416_834, 1_052_734, 1_182_105][stage],
            [967_949, 1_009_719, 949_672][stage],
        ),
        Kind::Normal(stage) => (
            [611_861, 870_241, 1_083_048, 1_331_513, 1_134_220][stage],
            [241_794, 259_200, 282_859, 498_069, 332_260][stage],
            [610_988, 1_133_133, 1_383_409, 1_434_812, 1_649_670][stage],
        ),
    };
    Windows {
        recovery,
        jump,
        movement,
        switch: match kind {
            Kind::Support => 1_083_416,
            Kind::Burst => 3_583_025,
            _ => recovery,
        },
        skill: if matches!(kind, Kind::Riposte) {
            0
        } else {
            recovery
        },
    }
}

pub(super) fn pause(effect: &str) -> Option<(u64, u64, u32)> {
    match effect {
        "GE_Player_Nanally_Melee5_Damage" => Some((0, 130_000, 10_000)),
        "GE_Player_Nanally_Melee9_Damage" => Some((0, 60_000, 10_000)),
        "GE_Player_Nanally_Melee10_Damage" => Some((0, 120_000, 10_000)),
        "GE_Player_Nanally_PerfectEvadeAttack_Damage" => Some((0, 300_000, 10_000)),
        "GE_Player_Nanally_QTE1_Damage" => Some((60_000, 250_000, 200_000)),
        "GE_Player_Nanally_AirAttack1_Damage" => Some((0, 60_000, 10_000)),
        "GE_Player_Nanally_ExtraAtk1_Damage" => Some((0, 80_000, 10_000)),
        "GE_Player_Nanally_ExtraAtk2_Damage" => Some((0, 170_000, 10_000)),
        "GE_Player_Nanally_ExtraAtkIntro1_Damage" => Some((0, 100_000, 500_000)),
        "GE_Player_Nanally_ExtraAtkIntro2_Damage" => Some((0, 100_000, 10_000)),
        "GE_Player_Nanally_ExtraAtkOutro1_Damage" => Some((50_000, 200_000, 10_000)),
        _ => None,
    }
}
