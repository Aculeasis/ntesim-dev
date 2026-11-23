// Frozen montage coordinates, nearest microsecond. Normal4's released actor
// pulses keep real .15s intervals from its reached spawn, independent of montage.
pub(super) const ROWS: [&str; 5] = [
    "GE_Player_Chiichan073_Melee1_Damage",
    "GE_Player_Chiichan073_Melee2_Damage",
    "GE_Player_Chiichan073_Melee3_Damage",
    "GE_Player_Chiichan073_Melee4_Damage",
    "GE_Player_Chiichan073_Melee5_Damage",
];
pub(super) const HITS: [&[u64]; 5] = [
    &[200_711],
    &[198_745, 283_704, 398_206],
    &[517_553],
    &[236_106, 386_106, 536_106, 686_106, 836_106],
    &[333_086, 666_537],
];
pub(super) const RECOVERY: [u64; 5] = [531_209, 783_916, 982_782, 1_099_458, 1_197_803];
pub(super) const PAUSE_DELAY: u64 = 16_700;
pub(super) const PAUSE_RATE_PPM: u32 = 10_000;

pub(super) fn recovery(kind: super::Kind) -> u64 {
    match kind {
        super::Kind::Burst => 3_754_213,
        super::Kind::BarrenCue => 500_015,
        super::Kind::Barren => 1_912_478,
        super::Kind::Normal(stage) => RECOVERY[stage],
        super::Kind::Plunge => 500_015,
        super::Kind::Riposte => 1_026_147,
        super::Kind::Support => 1_736_720,
        super::Kind::Dodge(back) => {
            if back {
                1_012_579
            } else {
                998_284
            }
        }
    }
}
pub(super) fn gate(kind: super::Kind, input: &str) -> Option<u64> {
    match (kind, input) {
        (super::Kind::Barren, "jump") => Some(868_260),
        (super::Kind::Normal(stage), "jump") => {
            Some([333_817, 248_872, 669_966, 847_815, 842_563][stage])
        }
        (super::Kind::Riposte, "jump" | "GA_Chiichan073_Skill") => Some(0),
        (super::Kind::Dodge(back), "jump") => Some(if back { 0 } else { 82_544 }),
        (super::Kind::Dodge(back), "GA_Chiichan073_Skill") => {
            Some(if back { 350_074 } else { 333_716 })
        }
        (_, "jump" | "GA_Chiichan073_Skill") => Some(recovery(kind)),
        _ => None,
    }
}

pub(super) fn pause(effect: &str) -> (u64, u32) {
    match effect {
        "GE_Player_Chiichan073_AirAttack1_Damage" => (150_000, 10_000),
        "GE_Player_Chiichan073_PerfectEvadeAttack2_Damage" => (200_000, 80_000),
        // All admitted Support gates are after the end of its Hermite curve:
        // its independently integrated lost progress is exactly160558us at
        // nearest microsecond. No gate inside that curve is exposed.
        "GE_Player_Chiichan073_QTE_Damage" => (160_558, 0),
        _ => (pause_duration(effect), PAUSE_RATE_PPM),
    }
}

pub(super) fn pause_duration(effect: &str) -> u64 {
    match effect {
        "GE_Player_Chiichan073_Melee1_Damage" | "GE_Player_Chiichan073_Melee3_Damage" => 60_000,
        "GE_Player_Chiichan073_Melee5_Damage" => 100_000,
        _ => 0,
    }
}
