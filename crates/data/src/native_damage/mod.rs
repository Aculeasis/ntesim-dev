//! Exact coefficients, energy, Break and classification for reached packets.
//! Native numeric units are stored as integer millionths, without rounding.
//! Unbound source candidates and destruction/localization fields are cache-only.
use crate::native::Table;
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

mod adler;
mod akane;
mod blackbird;
mod cang;
mod chaos071;
mod chiichan073;
mod daffodill;
mod edgar;
mod fadia;
mod female046;
mod haniel;
mod hathor;
mod jin;
mod kuhara;
mod lacrimosa;
mod mint019;
mod mitsuki;
mod nanally;
mod oneiroi;
mod radio072;
mod sagiri;
mod shinku;
mod skia;
mod zankou;

const PACKET_GROUPS: &[&[Packet]] = &[
    adler::PACKETS,
    akane::PACKETS,
    blackbird::PACKETS,
    cang::PACKETS,
    chaos071::PACKETS,
    chaos071::TRANSFORMED_NORMALS,
    chaos071::VERDICT,
    chaos071::SECONDARY,
    chaos071::REMORA_END,
    chiichan073::PACKETS,
    daffodill::PACKETS,
    edgar::PACKETS,
    fadia::PACKETS,
    female046::PACKETS,
    haniel::PACKETS,
    hathor::PACKETS,
    jin::PACKETS,
    kuhara::PACKETS,
    lacrimosa::PACKETS,
    mint019::PACKETS,
    mitsuki::PACKETS,
    nanally::PACKETS,
    oneiroi::PACKETS,
    radio072::PACKETS,
    sagiri::PACKETS,
    shinku::PACKETS,
    skia::PACKETS,
    zankou::PACKETS,
];

pub const TABLE_NAME: &str = "typed_native_damage";
pub const SOURCE_ID: &str = "table:typed_native_damage";
const OVERLAY_TABLE: &str = "DT_SkillDamageData";

fn cycle_milli(id: &str) -> Option<u64> {
    akane::cycle_milli(id)
        .or_else(|| cang::cycle_milli(id))
        .or_else(|| blackbird::cycle_milli(id))
        .or_else(|| chaos071::cycle_milli(id))
        .or_else(|| chiichan073::cycle_milli(id))
        .or_else(|| daffodill::cycle_milli(id))
        .or_else(|| edgar::cycle_milli(id))
        .or_else(|| fadia::cycle_milli(id))
        .or_else(|| haniel::cycle_milli(id))
        .or_else(|| hathor::cycle_milli(id))
        .or_else(|| jin::cycle_milli(id))
        .or_else(|| kuhara::cycle_milli(id))
        .or_else(|| lacrimosa::cycle_milli(id))
        .or_else(|| nanally::cycle_milli(id))
        .or_else(|| zankou::cycle_milli(id))
}

enum DamageType {
    Chaos,
    Cosmos,
    Incantation,
    Lakshana,
    Nature,
    Psyche,
    Psychically,
}

impl DamageType {
    fn native(&self) -> &'static str {
        match self {
            Self::Chaos => "EExecutionDamageTypeEX::DAMAGE_TYPE_CHAOS",
            Self::Cosmos => "EExecutionDamageTypeEX::DAMAGE_TYPE_COSMOS",
            Self::Incantation => "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION",
            Self::Lakshana => "EExecutionDamageTypeEX::DAMAGE_TYPE_LAKSHANA",
            Self::Nature => "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE",
            Self::Psyche => "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHE",
            Self::Psychically => "EExecutionDamageTypeEX::DAMAGE_TYPE_PSYCHICALLY",
        }
    }
}

enum SourceCategory {
    A,
    E,
    H,
    Q,
    R,
    X,
    Z,
}

impl SourceCategory {
    fn native(&self) -> &'static str {
        match self {
            Self::A => "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A",
            Self::E => "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E",
            Self::H => "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_H",
            Self::Q => "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q",
            Self::R => "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_R",
            Self::X => "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_X",
            Self::Z => "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Z",
        }
    }
}

struct Packet {
    native_id: &'static str,
    ability_id: &'static str,
    damage_type: DamageType,
    source_category: SourceCategory,
    atk: &'static [i64],
    def: &'static [i64],
    hp: &'static [i64],
    charge: i64,
    unbalance: i64,
    fixed_crit: i64,
}

