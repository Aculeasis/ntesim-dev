use super::*;
#[test]
fn native_aoe_energy_is_once_per_packet_from_first_positive_recipient() {
    for (first_hp, immune) in [(0, false), (1, false), (100_000_000, true)] {
        let mut req = request(1, 1, 35, "zankou skill;");
        req.targets[0].hp_current = first_hp;
        if immune {
            req.targets[0].resistance = Ratio::one();
        }
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["final_actor_states"][0]["granted_energy"], 3);
        assert_eq!(report["final_actor_states"][0]["charge_remainder"], "4/5");
        let leads = all_packets(&report, "GE_Player_Zankou_Skill2_Damage");
        assert_eq!(
            leads.len(),
            if immune {
                8
            } else if first_hp == 0 {
                4
            } else {
                5
            }
        );
        let groups = leads
            .iter()
            .map(|h| h["energy_group_id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(groups.len(), 4);
        for group in groups {
            assert_eq!(
                leads
                    .iter()
                    .filter(|h| h["energy_group_id"] == group && h["target_index"] == 1)
                    .count(),
                1
            );
        }
        assert!(all_packets(&report, "GE_Player_Zankou_DotDamage").is_empty());
        let grants = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "zankou_skill_direct_packet_energy_grant")
            .collect::<Vec<_>>();
        assert_eq!(grants.len(), 5);
        assert!(
            grants
                .iter()
                .all(|e| e["energy_fanout"]["recipients"].as_array().unwrap().len() == 2)
        );
    }
}
