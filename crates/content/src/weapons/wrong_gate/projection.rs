use crate::build::{
    compile_weapon_values, finish_weapon_projection, ratio, receipt, weapon_seconds_to_us,
};
use crate::formula::Ratio;
use crate::simulation::{GameActorConfig, GearProcConfig};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn project(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &mut GameActorConfig,
    positions: &[usize],
) -> Result<Value, String> {
    let item = ntesim_data::weapon_definitions::lookup(tables, "fork_Door")?
        .1
        .as_object()
        .ok_or("The Wrong Gate native item row missing")?;
    let character = ntesim_data::character::definition(&actor.character_id)?;
    if item["ElementData"]["ApplyGroupType"].as_str() != Some(character.weapon_group.native_key()) {
        return Err("The Wrong Gate group is incompatible with the wearer".into());
    }
    let native = compile_weapon_values(tables, schema_version, content_id, actor)?;
    let parameters = native["parameters"]
        .as_array()
        .ok_or("The Wrong Gate native parameters missing")?;
    let value = |name: &str, percent: bool| -> Result<Ratio, String> {
        let parameter = parameters
            .iter()
            .find(|parameter| parameter["name_id"].as_str() == Some(name))
            .ok_or_else(|| format!("The Wrong Gate parameter {name} missing"))?;
        if parameter["b_is_percent"] != percent {
            return Err(format!("The Wrong Gate has unsupported unit for {name}"));
        }
        ratio(receipt(&parameter["selected"]["value"])?)
    };
    if parameters.len() != 4 {
        return Err("unsupported additional The Wrong Gate parameter dependencies".into());
    }
    let duration = parameters
        .iter()
        .find(|parameter| parameter["name_id"] == "buff_Door_CD")
        .ok_or("The Wrong Gate duration parameter missing")?;
    if duration["b_is_percent"] != false {
        return Err("The Wrong Gate duration must be seconds".into());
    }
    let seconds = receipt(&duration["selected"]["value"])?;
    let duration_us = weapon_seconds_to_us(seconds, "The Wrong Gate duration")?;
    let proc = GearProcConfig::WrongGate {
        atk_bonus: value("buff_Door_Atk", true)?,
        ally_damage_bonus: value("buff_Door_OtherUp", true)?,
        anima_damage_bonus: value("buff_Door_Up", true)?,
        duration_us,
    };
    finish_weapon_projection(
        actor,
        positions,
        proc,
        native,
        json!({
            "runtime_execution_validated": false,
            "scope": crate::weapons::wrong_gate::HYPOTHESIS_ID.description()
        }),
    )
}

#[cfg(test)]
mod tests {
    use crate::build::{GearProjection, GearProjectionPolicy, Request, project_gear};
    use crate::formula::Ratio;
    use crate::simulation::{GameScenarioRequest, GearProcConfig};
    use serde_json::Value;
    use std::collections::BTreeMap;

    #[test]
    fn wrong_gate_binds_five_native_curves_and_rejects_changed_units() {
        let tables: BTreeMap<String, Value> = BTreeMap::from([
            (
                "DT_ForkUpgradeStarDataTable".into(),
                ntesim_data::weapon_refinements::overlay(),
            ),
            ("CT_ForkBuff".into(), ntesim_data::weapon_curves::overlay()),
        ]);
        let request: Request = serde_json::from_slice(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/native-gear-built-game-research.json"
        )))
        .unwrap();
        let mut scenario: GameScenarioRequest = request.scenario;
        scenario.actors[0].character_id = "character:1021".into();
        scenario.actors[0].weapon_id = "weapon:fork_Door".into();
        scenario.actors[0].gear_procs.clear();
        let projection = GearProjection {
            policy: GearProjectionPolicy::NativeWeaponCurveV1,
            hypothesis: crate::weapons::wrong_gate::HYPOTHESIS_ID
                .description()
                .into(),
        };
        let expected = [
            ((4, 25), (3, 20), (3, 10)),
            ((1, 5), (3, 16), (3, 8)),
            ((6, 25), (9, 40), (9, 20)),
            ((7, 25), (21, 80), (21, 40)),
            ((8, 25), (3, 10), (3, 5)),
        ];
        for (tier, (atk, other, anima)) in (1..=5).zip(expected) {
            scenario.actors[0].weapon_tier = tier;
            scenario.actors[0].gear_procs.clear();
            project_gear(&tables, &mut scenario, &projection).unwrap();
            assert_eq!(
                serde_json::to_value(&scenario.actors[0].gear_procs[0]).unwrap(),
                serde_json::to_value(GearProcConfig::WrongGate {
                    atk_bonus: Ratio {
                        numerator: atk.0,
                        denominator: atk.1,
                    },
                    ally_damage_bonus: Ratio {
                        numerator: other.0,
                        denominator: other.1,
                    },
                    anima_damage_bonus: Ratio {
                        numerator: anima.0,
                        denominator: anima.1,
                    },
                    duration_us: 20_000_000,
                })
                .unwrap()
            );
        }

        let mut wrong_group = scenario.clone();
        wrong_group.actors[0].character_id = "character:1020".into();
        wrong_group.actors[0].gear_procs.clear();
        assert!(
            project_gear(&tables, &mut wrong_group, &projection)
                .unwrap_err()
                .contains("group is incompatible")
        );

        let mut wrong_unit_tables = tables;
        let wrong_unit = wrong_unit_tables
            .get_mut("DT_ForkUpgradeStarDataTable")
            .unwrap()
            .pointer_mut("/rows/upgradestar_pack_fork_Door_1/DataList/0/bIsPercent")
            .unwrap();
        *wrong_unit = Value::Bool(false);
        scenario.actors[0].weapon_tier = 1;
        scenario.actors[0].gear_procs.clear();
        assert!(
            project_gear(&wrong_unit_tables, &mut scenario, &projection)
                .unwrap_err()
                .contains("unit")
        );
    }
}
