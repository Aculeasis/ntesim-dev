use super::*;
use serde_json::{Value, json};

// Market birth A3.420264 precedes cinematic release A4.250937.
pub(super) const MARKET_PAUSE_US: u64 = 4_250_937 - 3_420_264;

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chiz_held::request(targets);
    req.relative_duration_us = 20_000_000;
    req.actors[0].initial_energy = 100;
    for model in [
        crate::HypothesisId::ChizMarketV1,
        crate::HypothesisId::ChizRecordedMarketTraceV1,
    ] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    req
}
pub(super) fn burst(time: u64) -> GameActionCommand {
    let mut action = super::chiz_pressed::press(time);
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        energy_cost,
        cooldown_us,
        ..
    } = &mut action
    {
        *ability_id = "GA_Chiichan073_UltraSkill".into();
        *damage_effect_id = "GE_Player_Chiichan073_UltraSkill1_Damage".into();
        *energy_cost = 100;
        *cooldown_us = 15_000_000;
    }
    action
}
fn run(req: &GameScenarioRequest) -> Value {
    let report = run_game_scenario(req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    report
}
fn current(report: &Value, resource: &str) -> Value {
    report["final_actor_states"][0]["fractional_resources"][resource]["current"].clone()
}

#[test]
fn chiz_market_input_prices_settle_once_cap_proceeds_and_repay_with_recipient_shared_first_crit() {
    for targets in [1, 3] {
        let mut req = request(targets);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
        req.actions = vec![
            burst(0),
            super::chiz_pressed::press(4_500_000 + MARKET_PAUSE_US),
            super::chiz_pressed::press(6_000_000 + MARKET_PAUSE_US),
            super::chiz_pressed::press(7_200_000 + MARKET_PAUSE_US),
        ];
        let report = run(&req);
        // Entry: savings15000+unused loans15000. Input prices indexes2/5/7:
        // +.38 caps30000, then -.29 ->21300, +.06 ->22578. Repay15000.
        assert_eq!(current(&report, "chiz.grain"), json!("7578/1"));
        assert_eq!(current(&report, "chiz.loan_debt"), json!("0/1"));
        assert_eq!(current(&report, "chiz.market"), json!("0/1"));
        assert_eq!(report["hits_settled"], 17 * targets);
        let hits: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill1_CoinDamage")
            .collect();
        for (index, group) in hits.chunks(targets).enumerate() {
            assert_eq!(
                support::hit_metadata(&report, group[0])["packet"]["fixed_crit_rate"],
                json!(format!("{}/{}", if index == 0 { 1 } else { 0 }, 1))
            );
            for hit in group {
                assert_eq!(hit["damage"], group[0]["damage"]);
            }
        }
        let settlements: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "chiz_market_settlement")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            settlements,
            [
                4_700_000 + MARKET_PAUSE_US,
                6_200_000 + MARKET_PAUSE_US,
                7_400_000 + MARKET_PAUSE_US
            ]
        );
        let pools = &report["final_actor_states"][0]["fractional_resources"];
        assert_eq!(pools["chiz.market"]["overflow"], json!("11400/1"));
        for key in ["chiz.market", "chiz.grain", "chiz.loan_debt"] {
            assert_eq!(pools[key]["conserved"], true);
        }
    }
}

#[test]
fn chiz_market_chart_never_compounds_and_expiry_precedes_seventh_interest() {
    for interest in [false, true] {
        let mut req = request(1);
        req.actors[0].awakening =
            ntesim_data::AwakeningSelection::selected(if interest { vec![2] } else { vec![] })
                .unwrap();
        req.actions = vec![burst(0)];
        req.relative_duration_us = 7_000_000;
        let report = run(&req);
        //15000*(1.1)^6=26573.415, repayment leaves11573.415.
        assert_eq!(
            current(&report, "chiz.grain"),
            if interest {
                json!("2314683/200")
            } else {
                json!("0/1")
            }
        );
        let ticks = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "chiz_surplus_interest")
            .count();
        assert_eq!(ticks, if interest { 6 } else { 0 });
        assert_eq!(report["hits_settled"], 6);
    }
}
