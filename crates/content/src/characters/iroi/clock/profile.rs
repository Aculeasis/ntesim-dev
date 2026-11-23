//! Selected rate-one montage coordinates, rounded once to microseconds.
pub(super) const EFFECTS: [&str; 5] = [
    "GE_Player_Oneiroi_Melee1_Damage",
    "GE_Player_Oneiroi_Melee2_Damage",
    "GE_Player_Oneiroi_Melee3_Damage",
    "GE_Player_Oneiroi_Melee4_Damage",
    "GE_Player_Oneiroi_Melee5_Damage",
];
pub(super) const HITS: [&[u64]; 5] = [
    &[316667],
    &[433333, 616667, 800000],
    &[], // Its finite world actor is born at268779, first contact200000 later.
    &[416667],
    &[583333, 633532, 683518],
];
pub(super) const NEXT: [u64; 5] = [476652, 994707, 659691, 960171, 1051076];
pub(super) fn gate(kind: super::Kind, input: &str) -> u64 {
    match kind {
        super::Kind::Burst => {
            if input == "switch_actor" {
                3764482
            } else {
                4015812
            }
        }
        super::Kind::Transform => match input {
            "switch_actor" => 600000,
            "jump" => 0,
            "dash" => 250875,
            _ => 291120,
        },
        super::Kind::Support => match input {
            "switch_actor" => 1_344_117,
            _ => super::SUPPORT.recovery_us,
        },
        super::Kind::Branch { stage, hold_us } => match input {
            "jump" => stage.jump(hold_us),
            _ => stage.any_move(hold_us),
        },
        super::Kind::LucidDream { .. } => match input {
            "jump" => 0,
            _ => 7104511,
        },
        super::Kind::Normal(stage) => match input {
            "GA_Oneiroi_Melee" => NEXT[stage],
            "GA_Oneiroi_Skill" => [507329, 1039969, 740427, 903427, 876007][stage],
            "dash" => [836111, 1613270, 1336110, 1125417, 1234040][stage],
            _ => [836111, 1613270, 1166324, 1276495, 1051076][stage],
        },
        super::Kind::Skill => {
            if input == "dash" {
                1000372
            } else {
                819905
            }
        }
    }
}
pub(super) fn recovery(kind: super::Kind) -> u64 {
    match kind {
        super::Kind::Normal(stage) => NEXT[stage],
        super::Kind::Skill => 819905,
        super::Kind::Transform => 250875,
        super::Kind::Branch { stage, hold_us } => stage.any_move(hold_us),
        super::Kind::LucidDream { .. } => 7104511,
        super::Kind::Burst => 4015812,
        super::Kind::Support => super::SUPPORT.recovery_us,
    }
}
pub(super) fn pause(effect: &str) -> Option<(u64, u64)> {
    match effect {
        super::QTE_PACKET => Some((300_000, 0)),
        "GE_Player_Oneiroi_Melee1_Damage" | "GE_Player_Oneiroi_Melee2_Damage" => Some((33000, 0)),
        "GE_Player_Oneiroi_Melee4_Damage" | "GE_Player_Oneiroi_Melee5_Damage" => {
            Some((33000, 33000))
        }
        "GE_Player_Oneiroi_Skill_Gun_Damage"
        | "GE_Player_Oneiroi_Skill_Syringe_Damage"
        | "GE_Player_Oneiroi_Skill_Trumpet_Damage" => Some((100000, 0)),
        "GE_Player_Oneiroi_Branch1_Damage"
        | "GE_Player_Oneiroi_Branch2_Damage"
        | "GE_Player_Oneiroi_Branch3_Damage"
        | "GE_Player_Oneiroi_Branch5_Damage" => Some((33000, 0)),
        _ => None,
    }
}
