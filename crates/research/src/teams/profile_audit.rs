use crate::{character, damage_values, published_teams, simulation::ExactFraction, weapon};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const TABLES: [&str; 0] = [];

fn fraction(value: &Value) -> Result<ExactFraction, String> {
    let number = |key| {
        value[key]
            .as_str()
            .ok_or("native exact receipt missing")?
            .parse::<u128>()
            .map_err(|_| "native exact receipt invalid")
    };
    ExactFraction::new(number("numerator")?, number("denominator")?)
}

pub fn audit(
    tables: &BTreeMap<String, Value>,
    coverage: &Value,
    csv: &str,
) -> Result<Value, String> {
    let variants = published_teams::parse_published_variants_csv(csv)?;
    let slugs = variants
        .iter()
        .flat_map(|v| v.slots.iter())
        .collect::<BTreeSet<_>>();
    let entities = coverage["entities"]
        .as_array()
        .ok_or("coverage entities missing")?;
    let mut rows = Vec::new();
    let mut base_mismatches = 0;
    let mut damage_failures = 0;
    let mut rate_mismatches = 0;
    let mut empty_atk_arrays = 0;
    for slug in slugs {
        let profile = published_teams::get_character_profile(slug)?;
        let released = entities
            .iter()
            .enumerate()
            .filter(|(_, entity)| {
                entity["kind"] == "character"
                    && entity["slug"] == *slug
                    && entity["availability"] == "released"
            })
            .collect::<Vec<_>>();
        if released.len() != 1 {
            return Err(format!(
                "profile slug must identify one released character: {slug}"
            ));
        }
        let (coverage_index, entity) = released[0];
        let character_id = entity["id"]
            .as_str()
            .ok_or("released character ID missing")?;
        let character = character::compile(&character::Request {
            schema_version: 1,
            content_id: "ntesim-current".into(),
            character_id: character_id.into(),
            level: 80,
            breakthrough: 6,
        })?;
        let weapon = weapon::compile(
            tables,
            &weapon::Request {
                schema_version: 1,
                content_id: "ntesim-current".into(),
                weapon_id: profile.weapon_id.into(),
                level: 80,
                breakthrough: 6,
                refinement: profile.weapon_tier,
            },
        )?;
        let native_base = fraction(&character["attributes"]["AtkBase"])?
            .add(fraction(&weapon["attributes"]["AtkBase"])?)?;
        let manual_base = ExactFraction::new(u128::from(profile.base_atk), 1)?;
        let base_matches = native_base == manual_base;
        base_mismatches += usize::from(!base_matches);
        let damage_request = damage_values::Request {
            schema_version: 1,
            content_id: "ntesim-current".into(),
            character_id: character_id.into(),
            ability_id: profile.ability_id.into(),
            effect_ids: vec![profile.damage_effect_id.into()],
            array_index: 7,
        };
        let (damage, damage_diagnostic, rate_comparison) = match damage_values::compile(
            tables,
            &damage_request,
        ) {
            Ok(report) => {
                let selected = &report["effects"][0]["arrays"]["AtkRateBaseArray"];
                let comparison = if selected["state"] == "selected_native_coordinate" {
                    let native_rate = fraction(&selected["selected_value"])?;
                    let manual_rate = ExactFraction::from_ratio(profile.external_atk_rate_claim)?;
                    let matches = native_rate == manual_rate;
                    rate_mismatches += usize::from(!matches);
                    json!({"matches":matches,"manual":manual_rate,"native_coordinate_value":native_rate})
                } else {
                    empty_atk_arrays += 1;
                    Value::Null
                };
                (report, Value::Null, comparison)
            }
            Err(error) => {
                damage_failures += 1;
                (Value::Null, json!(error), Value::Null)
            }
        };
        rows.push(json!({
            "slug":slug, "quality":"incomplete", "game_dps_computed":false,
            "character_id_matches_released_inventory":profile.character_id == character_id,
            "release_evidence":{"source_id":"legacy:characters","file":"coverage.json",
                "pointer":format!("/entities/{coverage_index}")},
            "manual_profile":{"character_id":profile.character_id,"weapon_id":profile.weapon_id,
                "weapon_tier":profile.weapon_tier,"awakening":profile.awakening,
                "base_atk":profile.base_atk,"atk":profile.atk_ratio,"hp":profile.hp,"def":profile.def,
                "ability_id":profile.ability_id,"damage_effect_id":profile.damage_effect_id,
                "external_atk_rate_claim":profile.external_atk_rate_claim},
            "base_atk_comparison":{"matches":base_matches,"manual":manual_base,"native_character_plus_weapon":native_base},
            "bare_character_hp_comparison":{"matches":fraction(&character["attributes"]["HPMaxBase"])? == ExactFraction::new(u128::from(profile.hp),1)?,
                "manual":profile.hp,"native":character["attributes"]["HPMaxBase"]},
            "bare_character_def_comparison":{"matches":fraction(&character["attributes"]["DefBase"])? == ExactFraction::new(u128::from(profile.def),1)?,
                "manual":profile.def,"native":character["attributes"]["DefBase"]},
            "character_report":character,"weapon_report":weapon,
            "damage_request":damage_request,"damage_report":damage,"damage_diagnostic":damage_diagnostic,
            "atk_rate_comparison":rate_comparison,
            "full_build_stats_validated":false,"total_atk_comparison":null,
        }));
    }
    Ok(json!({
        "kind":"profile-source-audit-v1","schema_version":1,"content_id":"ntesim-current",
        "status":"INCOMPLETE","quality":"incomplete","game_dps_computed":false,"acceptance_ready":false,
        "variants_selected":variants.len(),"profiles_checked":rows.len(),
        "base_atk_mismatches":base_mismatches,"damage_binding_failures":damage_failures,
        "atk_rate_mismatches":rate_mismatches,"profiles":rows,
        "bound_empty_atk_arrays":empty_atk_arrays,
        "scope":"Native serialized identity/constants/coordinate comparisons only; no legal full builds or gameplay dispatch",
        "blockers":["Complete recommended console/substat/set builds and passive execution remain unavailable",
            "Coordinate 7 is explicit for this source audit; purchased level mapping and hit reachability remain unbound",
            "Native property application, weapon/set availability and full gameplay compatibility remain unresolved"],
    }))
}
