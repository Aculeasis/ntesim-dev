//! Consumed packet coefficients and metadata; native values in millionths.
use super::{DamageType, Packet, SourceCategory};

// Consumed by Chaos ordinary packet Cycle accrual, independently of ChargeAdd.
pub(super) fn cycle_milli(id: &str) -> Option<u64> {
    let id = match id {
        "GE_Player_Chaos_Melee2_Damage_Ultra" => "GE_Player_Chaos_Melee2_1_Damage",
        "GE_Player_Chaos_Melee2_1_Damage_Ultra" => "GE_Player_Chaos_Melee2_Damage",
        _ => id.strip_suffix("_Ultra").unwrap_or(id),
    };
    Some(match id {
        "GE_Player_Chaos_Melee1_Damage" => 1519,
        "GE_Player_Chaos_Melee2_Damage" => 1256,
        "GE_Player_Chaos_Melee2_1_Damage" => 3247,
        "GE_Player_Chaos_Melee3_Damage" => 1197,
        "GE_Player_Chaos_Melee4_Damage" => 2231,
        "GE_Player_Chaos_Melee4_1_Damage" => 848,
        "GE_Player_Chaos_Melee4_2_Damage" => 7159,
        "GE_Player_Chaos_Melee5_Damage" => 2074,
        "GE_Player_Chaos_Melee5_1_Damage" => 6649,
        "GE_Player_Chaos_Skill1_Damage" => 3353,
        "GE_Player_Chaos_Skill2_Damage" => 8847,
        "GE_Player_Chaos_Branch1_Damage" => 4800,
        "GE_Player_Chaos_Branch2_Damage" => 3900,
        "GE_Player_Chaos_Airattack_Damage" => 3800,
        "GE_Player_Chaos_PerfectEvadeAttack1_Damage" => 4400,
        "GE_Player_Chaos_QTE1_Damage"
        | "GE_Player_Chaos_UltraSkill1_Damage"
        | "GE_Player_Chaos_UltraSkill2_Damage" => 0,
        _ => return None,
    })
}

pub(super) const PACKETS: &[Packet] = &[
    Packet {
        native_id: "GE_Player_Chaos_Melee1_Damage",
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: &[
            182000, 197000, 212000, 229000, 248000, 267000, 289000, 312000, 337000, 364000, 393000,
            424000, 458000,
        ],
        def: &[],
        hp: &[],
        charge: 913000,
        unbalance: 181000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_Melee2_1_Damage",
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: &[
            390000, 421000, 455000, 491000, 531000, 573000, 619000, 668000, 722000, 780000, 842000,
            909000, 982000,
        ],
        def: &[],
        hp: &[],
        charge: 1951000,
        unbalance: 386000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_Melee2_Damage",
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: &[
            151000, 163000, 176000, 190000, 205000, 222000, 240000, 259000, 279000, 302000, 326000,
            352000, 380000,
        ],
        def: &[],
        hp: &[],
        charge: 754000,
        unbalance: 149000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_Melee3_Damage",
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: &[
            144000, 156000, 168000, 181000, 196000, 212000, 229000, 247000, 267000, 288000, 311000,
            336000, 363000,
        ],
        def: &[],
        hp: &[],
        charge: 719000,
        unbalance: 142000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_Melee4_1_Damage",
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: &[
            102000, 110000, 119000, 128000, 139000, 150000, 162000, 175000, 189000, 204000, 220000,
            238000, 257000,
        ],
        def: &[],
        hp: &[],
        charge: 510000,
        unbalance: 101000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_Melee4_2_Damage",
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: &[
            860000, 929000, 1003000, 1083000, 1170000, 1264000, 1365000, 1474000, 1592000, 1719000,
            1857000, 2005000, 2166000,
        ],
        def: &[],
        hp: &[],
        charge: 4300000,
        unbalance: 851000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_Melee4_Damage",
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: &[
            268000, 289000, 313000, 338000, 365000, 394000, 425000, 459000, 496000, 536000, 579000,
            625000, 675000,
        ],
        def: &[],
        hp: &[],
        charge: 1340000,
        unbalance: 265000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_Melee5_1_Damage",
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: &[
            798000, 862000, 931000, 1005000, 1086000, 1173000, 1266000, 1368000, 1477000, 1595000,
            1723000, 1861000, 2009000,
        ],
        def: &[],
        hp: &[],
        charge: 3994000,
        unbalance: 790000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_Melee5_Damage",
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: &[
            249000, 269000, 290000, 314000, 339000, 366000, 395000, 427000, 461000, 498000, 538000,
            581000, 627000,
        ],
        def: &[],
        hp: &[],
        charge: 1246000,
        unbalance: 246000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_QTE1_Damage",
        ability_id: "GA_Chaos071_QTE",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::H,
        atk: &[
            2000000, 2160000, 2333000, 2519000, 2721000, 2939000, 3174000, 3428000, 3702000,
            3998000, 4318000, 4663000, 5036000,
        ],
        def: &[],
        hp: &[],
        charge: 6400000,
        unbalance: 2500000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_Skill1_Damage",
        ability_id: "GA_Chaos071_Skill",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::E,
        atk: &[
            825000, 891000, 962000, 1039000, 1122000, 1212000, 1309000, 1414000, 1527000, 1649000,
            1781000, 1924000, 2077000,
        ],
        def: &[],
        hp: &[],
        charge: 2006000,
        unbalance: 412000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_Skill2_Damage",
        ability_id: "GA_Chaos071_Skill",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::E,
        atk: &[
            2175000, 2349000, 2537000, 2740000, 2959000, 3196000, 3451000, 3728000, 4026000,
            4348000, 4696000, 5071000, 5477000,
        ],
        def: &[],
        hp: &[],
        charge: 5294000,
        unbalance: 1088000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_UltraSkill1_Damage",
        ability_id: "GA_Chaos071_UltraSkill",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::Q,
        atk: &[
            1273000, 1375000, 1485000, 1604000, 1732000, 1870000, 2020000, 2182000, 2356000,
            2545000, 2748000, 2968000, 3206000,
        ],
        def: &[],
        hp: &[],
        charge: 0,
        unbalance: 239000,
        fixed_crit: 0,
    },
    Packet {
        native_id: "GE_Player_Chaos_UltraSkill2_Damage",
        ability_id: "GA_Chaos071_UltraSkill",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::Q,
        atk: &[
            2909000, 3142000, 3393000, 3665000, 3958000, 4274000, 4616000, 4986000, 5384000,
            5815000, 6280000, 6783000, 7325000,
        ],
        def: &[],
        hp: &[],
        charge: 0,
        unbalance: 545000,
        fixed_crit: 0,
    },
];

