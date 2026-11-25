//! Consumed constants and 13-coordinate `DT_AkaneEffectFigure` CoefAdd tables for Akane Rin.
use crate::simulation::ExactFraction;

pub(crate) struct Parameters {
    pub(crate) max_string_energy: u8,
    pub(crate) hold_threshold_us: u64,
    pub(crate) skill_cooldown_us: u64,
    pub(crate) burst_cooldown_us: u64,
    pub(crate) burst_stage2_cooldown_us: u64,
    pub(crate) mode_duration_us: u64,
    pub(crate) cadenza_unlock_us: u64,
    pub(crate) cadenza_duration_us: u64,
    pub(crate) anthem_duration_us: u64,
    pub(crate) anthem_atk_bonus: ExactFraction,
    pub(crate) perfect_pitch_tick_us: u64,
    pub(crate) perfect_pitch_icd_us: u64,
    pub(crate) a1_heal_base_atk_ratio: ExactFraction,
    pub(crate) a1_def_bonus: ExactFraction,
    pub(crate) a1_duration_us: u64,
    pub(crate) a2_res_reduction: ExactFraction,
    pub(crate) a3_press_fixed_ratio: ExactFraction,
    pub(crate) a3_pull_duration_us: u64,
    pub(crate) a4_crit_dmg_bonus: ExactFraction,
    pub(crate) a4_duration_us: u64,
    pub(crate) a6_def_ignore: ExactFraction,
    pub(crate) reverb_dmg_bonus: ExactFraction,
    pub(crate) reverb_a5_dmg_bonus: ExactFraction,
    pub(crate) r3_upskill_fixed_ratio: ExactFraction,
    pub(crate) r3_ultra2_fixed_ratio: ExactFraction,
    pub(crate) r6_string_refund: u8,
    pub(crate) r6_crit_dmg_bonus: ExactFraction,
    pub(crate) r6_duration_us: u64,
}

pub(crate) const PARAMETERS: Parameters = Parameters {
    max_string_energy: 30,
    hold_threshold_us: 200_000,
    skill_cooldown_us: 16_000_000,
    burst_cooldown_us: 20_000_000,
    burst_stage2_cooldown_us: 100_000,
    mode_duration_us: 25_000_000,
    cadenza_unlock_us: 4_000_000,
    cadenza_duration_us: 20_000_000,
    anthem_duration_us: 20_000_000,
    anthem_atk_bonus: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    perfect_pitch_tick_us: 3_000_000,
    perfect_pitch_icd_us: 1_000_000,
    a1_heal_base_atk_ratio: ExactFraction {
        numerator: 10,
        denominator: 1,
    },
    a1_def_bonus: ExactFraction {
        numerator: 1,
        denominator: 2,
    },
    a1_duration_us: 15_000_000,
    a2_res_reduction: ExactFraction {
        numerator: 3,
        denominator: 25,
    },
    a3_press_fixed_ratio: ExactFraction {
        numerator: 10,
        denominator: 1,
    },
    a3_pull_duration_us: 8_000_000,
    a4_crit_dmg_bonus: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    a4_duration_us: 15_000_000,
    a6_def_ignore: ExactFraction {
        numerator: 4,
        denominator: 25,
    },
    reverb_dmg_bonus: ExactFraction {
        numerator: 3,
        denominator: 5,
    },
    reverb_a5_dmg_bonus: ExactFraction {
        numerator: 9,
        denominator: 10,
    },
    r3_upskill_fixed_ratio: ExactFraction {
        numerator: 8,
        denominator: 1,
    },
    r3_ultra2_fixed_ratio: ExactFraction {
        numerator: 10,
        denominator: 1,
    },
    r6_string_refund: 10,
    r6_crit_dmg_bonus: ExactFraction {
        numerator: 16,
        denominator: 25,
    },
    r6_duration_us: 25_000_000,
};

// DT_AkaneEffectFigure millionths (Levels 1..13; 1_000_000 = 1.000).
const UPSKILL_SHORT_COEF_ADD: [u64; 13] = [
    1_125_000, 1_215_000, 1_312_200, 1_417_200, 1_530_600, 1_653_000, 1_785_200, 1_928_100,
    2_082_300, 2_248_900, 2_428_800, 2_623_100, 2_832_900,
];

const MELEE1_COEF_ADD: [u64; 13] = [
    59_600, 64_400, 69_500, 75_100, 81_100, 87_600, 94_600, 102_100, 110_300, 119_100, 128_700,
    139_000, 150_100,
];
const MELEE1_COEF_ADD_A5: [u64; 13] = [
    95_400, 103_000, 111_300, 120_200, 129_800, 140_200, 151_400, 163_500, 176_600, 190_700,
    206_000, 222_400, 240_200,
];