fn decimal(amount: i64) -> Value {
    let text = format!("{}.{:06}", amount / 1_000_000, amount % 1_000_000);
    serde_json::from_str(text.trim_end_matches('0').trim_end_matches('.'))
        .expect("typed nonnegative packet decimal")
}

pub fn transport() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let rows = PACKET_GROUPS.iter().flat_map(|group| group.iter()).map(|packet| {
            let array = |values: &[i64]| values.iter().copied().map(decimal).collect::<Vec<_>>();
            let mut row = json!({
                "GAName":packet.ability_id,"DamageTypeEX":packet.damage_type.native(),
                "DamageSourceCategory":packet.source_category.native(),
                "AtkRateBaseArray":array(packet.atk),"DefRateBaseArray":array(packet.def),
                "HPRateBaseArray":array(packet.hp),"ChargeAdd":decimal(packet.charge),
                "UnbalValue":decimal(packet.unbalance),"FixedCritRate":decimal(packet.fixed_crit)
            });
            if let Some(milli) = cycle_milli(packet.native_id) {
                row["HeterochromeAdd"] = decimal((milli * 1000) as i64);
            }
            (packet.native_id, row)
        }).collect::<BTreeMap<_,_>>();
        json!({"schema_version":1,"source_id":SOURCE_ID,"rows":rows})
    })
}

pub(crate) fn table(tables: &BTreeMap<String, Value>) -> Result<Table<'_>, String> {
    if let Some(value) = tables.get(OVERLAY_TABLE) {
        Table::new(OVERLAY_TABLE, value)
    } else {
        Table::new(
            TABLE_NAME,
            tables.get(TABLE_NAME).unwrap_or_else(|| transport()),
        )
    }
}

pub fn rows(
    tables: &BTreeMap<String, Value>,
) -> Result<(&'static str, &Map<String, Value>), String> {
    let name = if tables.contains_key(OVERLAY_TABLE) {
        OVERLAY_TABLE
    } else {
        TABLE_NAME
    };
    Ok((name, table(tables)?.rows()))
}

pub fn lookup<'a>(
    tables: &'a BTreeMap<String, Value>,
    id: &str,
) -> Result<(&'a str, &'a Value), String> {
    table(tables)?.lookup(id)
}

/// Only reached, owner-bound Heterochrome rows are retained. Explicit overlays
/// must provide the field themselves, including an explicit zero.
pub fn cycle_add(
    tables: &BTreeMap<String, Value>,
    id: &str,
) -> Result<Option<crate::native::NativeScalar>, String> {
    let Some(milli) = cycle_milli(id) else {
        return Ok(None);
    };
    let value = if tables.contains_key(OVERLAY_TABLE) || tables.contains_key(TABLE_NAME) {
        lookup(tables, id)?
            .1
            .get("HeterochromeAdd")
            .cloned()
            .ok_or_else(|| format!("explicit damage overlay {id} has no HeterochromeAdd"))?
    } else {
        decimal((milli * 1000) as i64)
    };
    crate::native::parse_native_scalar(&value).map(Some)
}

