use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn run(source: &str) -> SourceScenarioReport {
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    result
}

#[test]
fn source_remaining_plunges_preserve_owner_airtime_contacts_and_aoe_across_cycles() {
    for (owner, delay) in [
        ("blackbird", 653_784),
        ("chiz", 799_456),
        ("daffodil", 653_784),
        ("edgar", 649_876),
        ("fadia", 632_851),
        ("haniel", 665_670),
        ("hathor", 716_417),
    ] {
        let source = format!(
            "options duration=20 crit=random stamina_capacity=100 initial_stamina=100
            stamina_use_efficiency=1 stamina_recovery_rate=0;
            mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
            {owner} char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
            target hp=100000000 def=0 resist=0;
            target hp=100000000 def=0 resist=0; active mint;
            repeat 2 {{{owner} plunge[target=t1,airtime=600ms,height_bonus=1/2]; wait(2);}}"
        );
        let result = run(&source);
        assert_eq!(result.request.plunge_inputs.len(), 2, "{owner}");
        assert_eq!(result.request.aim_targets.len(), 2, "{owner}");
        for (&command, profile) in &result.request.plunge_inputs {
            assert_eq!(result.request.aim_targets[&command], 1);
            assert_eq!(profile.landing_delay_us, 600_000);
            let hits: Vec<_> = result
                .report
                .hit_receipts
                .iter()
                .filter(|hit| hit.command_index == Some(command))
                .collect();
            assert_eq!(hits.len(), 2, "{owner}");
            assert_eq!(
                {
                    let mut targets = hits.iter().map(|hit| hit.target_index).collect::<Vec<_>>();
                    targets.sort_unstable();
                    targets
                },
                vec![0, 1],
                "{owner}"
            );
            for hit in hits {
                assert_eq!(
                    hit.time_us,
                    result.request.actions[command].timestamp_us() + delay,
                    "{owner}"
                );
            }
        }
        super::assert_native_replay(&result);
        let scheduled = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        assert_eq!(
            scheduled.plunge_inputs, result.request.plunge_inputs,
            "{owner}"
        );
        assert_eq!(scheduled.aim_targets, result.request.aim_targets, "{owner}");
    }
}
