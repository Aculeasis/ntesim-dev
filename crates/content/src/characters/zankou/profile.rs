//! Selected current rate-one montage coordinates, in microseconds.
use super::clock::Kind;
pub(crate) type Hit = (&'static str, u64);
pub(crate) fn spread(kind: Kind) -> Option<u64> {
    Some(match kind {
        Kind::Normal(false, stage) => [124_605, 242_848, 196_154, 118_911, 393_359][stage],
        Kind::Normal(true, stage) => [142_873, 55_813, 133_173, 68_755][stage],
        Kind::Held(false) => 51_330,
        Kind::Held(true) => 154_097,
        Kind::Skill(super::forms::Variant::Entry(true)) => 126_483,
        Kind::Skill(super::forms::Variant::Return(true)) => 99_812,
        Kind::Riposte => 166_566,
        _ => return None,
    })
}
const H0: &[Hit] = &[
    ("GE_Player_Zankou_Melee1_Damage", 124456),
    ("GE_Player_Zankou_Melee1_1_Damage", 342267),
];
const H1: &[Hit] = &[("GE_Player_Zankou_Melee2_Damage", 242533)];
const H2: &[Hit] = &[
    ("GE_Player_Zankou_Melee3_Damage", 195147),
    ("GE_Player_Zankou_Melee3_1_Damage", 329921),
    ("GE_Player_Zankou_Melee3_2_Damage", 582381),
];
const H3: &[Hit] = &[
    ("GE_Player_Zankou_Melee4_Damage", 117959),
    ("GE_Player_Zankou_Melee4_1_Damage", 286623),
    ("GE_Player_Zankou_Melee4_2_Damage", 437566),
    ("GE_Player_Zankou_Melee4_3_Damage", 783448),
    ("GE_Player_Zankou_Melee4_4_Damage", 816775),
];
const H4: &[Hit] = &[
    ("GE_Player_Zankou_Melee5_Damage", 383892),
    ("GE_Player_Zankou_Melee5_1_Damage", 548477),
    ("GE_Player_Zankou_Melee5_Damage", 642110),
    ("GE_Player_Zankou_Melee5_1_Damage", 767874),
    ("GE_Player_Zankou_Melee5_2_Damage", 1109198),
    ("GE_Player_Zankou_Melee6_Damage", 1146869),
];
const H5: &[Hit] = &[("GE_Player_Zankou_MagicMelee1_Damage", 142778)];
const H6: &[Hit] = &[
    ("GE_Player_Zankou_MagicMelee2_Damage", 56018),
    ("GE_Player_Zankou_MagicMelee2_1_Damage", 232895),
    ("GE_Player_Zankou_MagicMelee2_2_Damage", 583804),
];
const H7: &[Hit] = &[
    ("GE_Player_Zankou_MagicMelee3_Damage", 132990),
    ("GE_Player_Zankou_MagicMelee3_1_Damage", 300859),
    ("GE_Player_Zankou_MagicMelee3_Damage", 468851),
    ("GE_Player_Zankou_MagicMelee3_1_Damage", 637310),
];
const H8: &[Hit] = &[
    ("GE_Player_Zankou_MagicMelee4_Damage", 48974),
    ("GE_Player_Zankou_MagicMelee4_Damage", 114984),
    ("GE_Player_Zankou_MagicMelee4_1_Damage", 623714),
];
const H9: &[Hit] = &[
    ("GE_Player_Zankou_Branch1_Damage", 50454),
    ("GE_Player_Zankou_Branch2_Damage", 179547),
    ("GE_Player_Zankou_Branch1_Damage", 309436),
    ("GE_Player_Zankou_Branch3_Damage", 850204),
];
const H10: &[Hit] = &[
    ("GE_Player_Zankou_MagicBranch1_Damage", 152536),
    ("GE_Player_Zankou_MagicBranch2_Damage", 359393),
    ("GE_Player_Zankou_MagicBranch3_Damage", 562700),
    ("GE_Player_Zankou_MagicBranch4_Damage", 927319),
    ("GE_Player_Zankou_MagicBranch4_Damage", 1090510),
];
pub(crate) fn hits(kind: Kind) -> &'static [Hit] {
    match kind {
        Kind::Normal(false, 0) => H0,
        Kind::Normal(false, 1) => H1,
        Kind::Normal(false, 2) => H2,
        Kind::Normal(false, 3) => H3,
        Kind::Normal(false, 4) => H4,
        Kind::Normal(true, 0) => H5,
        Kind::Normal(true, 1) => H6,
        Kind::Normal(true, 2) => H7,
        Kind::Normal(true, 3) => H8,
        Kind::Held(false) => H9,
        Kind::Held(true) => H10,
        _ => &[],
    }
}
pub(crate) fn recovery(kind: Kind) -> u64 {
    match kind {
        Kind::Normal(false, 0) => 1084427,
        Kind::Normal(false, 1) => 1127670,
        Kind::Normal(false, 2) => 1117351,
        Kind::Normal(false, 3) => 1958435,
        Kind::Normal(false, 4) => 1474013,
        Kind::Normal(true, 0) => 1084427,
        Kind::Normal(true, 1) => 1279700,
        Kind::Normal(true, 2) => 1691888,
        Kind::Normal(true, 3) => 892160,
        Kind::Held(false) => 1580795,
        Kind::Held(true) => 1620425,
        Kind::Plunge(landing) => landing + 281249,
        Kind::Riposte => 387993,
        Kind::Support => 1034234,
        Kind::Dodge(false) => 1014541,
        Kind::Dodge(true) => 1007068,
        Kind::Burst(variant) => variant.recovery(),
        Kind::Skill(variant) => variant.recovery(),
        Kind::Normal(..) => unreachable!("Zankou stage outside its finite chain"),
    }
}
pub(crate) fn skill(kind: Kind) -> u64 {
    match kind {
        Kind::Normal(false, 0) => 483052,
        Kind::Normal(false, 1) => 519612,
        Kind::Normal(false, 2) => 992815,
        Kind::Normal(false, 3) => 601789,
        Kind::Normal(false, 4) => 1228513,
        Kind::Normal(true, 0) => 337351,
        Kind::Normal(true, 1) => 616365,
        Kind::Normal(true, 2) => 941312,
        Kind::Normal(true, 3) => 892160,
        Kind::Held(false) => 1041360,
        Kind::Held(true) => 1430433,
        Kind::Plunge(landing) => landing + 281249,
        Kind::Riposte => 0,
        Kind::Support => 1034234,
        Kind::Dodge(false) => 333754,
        Kind::Dodge(true) => 494283,
        Kind::Burst(variant) => variant.recovery(),
        Kind::Skill(variant) => variant.recovery(),
        Kind::Normal(..) => unreachable!("Zankou stage outside its finite chain"),
    }
}
pub(crate) fn jump(kind: Kind) -> u64 {
    match kind {
        Kind::Normal(false, 0) => 251033,
        Kind::Normal(false, 1) => 410507,
        Kind::Normal(false, 2) => 601789,
        Kind::Normal(false, 3) => 601789,
        Kind::Normal(false, 4) => 1237350,
        Kind::Normal(true, 0) => 251033,
        Kind::Normal(true, 1) => 592590,
        Kind::Normal(true, 2) => 930315,
        Kind::Normal(true, 3) => 251033,
        Kind::Held(false) => 737644,
        Kind::Held(true) => 1288724,
        Kind::Plunge(landing) => landing + 149206,
        Kind::Riposte => 0,
        Kind::Support => 1034234,
        Kind::Dodge(false) => 82544,
        Kind::Dodge(true) => 0,
        Kind::Burst(variant) => variant.recovery(),
        Kind::Skill(variant) => variant.recovery(),
        Kind::Normal(..) => unreachable!("Zankou stage outside its finite chain"),
    }
}
pub(crate) fn movement(kind: Kind) -> u64 {
    match kind {
        Kind::Normal(false, 0) => 779794,
        Kind::Normal(false, 1) => 896485,
        Kind::Normal(false, 2) => 1731192,
        Kind::Normal(false, 3) => 1315357,
        Kind::Normal(false, 4) => 1599384,
        Kind::Normal(true, 0) => 651010,
        Kind::Normal(true, 1) => 1084427,
        Kind::Normal(true, 2) => 1383541,
        Kind::Normal(true, 3) => 994050,
        Kind::Held(false) => 1580795,
        Kind::Held(true) => 1620425,
        Kind::Plunge(landing) => landing + 750726,
        Kind::Riposte => 652065,
        Kind::Support => 1034234,
        Kind::Dodge(false) => 1014541,
        Kind::Dodge(true) => 836966,
        Kind::Burst(variant) => variant.recovery(),
        Kind::Skill(variant) => variant.recovery(),
        Kind::Normal(..) => unreachable!("Zankou stage outside its finite chain"),
    }
}
pub(crate) fn pause(effect: &str) -> u64 {
    match effect {
        "GE_Player_Zankou_Branch3_Damage" => 19800,
        "GE_Player_Zankou_MagicMelee2_2_Damage" => 58800,
        "GE_Player_Zankou_Melee2_Damage" => 29700,
        "GE_Player_Zankou_Melee4_3_Damage" => 19800,
        "GE_Player_Zankou_PerfectEvadeAttack_Damage" => 297000,
        "GE_Player_Zankou_QTE_Damage" => 320000,
        "GE_Player_Zankou_Skill1_1_Damage" => 27000,
        "GE_Player_Zankou_Skill2_1_Damage" => 27000,
        _ => 0,
    }
}
