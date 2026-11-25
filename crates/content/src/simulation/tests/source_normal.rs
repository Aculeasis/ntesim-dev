use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_next_normal_tracks_two_continuing_chains_for_every_released_owner() {
    for owner in crate::registry::character_descriptors().unwrap() {
        let name = owner.slug;
        let length = if matches!(name, "aurelia" | "akane") {
            4
        } else {
            5
        };
        let result = run(&format!(
            "options duration=60 crit=fixed;
            {name} char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active {name};
            repeat 2 {{
                for let stage=1; stage<={length}; stage=stage+1 {{
                    if .{name}.normal!=stage {{let assertion_failure=1/0;}}
                    {name} attack;
                }}
                if .{name}.normal!=1 {{let assertion_failure=1/0;}}
            }}"
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{name}: {:?}",
            result.diagnostic
        );
        assert_eq!(result.request.actions.len(), length * 2, "{name}");
    }
}

#[test]
fn source_next_normal_reads_exact_gap_expiry_and_actual_packet_selection() {
    let result = run("options duration=8 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active mint;
        if .mint.normal!=1 {let assertion_failure=1/0;}
        mint attack;
        wait_until(2);
        if .mint.normal!=2 {let assertion_failure=1/0;}
        wait(0.000001);
        if .mint.normal!=1 {let assertion_failure=1/0;}
        mint attack;
        if .mint.normal!=2 {let assertion_failure=1/0;}
        mint skill;
        if .mint.normal!=1 {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let normals: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.ability_id == "GA_Mint019_Melee")
        .map(|hit| hit.damage_effect_id.as_str())
        .collect();
    assert_eq!(
        normals,
        [
            "GE_Player_Mint_Melee1_Damage_New",
            "GE_Player_Mint_Melee2_Damage_New",
            "GE_Player_Mint_Melee1_Damage_New",
            "GE_Player_Mint_Melee2_Damage_New"
        ]
    );
}

#[test]
fn source_next_normal_preserves_same_actor_swap_and_resets_on_field_exit() {
    let result = run("options duration=10 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active mint;
        mint attack;
        if .mint.normal!=2 {let assertion_failure=1/0;}
        mint swap;
        if .mint.normal!=2 {let assertion_failure=1/0;}
        zero swap;
        if .mint.normal!=1 {let assertion_failure=1/0;}
        mint attack;
        if .mint.normal!=2 {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
}

#[test]
fn source_next_normal_queries_leave_rng_and_native_replay_unchanged() {
    let prefix = "options duration=10 crit=random;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active mint;";
    let baseline = run(&format!("{prefix} repeat 2 {{mint attack:5;}}"));
    let observed = run(&format!(
        "{prefix}
        repeat 2 {{
            for let i=1; i<=5; i=i+1 {{
                repeat 3 {{if .mint.normal!=i {{let assertion_failure=1/0;}}}}
                mint attack;
            }}
        }}"
    ));
    assert!(observed.report.success && observed.report.execution_completed);
    super::assert_same_combat(&observed.report, &baseline.report);
    let replay =
        crate::simulation::run_game_scenario_report(&observed.request, &Default::default())
            .unwrap();
    assert!(
        serde_json::to_value(&observed.report).unwrap() == serde_json::to_value(replay).unwrap()
    );
}

#[test]
fn source_next_normal_reads_owner_boundaries_and_modes_without_advancing_them() {
    use crate::characters::RuntimeState;
    for name in ["character:1021", "character:1020", "character:1025"] {
        let mut state = RuntimeState::new(name);
        match &mut state {
            RuntimeState::Edgar(s) => s.clock = Some(Default::default()),
            RuntimeState::Haniel(s) => s.clock = Some(Default::default()),
            RuntimeState::Hathor(s) => s.clock = Some(Default::default()),
            _ => unreachable!(),
        }
        let (stage, deadline) = state.normal_chain_fields().unwrap();
        *stage = 3;
        *deadline = Some(1_000_000);
        assert_eq!(state.next_normal(999_999).unwrap(), 1, "{name}");
        assert_eq!(state.next_normal(1_000_000).unwrap(), 4, "{name}");
        assert_eq!(state.next_normal(1_000_001).unwrap(), 1, "{name}");
        let (stage, deadline) = state.normal_chain_fields().unwrap();
        assert_eq!((*stage, *deadline), (3, Some(1_000_000)));
    }
    let mut fadia = RuntimeState::new("character:1039");
    let state = fadia.fadia_mut().unwrap();
    state.full_kit = true;
    state.normal_stage = 1;
    state.lilith_until_us = Some(2_000_000);
    state.lilith_stage = 4;
    assert_eq!(fadia.next_normal(1_999_999).unwrap(), 5);
    assert_eq!(fadia.next_normal(2_000_000).unwrap(), 2);
    assert_eq!(fadia.fadia_mut().unwrap().lilith_stage, 4);

    let mut hotori = RuntimeState::new("character:1052");
    hotori.hotori_mut().unwrap().timepiece.mode =
        Some(crate::characters::hotori::timepiece::Mode {
            entry_us: 1_000_000,
            end_us: 9_000_000,
            records_n: 0,
            root_command_index: 0,
        });
    for (count, expected) in [(0, 1), (4, 5), (5, 1), (9, 5), (10, 6), (11, 1)] {
        hotori.hotori_mut().unwrap().timepiece.katana_count = count;
        assert_eq!(hotori.next_normal(1_000_000).unwrap(), expected);
        assert_eq!(hotori.hotori_mut().unwrap().timepiece.katana_count, count);
    }
    assert_eq!(hotori.next_normal(9_000_000).unwrap(), 1);
}
