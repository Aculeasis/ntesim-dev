//! Consumed terminal breakthrough bindings for supported weapons.
//! Every group has seven exact cap/modifier coordinates. Currency and material
//! costs are acquisition evidence only; no combat or build consumer reads them.
use crate::native::Table;
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

pub const TABLE_NAME: &str = "typed_weapon_breakthrough";
pub const SOURCE_ID: &str = "table:typed_weapon_breakthrough";
const OVERLAY_TABLE: &str = "DT_ForkBreakthroughData";
pub const LEVEL_CAPS: [u32; 7] = [20, 30, 40, 50, 60, 70, 80];

pub struct Group {
    pub pack: &'static str,
    pub modifier_prefix: &'static str,
}

pub const GROUPS: &[Group] = &[
    Group {
        pack: "Breakthrough_pack_fork_elyms",
        modifier_prefix: "fork_elyms_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_Arachne",
        modifier_prefix: "fork_Arachne_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_BitGame",
        modifier_prefix: "fork_BitGame_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_BitterCake",
        modifier_prefix: "fork_BitterCake_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_BlackBook",
        modifier_prefix: "fork_BlackBook_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_BlastCandy",
        modifier_prefix: "fork_BlastCandy_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_BoxingCandy",
        modifier_prefix: "fork_BoxingCandy_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_Butterfly",
        modifier_prefix: "fork_Butterfly_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_Castle",
        modifier_prefix: "fork_Castle_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_Crowbar",
        modifier_prefix: "fork_Crowbar_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_DemonBlade",
        modifier_prefix: "fork_DemonBlade_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_Door",
        modifier_prefix: "fork_Door_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_GoldRecord",
        modifier_prefix: "fork_GoldRecord_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_GoldWool",
        modifier_prefix: "fork_GoldWool_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_Kite",
        modifier_prefix: "fork_Kite_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_KnightCandy",
        modifier_prefix: "fork_KnightCandy_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_LunarPhase",
        modifier_prefix: "fork_LunarPhase_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_MotorCandy",
        modifier_prefix: "fork_MotorCandy_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_Nakupeda",
        modifier_prefix: "fork_Nakupeda_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_NestBird",
        modifier_prefix: "fork_NestBird_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_PaperPlane",
        modifier_prefix: "fork_PaperPlane_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_PoliceRat",
        modifier_prefix: "fork_PoliceRat_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_Prokaryon",
        modifier_prefix: "fork_Prokaryon_rank3",
    },
    Group {
        pack: "Breakthrough_pack_fork_Rose",
        modifier_prefix: "fork_Rose_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_ThiefCandy",
        modifier_prefix: "fork_ThiefCandy_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_TigerTally",
        modifier_prefix: "fork_TigerTally_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_Time",
        modifier_prefix: "fork_Time_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_Whale",
        modifier_prefix: "fork_Whale_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_appliance",
        modifier_prefix: "fork_appliance_rank3",
    },
    Group {
        pack: "Breakthrough_pack_fork_bopu",
        modifier_prefix: "fork_bopu_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_dustbin",
        modifier_prefix: "fork_dustbin_rank3",
    },
    Group {
        pack: "Breakthrough_pack_fork_jiaojuan",
        modifier_prefix: "fork_jiaojuan_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_jingmotingyuan",
        modifier_prefix: "fork_jingmotingyuan_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_koinobori",
        modifier_prefix: "fork_koinobori_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_lingganzhongjiezhe",
        modifier_prefix: "fork_lingganzhongjiezhe_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_mamen",
        modifier_prefix: "fork_mamen_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_mofeikesi",
        modifier_prefix: "fork_mofeikesi_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_moon",
        modifier_prefix: "fork_moon_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_nonos",
        modifier_prefix: "fork_nonos_rank3",
    },
    Group {
        pack: "Breakthrough_pack_fork_oulaquantao",
        modifier_prefix: "fork_oulaquantao_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_prism",
        modifier_prefix: "fork_prism_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_rishi",
        modifier_prefix: "fork_rishi_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_snowman",
        modifier_prefix: "fork_snowman_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_spider",
        modifier_prefix: "fork_spider_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_tuansanlang",
        modifier_prefix: "fork_tuansanlang_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_twinbirds",
        modifier_prefix: "fork_twinbirds_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_vine",
        modifier_prefix: "fork_vine_rank3",
    },
    Group {
        pack: "Breakthrough_pack_fork_worldrain",
        modifier_prefix: "fork_worldrain_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_wuhuakuang",
        modifier_prefix: "fork_wuhuakuang_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_wushoutieyu",
        modifier_prefix: "fork_wushoutieyu_rank5",
    },
    Group {
        pack: "Breakthrough_pack_fork_yaodao",
        modifier_prefix: "fork_yaodao_rank4",
    },
    Group {
        pack: "Breakthrough_pack_fork_yuren",
        modifier_prefix: "fork_yuren_rank4",
    },
];

