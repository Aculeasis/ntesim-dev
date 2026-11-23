use super::*;

#[test]
fn qte_generated_word_captures_burst_talent_and_retains_native_ability_identity() {
    let mut req = phase2_request();
    req.actors[0].talent_levels = Some([1, 2, 8]);
    req.actors[0].purchased_skill_level = 1;
    // Three selected effects grant R3 to Burst, while QTE remains its own level1.
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![1, 3, 5]).unwrap();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.actions = vec![
        burst(0),
        make_action(
            3_000_000,
            0,
            "GA_Cang_QTE",
            "GE_Player_Cang_QTE_Damage",
            0,
            0,
        ),
    ];
    // QTE is outside the three-slot talent tuple; its admitted coordinate0
    // explicitly supplies the frozen level1 coefficient (2.0 ATK).
    if let GameActionCommand::InvokeAbility { atk_rate, .. } = &mut req.actions[1] {
        *atk_rate = Ratio {
            numerator: 2,
            denominator: 1,
        };
    }
    let report = run(&req);
    let hits = report["hit_receipts"].as_array().unwrap();
    let qte = hits
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Cang_QTE_Damage")
        .unwrap();
    let word = hits
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Cang_WorldA_Damage")
        .unwrap();
    assert_eq!(qte["scaling_rate"], json!("2/1"));
    assert_eq!(word["scaling_rate"], json!("5553/1000"));
    assert_eq!(word["time_us"], 5_400_358);
    let word_meta = support::hit_metadata(&report, word);
    assert_eq!(word_meta["action"]["native_ability_index"], 3);
    assert_eq!(word_meta["action"]["ability_id"], "GA_Cang_UltraSkill");
    assert_eq!(word_meta["binding"]["kind"], "owner_child");
    let burst = hits
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Cang_UltraSkill2_Damage")
        .unwrap();
    assert_eq!(
        word_meta["action"],
        support::hit_metadata(&report, burst)["action"]
    );
}
