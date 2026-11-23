// Consumed source montage coordinates, nearest microsecond. Coin1 shares the
// reached section event, rather than borrowing its ATK coefficient.
pub(super) const HITS: [&[(&str, u64)]; 3] = [
    &[
        ("GE_Player_Chiichan073_Skill1_1Damage", 183_298),
        ("GE_Player_Chiichan073_Skill1_Damage", 599_558),
    ],
    &[
        ("GE_Player_Chiichan073_Skill2_2Damage", 599_893),
        ("GE_Player_Chiichan073_Skill2_2Damage", 966_861),
    ],
    &[
        ("GE_Player_Chiichan073_Skill3_Damage", 252_173),
        ("GE_Player_Chiichan073_Skill4_Damage", 751_256),
        ("GE_Player_Chiichan073_Skill4_Damage", 865_237),
        ("GE_Player_Chiichan073_Skill4_Damage", 949_679),
    ],
];
pub(super) const RECOVERY: [u64; 3] = [919_881, 1_050_484, 1_500_268];
pub(super) const COIN_PROGRESS: [u64; 3] = [599_558, 599_893, 252_173];
pub(super) fn pause(effect: &str) -> Option<(u64, u32)> {
    match effect {
        "GE_Player_Chiichan073_Skill1_1Damage" | "GE_Player_Chiichan073_Skill1_Damage" => {
            Some((200_000, 100_000))
        }
        "GE_Player_Chiichan073_Skill4_Damage" => Some((100_000, 200_000)),
        _ => None,
    }
}