/// Explicit overlay for research/tests; missing rows and fields never merge.
pub fn overlay() -> Value {
    let mut value = transport().clone();
    value["source_id"] = json!("table:DT_SkillDamageData");
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_consumed_packet_fields_and_exact_coordinates_are_retained() {
        let rows = transport()["rows"].as_object().unwrap();
        assert_eq!(rows.len(), 625);
        for (id, row) in rows {
            let expected = if id.starts_with("GE_Player_Cang_")
                || id.starts_with("GE_Player_BlackBird_")
                || id.starts_with("GE_Player_Chaos_")
                || akane::cycle_milli(id).is_some()
                || chiichan073::cycle_milli(id).is_some()
                || daffodill::cycle_milli(id).is_some()
                || edgar::cycle_milli(id).is_some()
                || fadia::cycle_milli(id).is_some()
                || haniel::cycle_milli(id).is_some()
                || hathor::cycle_milli(id).is_some()
                || jin::cycle_milli(id).is_some()
                || kuhara::cycle_milli(id).is_some()
                || lacrimosa::cycle_milli(id).is_some()
                || nanally::cycle_milli(id).is_some()
                || zankou::cycle_milli(id).is_some()
            {
                10
            } else {
                9
            };
            assert_eq!(row.as_object().unwrap().len(), expected);
            for field in ["AtkRateBaseArray", "DefRateBaseArray", "HPRateBaseArray"] {
                assert!(matches!(row[field].as_array().unwrap().len(), 0 | 13));
            }
        }
        let mint = &rows["GE_Player_Mint_Skill1_Damage_New"];
        assert_eq!(
            crate::native::Scalar::parse(&mint["AtkRateBaseArray"][7])
                .unwrap()
                .receipt(),
            json!({"numerator":"771","denominator":"500"})
        );
        assert_eq!(mint["ChargeAdd"], json!(2.45));
        assert_eq!(mint["UnbalValue"], json!(0.5));
        assert!(mint["DefRateBaseArray"].as_array().unwrap().is_empty());
        assert_eq!(mint["FixedCritRate"], 0);
        let skia_entry = &rows["GE_Player_Cang_Skia_Damage"];
        assert_eq!(skia_entry["GAName"], "GA_Cang_Melee");
        for (coordinate, expected) in [(0, json!(0.675)), (7, json!(1.157)), (8, json!(1.249))] {
            assert_eq!(skia_entry["AtkRateBaseArray"][coordinate], expected);
        }
        assert_eq!(skia_entry["ChargeAdd"], json!(2.3));
        assert_eq!(skia_entry["HeterochromeAdd"], json!(3.8));
        assert_eq!(skia_entry["UnbalValue"], json!(0.5));
        let fadia = &rows["GE_Player_Cang_Fadia_Damage"];
        assert_eq!(fadia["GAName"], "GA_Cang_Melee");
        for (coordinate, expected) in [(0, json!(0.93)), (7, json!(1.594)), (8, json!(1.721))] {
            assert_eq!(fadia["AtkRateBaseArray"][coordinate], expected);
        }
        assert_eq!(fadia["ChargeAdd"], json!(3.1));
        assert_eq!(fadia["HeterochromeAdd"], json!(5.2));
        assert_eq!(fadia["UnbalValue"], json!(0.6));
        let zero_support = &rows["GE_Player_Female046_QTE2_Damage"];
        assert_eq!(zero_support["AtkRateBaseArray"][0], json!(1.247));
        assert_eq!(
            crate::native::Scalar::parse(&zero_support["AtkRateBaseArray"][12])
                .unwrap()
                .receipt(),
            json!({"numerator":"157","denominator":"50"}),
        );
        assert_eq!(zero_support["ChargeAdd"], json!(4.801));
        assert_eq!(zero_support["UnbalValue"], json!(1.559));
        let riposte = &rows["GE_Player_Nanally_PerfectEvadeAttack_Damage"];
        assert_eq!(riposte["GAName"], "GA_Nanally_ExtremEvadeAtk");
        assert_eq!(
            riposte["DamageSourceCategory"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Z"
        );
        assert_eq!(riposte["AtkRateBaseArray"][7], json!(2.845));
        assert_eq!(
            rows["GE_Player_Nanally_Cat_PerfectEvadeAttack_Damage"]["GAName"],
            "GA_Nanally_UltraSkill"
        );
        assert!(!rows.contains_key("Buff_3_Dam"));
    }

    #[test]
    fn hathor_held_rows_retain_exact_consumed_source_coordinates() {
        let rows = transport()["rows"].as_object().unwrap();
        for (suffix, coefficients, charge, cycle, unbalance) in [
            (
                "Skill1Power",
                [json!(0.556), json!(0.953), json!(1.029)],
                json!(1.359),
                json!(2.267),
                json!(0.27),
            ),
            (
                "Skill2",
                [json!(0.102), json!(0.175), json!(0.189)],
                json!(0.25),
                json!(0.416),
                json!(0.05),
            ),
            (
                "Skill3",
                [json!(1.225), json!(2.099), json!(2.267)],
                json!(2.995),
                json!(4.995),
                json!(0.596),
            ),
            (
                "Skill4",
                [json!(0.552), json!(0.946), json!(1.022)],
                json!(1.348),
                json!(2.248),
                json!(0.268),
            ),
        ] {
            let row = &rows[&format!("GE_Player_Hathor_{suffix}_Damage")];
            assert_eq!(row["GAName"], "GA_Hathor_Skill");
            for (coordinate, value) in [0, 7, 8].into_iter().zip(coefficients) {
                assert_eq!(row["AtkRateBaseArray"][coordinate], value);
            }
            assert_eq!(row["ChargeAdd"], charge);
            assert_eq!(row["HeterochromeAdd"], cycle);
            assert_eq!(row["UnbalValue"], unbalance);
        }
    }

    #[test]
    fn hathor_cyclone_rows_bind_child_identity_and_exact_talent_resources() {
        let rows = transport()["rows"].as_object().unwrap();
        for (stage, coefficients, charge, cycle, unbalance) in [
            (5, [0.429, 0.735, 0.794], 1.271, 2.129, 0.257),
            (6, [1.432, 2.454, 2.651], 2.435, 4.046, 0.501),
            (7, [1.136, 1.947, 2.103], 1.931, 3.208, 0.397),
            (8, [1.432, 2.454, 2.651], 2.435, 4.046, 0.501),
            (9, [1.969, 3.375, 3.644], 3.115, 5.192, 0.609),
            (10, [3.531, 6.052, 6.536], 5.585, 9.308, 1.091),
        ] {
            let row = &rows[&format!("GE_Player_Hathor_Skill{stage}_Damage")];
            assert_eq!(row["GAName"], "GA_Hathor_Skill2");
            for (coordinate, value) in [0, 7, 8].into_iter().zip(coefficients) {
                assert_eq!(row["AtkRateBaseArray"][coordinate], json!(value));
            }
            assert_eq!(row["ChargeAdd"], json!(charge));
            assert_eq!(row["HeterochromeAdd"], json!(cycle));
            assert_eq!(row["UnbalValue"], json!(unbalance));
        }
    }

    #[test]
    fn hathor_secondary_rows_keep_distinct_riposte_and_support_resources() {
        let rows = transport()["rows"].as_object().unwrap();
        for (suffix, ga, coefficients, charge, cycle, unbalance) in [
            (
                "PerfectEvadeAttack",
                "GA_Hathor_PerfectEvade",
                [0.680, 1.165, 1.259],
                1.717,
                json!(2.851),
                0.859,
            ),
            (
                "PerfectEvadeAttack1",
                "GA_Hathor_PerfectEvade",
                [1.300, 2.228, 2.406],
                3.283,
                json!(5.449),
                1.641,
            ),
            (
                "QTE2",
                "GA_Hathor_QTE",
                [1.361, 2.333, 2.519],
                5.58,
                json!(0),
                1.701,
            ),
        ] {
            let row = &rows[&format!("GE_Player_Hathor_{suffix}_Damage")];
            assert_eq!(row["GAName"], ga);
            for (coordinate, value) in [0, 7, 8].into_iter().zip(coefficients) {
                assert_eq!(row["AtkRateBaseArray"][coordinate], json!(value));
            }
            assert_eq!(row["ChargeAdd"], json!(charge));
            assert_eq!(row["HeterochromeAdd"], cycle);
            assert_eq!(row["UnbalValue"], json!(unbalance));
        }
    }

    #[test]
    fn hathor_airattack_keeps_frozen_charge_cycle_and_normal_talent_coordinates() {
        let row = &transport()["rows"]["GE_Player_Hathor_AirAttack_Damage"];
        assert_eq!(row["ChargeAdd"], json!(6.3));
        assert_eq!(row["HeterochromeAdd"], json!(10.4));
        for (index, value) in [(0, 1.250), (7, 2.142), (8, 2.314)] {
            assert_eq!(row["AtkRateBaseArray"][index], json!(value));
        }
    }

    #[test]
    fn explicit_overlay_preserves_missing_values_and_validation() {
        let key = "GE_Player_Mint_Skill1_Damage_New";
        let mut value = overlay();
        value["rows"].as_object_mut().unwrap().remove(key);
        let mut tables = BTreeMap::from([(OVERLAY_TABLE.into(), value)]);
        assert!(lookup(&tables, key).is_err());
        tables.get_mut(OVERLAY_TABLE).unwrap()["rows"][key] = json!({"GAName":"GA_Mint019_Skill"});
        assert!(lookup(&tables, key).unwrap().1.get("ChargeAdd").is_none());
        tables.get_mut(OVERLAY_TABLE).unwrap()["source_id"] = json!(SOURCE_ID);
        assert!(lookup(&tables, key).is_err());
    }
}
#[test]
fn cang_cycle_rows_retain_exact_units_zero_and_explicit_overlay_absence() {
    let tables = BTreeMap::new();
    let row = cycle_add(&tables, "GE_Player_Cang_Melee1_Damage")
        .unwrap()
        .unwrap();
    assert_eq!((row.numerator(), row.denominator()), (3771, 1000));
    assert_eq!(
        cycle_add(&tables, "GE_Player_Cang_WorldC_Damage")
            .unwrap()
            .unwrap()
            .numerator(),
        0
    );
    assert!(
        cycle_add(&tables, "GE_Player_Mint_Melee1_Damage")
            .unwrap()
            .is_none()
    );
    let mut overlay = overlay();
    overlay["rows"]["GE_Player_Cang_Melee1_Damage"]
        .as_object_mut()
        .unwrap()
        .remove("HeterochromeAdd");
    let mut tables = BTreeMap::from([("DT_SkillDamageData".into(), overlay)]);
    assert!(
        cycle_add(&tables, "GE_Player_Cang_Melee1_Damage")
            .unwrap_err()
            .contains("HeterochromeAdd")
    );
    tables.get_mut("DT_SkillDamageData").unwrap()["rows"]["GE_Player_Cang_Melee1_Damage"]["HeterochromeAdd"] =
        json!(0.125);
    let row = cycle_add(&tables, "GE_Player_Cang_Melee1_Damage")
        .unwrap()
        .unwrap();
    assert_eq!((row.numerator(), row.denominator()), (1, 8));
}

#[test]
fn blackbird_and_chaos_cycle_rows_retain_exact_channels_zero_and_overlay_requirements() {
    let tables = BTreeMap::new();
    for (id, numerator, denominator) in [
        ("GE_Player_BlackBird_Melee1_Damage", 1161, 1000),
        ("GE_Player_BlackBird_Melee2_Damage", 2391, 1000),
        ("GE_Player_BlackBird_Melee3_Damage", 369, 250),
        ("GE_Player_BlackBird_Melee4_Damage", 253, 200),
        ("GE_Player_BlackBird_Melee5_Damage", 357, 200),
        ("GE_Player_BlackBird_Skill_Damage", 13, 5),
        ("GE_Player_BlackBird_Branch1_Damage", 28, 5),
        ("GE_Player_BlackBird_QTE_Damage", 0, 1),
        ("GE_Player_BlackBird_UltraSkill_Damage", 0, 1),
        ("GE_Player_BlackBird_Evolve_AirAttack_Damage", 23, 10),
        ("GE_Player_BlackBird_Evolve_PerfectEvade_Damage", 7, 2),
        ("GE_Player_BlackBird_Evolve_QTE_Damage", 0, 1),
        ("GE_Player_Chaos_Melee1_Damage", 1519, 1000),
        ("GE_Player_Chaos_Skill2_Damage", 8847, 1000),
        ("GE_Player_Chaos_QTE1_Damage", 0, 1),
        ("GE_Player_Chaos_Melee2_Damage_Ultra", 3247, 1000),
        ("GE_Player_Chaos_Melee2_1_Damage_Ultra", 157, 125),
        ("GE_Player_Daffodill_Melee1_Damage", 1737, 1000),
        ("GE_Player_Daffodill_Melee3_3_Damage", 2733, 1000),
        ("GE_Player_Daffodill_Skill6_Damage", 567, 500),
        ("GE_Player_Daffodill_Skill4_Damage", 767, 250),
        ("GE_Player_Daffodill_UltraSkill6_Damage", 0, 1),
    ] {
        let scalar = cycle_add(&tables, id).unwrap().unwrap();
        assert_eq!(
            (scalar.numerator(), scalar.denominator()),
            (numerator, denominator)
        );
    }
    for id in [
        "GE_Player_BlackBird_Skill_Damage",
        "GE_Player_Chaos_Skill2_Damage",
    ] {
        let mut data = overlay();
        data["rows"][id]
            .as_object_mut()
            .unwrap()
            .remove("HeterochromeAdd");
        let mut tables = BTreeMap::from([("DT_SkillDamageData".into(), data)]);
        assert!(
            cycle_add(&tables, id)
                .unwrap_err()
                .contains("HeterochromeAdd")
        );
        tables.get_mut("DT_SkillDamageData").unwrap()["rows"][id]["HeterochromeAdd"] = json!(0);
        assert_eq!(cycle_add(&tables, id).unwrap().unwrap().numerator(), 0);
    }
}
