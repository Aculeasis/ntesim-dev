// Native notify progress in nearest microseconds (`HT/Content/Characters/Player/057_Akane_fashion2/Animation/Skill/*.uasset`).
pub(crate) const MELEE1_HITS: &[u64] = &[133_639, 465_515];
pub(crate) const MELEE_PRE_HITS: &[u64] = &[117_649, 449_141];
pub(crate) const MELEE2_HITS: &[u64] = &[183_217, 353_334];
pub(crate) const MELEE3_HITS: &[u64] = &[
    133_022, 393_471, 538_558, 812_638, 835_430, 849_551, 867_402,
];
// DamageActorAkane_Melee4 configures DelayTriggerTime = 0.03s (30_000 us) on top of notify 589_367 us.
pub(crate) const MELEE4_HITS: &[u64] = &[619_367];

pub(crate) const NORMAL_RECOVERY: [u64; 4] = [781_518, 491_588, 896_640, 747_497];

pub(crate) const SKILL_PRESS_HITS: &[u64] = &[345_126];
pub(crate) const SKILL_PRESS_MODE_PROGRESS: u64 = 342_294;
pub(crate) const SKILL_PRESS_CD_MARKER: u64 = 276_833;
pub(crate) const SKILL_PRESS_RECOVERY: u64 = 1_498_994;

pub(crate) const SKILL_HOLD_HITS: &[u64] = &[70_836, 641_018];
pub(crate) const SKILL_HOLD_MODE_PROGRESS: u64 = 612_793;
pub(crate) const SKILL_HOLD_CD_MARKER: u64 = 643_221;
pub(crate) const SKILL_HOLD_RECOVERY: u64 = 1_417_949;

pub(crate) const PROSKILL_1_HITS: &[u64] = &[429_237];
pub(crate) const PROSKILL_1_BUFF_PROGRESS: u64 = 423_777;
pub(crate) const PROSKILL_1_SWITCH_LOCK: u64 = 611_700;
pub(crate) const PROSKILL_1_RECOVERY: u64 = 1_438_672;
pub(crate) const PROSKILL_1_HIT: u64 = 429_237;

pub(crate) const PROSKILL_2_HITS: &[u64] = &[1_004_348];
pub(crate) const PROSKILL_2_BUFF_PROGRESS: u64 = 938_194;
pub(crate) const PROSKILL_2_SWITCH_LOCK: u64 = 1_309_400;
pub(crate) const PROSKILL_2_RECOVERY: u64 = 1_417_949;
pub(crate) const PROSKILL_2_HIT: u64 = 1_004_348;

pub(crate) const BURST1_HITS: &[u64] = &[
    2_856_699, 2_940_793, 3_021_996, 3_103_198, 3_185_962, 3_279_656, 3_374_913, 3_470_168,
    3_568_547, 3_804_156,
];
pub(crate) const BURST1_SWITCH_LOCK: u64 = 4_059_677;
pub(crate) const BURST1_RECOVERY: u64 = 4_115_238;

pub(crate) const BURST2_HITS: &[u64] = &[948_097];
pub(crate) const BURST2_SWITCH_LOCK: u64 = 1_540_640;
pub(crate) const BURST2_RECOVERY: u64 = 1_568_251;
pub(crate) const BURST2_HIT: u64 = 948_097;

pub(crate) const QTE_HITS: &[u64] = &[704_643];
pub(crate) const QTE_SWITCH_LOCK: u64 = 875_734;
pub(crate) const QTE_RECOVERY: u64 = 1_424_255;

pub(crate) const PLUNGE_HIT: u64 = 97_189;
pub(crate) const PLUNGE_RECOVERY: u64 = 790_528;

pub(crate) const RIPOSTE_HIT: u64 = 251_899;
pub(crate) const RIPOSTE_RECOVERY: u64 = 676_566;

pub(crate) const EVADE_FORWARD_RECOVERY: u64 = 431_326;
pub(crate) const EVADE_BACK_RECOVERY: u64 = 499_987;

pub(super) fn pause(effect: &str) -> u64 {
    match effect {
        "GE_Player_Akane_Melee2_Damage" | "GE_Player_Akane_Melee3_Damage" => 19_800,
        "GE_Player_Akane_Melee3_1_Damage" => 18_000,
        _ => 0,
    }
}
