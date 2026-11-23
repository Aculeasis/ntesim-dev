use super::*;
use crate::native::Scalar;

fn fixture() -> (BTreeMap<String, Value>, Request) {
    let tables = BTreeMap::from([
        (
            "DT_ForkUpgradeStarDataTable".into(),
            crate::weapon_refinements::overlay(),
        ),
        ("CT_ForkBuff".into(), crate::weapon_curves::overlay()),
    ]);
    (
        tables,
        Request {
            schema_version: 1,
            content_id: "ntesim-current".into(),
            weapon_id: "weapon:fork_lingganzhongjiezhe".into(),
            refinement: 1,
        },
    )
}

#[test]
fn mind_royale_parameters_keep_native_t_values_and_constant_key_receipts() {
    let (tables, mut request) = fixture();
    for tier in 1..=5 {
        request.refinement = tier;
        let report = compile(&tables, &request).unwrap();
        assert_eq!(report["parameter_count"], 2);
        assert_eq!(
            report["parameters"][0]["selected"]["value"],
            Scalar::new(u128::from(8 + 2 * tier), 1).receipt()
        );
        assert_eq!(
            report["parameters"][1]["selected"]["value"],
            Scalar::new(20, 1).receipt()
        );
        assert_eq!(
            report["parameters"][1]["selected"]["selection"],
            if tier == 1 || tier == 5 {
                "exact_serialized_key"
            } else {
                "serialized_constant_left_key"
            }
        );
        assert_eq!(report["passive_adapter_complete"], false);
        assert_eq!(compile(&tables, &request).unwrap(), report);
    }
}

#[test]
fn all_frozen_weapon_refinements_resolve_listed_parameters_without_release_claims() {
    let (tables, mut request) = fixture();
    let mut count = 0;
    let mut parameters = 0;
    let mut interior = 0;
    for weapon in crate::weapon_definitions::rows(&tables).unwrap().1.keys() {
        request.weapon_id = format!("weapon:{weapon}");
        for tier in 1..=5 {
            request.refinement = tier;
            let report = compile(&tables, &request).unwrap();
            assert_eq!(report["release_status"], "unverified");
            count += 1;
            parameters += report["parameter_count"].as_u64().unwrap();
            interior += report["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|p| p["selected"]["selection"] == "serialized_constant_left_key")
                .count();
        }
    }
    assert_eq!((count, parameters, interior), (255, 770, 159));
}

#[test]
fn unsupported_interpolation_extrapolation_and_malformed_parameters_fail_closed() {
    let (tables, mut request) = fixture();
    request.refinement = 2;
    for (pointer, value) in [
        (
            "/rows/buff_xiangdao_CD/InterpMode",
            json!("ERichCurveInterpMode::RCIM_Linear"),
        ),
        ("/rows/buff_xiangdao_CD/Keys/0/Time", json!(3)),
        ("/rows/buff_xiangdao_CD/Keys/1/Time", json!(1)),
        ("/rows/buff_xiangdao_CD/Keys/0/Time", json!(1.5)),
        ("/rows/buff_xiangdao_CD/Keys/1/Value", json!("20")),
        ("/rows/buff_xiangdao_CD/Keys", json!([])),
    ] {
        let mut modified = tables.clone();
        *modified
            .get_mut("CT_ForkBuff")
            .unwrap()
            .pointer_mut(pointer)
            .unwrap() = value;
        assert!(compile(&modified, &request).is_err(), "{pointer}");
    }
    let mut modified = tables.clone();
    modified.get_mut("CT_ForkBuff").unwrap()["rows"]
        .as_object_mut()
        .unwrap()
        .remove("buff_xiangdao_energy");
    assert!(compile(&modified, &request).is_err());
    let mut modified = tables;
    modified.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_lingganzhongjiezhe_2"]
        ["DataList"][1]["NameID"] = json!("BUFF_XIANGDAO_ENERGY");
    assert!(compile(&modified, &request).is_err());
}