// Only the nine transformed Normal rows reached by Dread Echo are retained.
// Native counterparts share these arrays/channels; stage2 explicitly swaps them.
const fn transformed(native_id: &'static str, source: usize) -> Packet {
    Packet {
        native_id,
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: PACKETS[source].atk,
        def: &[],
        hp: &[],
        charge: PACKETS[source].charge,
        unbalance: PACKETS[source].unbalance,
        fixed_crit: 0,
    }
}

pub(super) const TRANSFORMED_NORMALS: &[Packet] = &[
    transformed("GE_Player_Chaos_Melee1_Damage_Ultra", 0),
    transformed("GE_Player_Chaos_Melee2_Damage_Ultra", 1),
    transformed("GE_Player_Chaos_Melee2_1_Damage_Ultra", 2),
    transformed("GE_Player_Chaos_Melee3_Damage_Ultra", 3),
    transformed("GE_Player_Chaos_Melee4_Damage_Ultra", 6),
    transformed("GE_Player_Chaos_Melee4_1_Damage_Ultra", 4),
    transformed("GE_Player_Chaos_Melee4_2_Damage_Ultra", 5),
    transformed("GE_Player_Chaos_Melee5_Damage_Ultra", 8),
    transformed("GE_Player_Chaos_Melee5_1_Damage_Ultra", 7),
];

// Only held Verdict's consumed rows; native ordinary/_Ultra counterparts agree.
const BRANCH1: &[i64] = &[
    575000, 621000, 671000, 724000, 782000, 845000, 912000, 985000, 1064000, 1149000, 1241000,
    1341000, 1448000,
];
const BRANCH2: &[i64] = &[
    4000000, 4320000, 4666000, 5039000, 5442000, 5877000, 6347000, 6855000, 7404000, 7996000,
    8636000, 9327000, 10073000,
];
const fn branch(native_id: &'static str, full: bool) -> Packet {
    Packet {
        native_id,
        ability_id: "GA_Chaos071_Melee",
        damage_type: DamageType::Lakshana,
        source_category: SourceCategory::A,
        atk: if full { BRANCH2 } else { BRANCH1 },
        def: &[],
        hp: &[],
        charge: if full { 2350000 } else { 2900000 },
        unbalance: if full { 450000 } else { 600000 },
        fixed_crit: 0,
    }
}
pub(super) const VERDICT: &[Packet] = &[
    branch("GE_Player_Chaos_Branch1_Damage", false),
    branch("GE_Player_Chaos_Branch1_Damage_Ultra", false),
    branch("GE_Player_Chaos_Branch2_Damage", true),
    branch("GE_Player_Chaos_Branch2_Damage_Ultra", true),
];

const PLUNGE: &[i64] = &[
    450000, 486000, 525000, 567000, 612000, 661000, 714000, 771000, 833000, 900000, 972000,
    1049000, 1133000,
];
const RIPOSTE: &[i64] = &[
    1060000, 1145000, 1236000, 1335000, 1442000, 1557000, 1682000, 1817000, 1962000, 2119000,
    2288000, 2472000, 2669000,
];
const fn secondary(native_id: &'static str, plunge: bool) -> Packet {
    Packet {
        native_id,
        ability_id: if plunge {
            "GA_Chaos071_Melee"
        } else {
            "GA_Chaos071_PerfectEvade"
        },
        damage_type: DamageType::Lakshana,
        source_category: if plunge {
            SourceCategory::X
        } else {
            SourceCategory::Z
        },
        atk: if plunge { PLUNGE } else { RIPOSTE },
        def: &[],
        hp: &[],
        charge: if plunge { 2300000 } else { 2700000 },
        unbalance: if plunge { 500000 } else { 2500000 },
        fixed_crit: 0,
    }
}
pub(super) const SECONDARY: &[Packet] = &[
    secondary("GE_Player_Chaos_Airattack_Damage", true),
    secondary("GE_Player_Chaos_Airattack_Damage_Ultra", true),
    secondary("GE_Player_Chaos_PerfectEvadeAttack1_Damage", false),
];

// Justice in Time's concrete owner end callback consumes this fixed native row.
pub(super) const REMORA_END: &[Packet] = &[Packet {
    native_id: "GE_Reaction_3_new_1071_Damage",
    ability_id: "GA_Chaos071_UltraSkill",
    damage_type: DamageType::Lakshana,
    source_category: SourceCategory::Q,
    atk: &[8000000; 13],
    def: &[],
    hp: &[],
    charge: 0,
    unbalance: 0,
    fixed_crit: 0,
}];