const MELEE2_COEF_ADD: [u64; 13] = [
    87_600, 94_600, 102_200, 110_400, 119_200, 128_700, 139_000, 150_100, 162_100, 175_100,
    189_100, 204_300, 220_600,
];
const MELEE2_COEF_ADD_A5: [u64; 13] = [
    140_200, 151_400, 163_500, 176_600, 190_700, 206_000, 222_500, 240_300, 259_500, 280_300,
    302_700, 326_900, 353_000,
];

const MELEE3_COEF_ADD: [u64; 13] = [
    41_200, 44_500, 48_100, 51_900, 56_100, 60_500, 65_400, 70_600, 76_300, 82_400, 88_900, 96_100,
    103_700,
];
const MELEE3_COEF_ADD_A5: [u64; 13] = [
    65_900, 71_200, 76_900, 83_000, 89_700, 96_800, 104_600, 112_900, 122_000, 131_700, 142_300,
    153_700, 165_900,
];

const MELEE3_1_COEF_ADD: [u64; 13] = [
    54_000, 58_300, 63_000, 68_000, 73_500, 79_300, 85_700, 92_500, 100_000, 107_900, 116_600,
    125_900, 136_000,
];
const MELEE3_1_COEF_ADD_A5: [u64; 13] = [
    86_400, 93_300, 100_800, 108_800, 117_500, 126_900, 137_100, 148_100, 159_900, 172_700,
    186_500, 201_500, 217_600,
];

const MELEE4_COEF_ADD: [u64; 13] = [
    1_698_800, 1_834_700, 1_981_500, 2_140_000, 2_311_200, 2_496_100, 2_695_800, 2_911_400,
    3_144_400, 3_395_900, 3_667_600, 3_961_000, 4_277_900,
];
const MELEE4_COEF_ADD_A5: [u64; 13] = [
    2_718_100, 2_935_500, 3_170_400, 3_424_000, 3_697_900, 3_993_800, 4_313_300, 4_658_300,
    5_031_000, 5_433_500, 5_868_200, 6_337_600, 6_844_600,
];

const RIPOSTE_COEF_ADD: [u64; 13] = [
    550_000, 594_000, 641_500, 692_800, 748_300, 808_100, 872_800, 942_600, 1_018_000, 1_099_500,
    1_187_400, 1_282_400, 1_385_000,
];
const RIPOSTE_COEF_ADD_A5: [u64; 13] = [
    880_000, 950_400, 1_026_400, 1_108_500, 1_197_200, 1_293_000, 1_396_400, 1_508_200, 1_628_800,
    1_759_100, 1_899_900, 2_051_800, 2_216_000,
];

fn lookup_coord(table: &[u64; 13], array_index: usize) -> Result<ExactFraction, String> {
    let raw = table
        .get(array_index)
        .copied()
        .ok_or("Akane CoefAdd coordinate out of bounds (expected 0..13)")?;
    ExactFraction::new(u128::from(raw), 1_000_000)
}

pub(crate) fn upskill_short_coef_add(array_index: usize) -> Result<ExactFraction, String> {
    lookup_coord(&UPSKILL_SHORT_COEF_ADD, array_index)
}

pub(crate) fn overtone_coef_add(
    damage_effect_id: &str,
    array_index: usize,
    a5: bool,
) -> Result<Option<ExactFraction>, String> {
    let table = match (damage_effect_id, a5) {
        ("GE_Player_Akane_Melee1_Damage", false) => &MELEE1_COEF_ADD,
        ("GE_Player_Akane_Melee1_Damage", true) => &MELEE1_COEF_ADD_A5,
        ("GE_Player_Akane_Melee2_Damage", false) => &MELEE2_COEF_ADD,
        ("GE_Player_Akane_Melee2_Damage", true) => &MELEE2_COEF_ADD_A5,
        ("GE_Player_Akane_Melee3_Damage", false) => &MELEE3_COEF_ADD,
        ("GE_Player_Akane_Melee3_Damage", true) => &MELEE3_COEF_ADD_A5,
        ("GE_Player_Akane_Melee3_1_Damage", false) => &MELEE3_1_COEF_ADD,
        ("GE_Player_Akane_Melee3_1_Damage", true) => &MELEE3_1_COEF_ADD_A5,
        ("GE_Player_Akane_Melee4_Damage", false) => &MELEE4_COEF_ADD,
        ("GE_Player_Akane_Melee4_Damage", true) => &MELEE4_COEF_ADD_A5,
        ("GE_Player_Akane_PerfectEvadeAttack_Damage", false) => &RIPOSTE_COEF_ADD,
        ("GE_Player_Akane_PerfectEvadeAttack_Damage", true) => &RIPOSTE_COEF_ADD_A5,
        _ => return Ok(None),
    };
    lookup_coord(table, array_index).map(Some)
}
