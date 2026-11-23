// Native notify progress, nearest microsecond. Effective lost progress is
// consumed by the owner clock and checked independently against the full chain.
pub(super) const HITS: [&[u64]; 5] = [
    &[214_573],
    &[237_320, 394_444],
    &[342_045, 478_629],
    &[194_213, 304_919, 418_481, 618_425],
    &[93_479, 252_913, 729_399],
];
pub(super) const RECOVERY: [u64; 5] = [499_062, 1_153_976, 506_253, 1_968_881, 1_072_842];
pub(super) const SKILL_HITS: &[u64] = &[298_280, 636_395];
pub(super) const SKILL_RECOVERY: u64 = 955_994;
pub(super) const SKILL_CD_MARKER: u64 = 243_640;
pub(super) const BURST_HITS: &[u64] = &[2_807_466, 3_180_295, 3_501_765, 3_947_290, 4_792_259];
pub(super) const BURST_RECOVERY: u64 = 4_915_031;
pub(super) const TRANSFORMED_LAST: &[u64] = &[93_479, 252_913, 752_119];
pub(super) const APPEAR_HITS: &[u64] = &[217_820];
pub(super) const SUPPORT_HITS: &[u64] = &[704_643];

pub(super) fn pause(effect: &str) -> u64 {
    let effect = effect.strip_suffix("_Ultra").unwrap_or(effect);
    match effect {
        "GE_Player_Chaos_PerfectEvadeAttack1_Damage" => 297_000,
        "GE_Player_Chaos_Branch2_Damage" => 185_253,
        "GE_Player_Chaos_Melee1_Damage" | "GE_Player_Chaos_Melee3_Damage" => 19_800,
        "GE_Player_Chaos_Melee2_Damage" => 5_000,
        "GE_Player_Chaos_Melee2_1_Damage" => 16_000,
        "GE_Player_Chaos_Melee4_Damage"
        | "GE_Player_Chaos_Skill1_Damage"
        | "GE_Player_Chaos_Skill2_Damage" => 49_500,
        _ => 0,
    }
}