pub fn transport() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let rows = GROUPS.iter().flat_map(|group| {
            LEVEL_CAPS.iter().enumerate().map(move |(stage, cap)| (
                format!("{}_{stage}", group.pack),
                json!({"MaxForkLevel":cap,"ModifyPackID":format!("{}_stage{stage}", group.modifier_prefix)}),
            ))
        }).collect::<BTreeMap<_, _>>();
        json!({"schema_version":1,"source_id":SOURCE_ID,"rows":rows})
    })
}

pub(crate) fn table(tables: &BTreeMap<String, Value>) -> Result<Table<'_>, String> {
    match tables.get(OVERLAY_TABLE) {
        Some(value) => Table::new(OVERLAY_TABLE, value),
        None => Table::new(TABLE_NAME, transport()),
    }
}

/// Validated transport for consumers outside the data crate.
pub fn rows(tables: &BTreeMap<String, Value>) -> Result<&Map<String, Value>, String> {
    Ok(table(tables)?.rows())
}

/// Explicit test/research overlay; absent rows and fields are never filled in.
pub fn overlay() -> Value {
    let mut value = transport().clone();
    value["source_id"] = json!("table:DT_ForkBreakthroughData");
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn supported_weapons_reach_exactly_the_retained_breakthrough_coordinates() {
        let tables = crate::stat_projection::frozen_tables();
        let required = crate::weapon_definitions::rows(tables)
            .unwrap()
            .1
            .values()
            .map(|row| {
                row["ElementData"]["BreakthroughPackId"]
                    .as_str()
                    .unwrap()
                    .to_ascii_lowercase()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            required,
            GROUPS
                .iter()
                .map(|group| group.pack.to_ascii_lowercase())
                .collect()
        );
        assert_eq!(GROUPS.len(), crate::weapon_definitions::definitions().len());
        let rows = transport()["rows"].as_object().unwrap();
        assert_eq!(rows.len(), GROUPS.len() * LEVEL_CAPS.len());
        for group in GROUPS {
            for (stage, cap) in LEVEL_CAPS.iter().enumerate() {
                let row = &rows[&format!("{}_{stage}", group.pack)];
                assert_eq!(row.as_object().unwrap().len(), 2);
                assert_eq!(row["MaxForkLevel"], *cap);
                assert!(row["ModifyPackID"].is_string());
            }
            assert!(!rows.contains_key(&format!("{}_7", group.pack)));
        }
    }

    #[test]
    fn explicit_overlay_preserves_missing_rows_fields_and_source_validation() {
        let mut value = overlay();
        let key = format!("{}_6", GROUPS[0].pack);
        value["rows"].as_object_mut().unwrap().remove(&key);
        let mut tables = BTreeMap::from([(OVERLAY_TABLE.into(), value)]);
        assert!(table(&tables).unwrap().lookup(&key).is_err());
        tables.get_mut(OVERLAY_TABLE).unwrap()["rows"][&key] = json!({"MaxForkLevel":80});
        let bound = table(&tables).unwrap();
        let (_, row) = bound.lookup(&key).unwrap();
        assert!(crate::native::text(row, "ModifyPackID").is_err());
        tables.get_mut(OVERLAY_TABLE).unwrap()["source_id"] = json!(SOURCE_ID);
        assert!(table(&tables).is_err());
        assert!(table(&BTreeMap::new()).unwrap().lookup(&key).is_ok());
    }
}
