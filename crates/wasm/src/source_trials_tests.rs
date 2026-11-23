use super::source_trials::dispatch;
use serde_json::{Value, json};

fn call(action: &str, payload: Value) -> Result<Value, String> {
    dispatch(action, &serde_json::to_vec(&payload).unwrap())
}

fn compiled(rotation: &str) -> Value {
    let source = format!(
        "options duration=10 iteration=3 crit=random; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0; target hp=100000000 def=0 resist=0; active mint; {rotation}"
    );
    let response = ntesim_content::dsl::source::compile_response(
        &ntesim_content::dsl::compiler::SourceRequest {
            source,
            limits: Default::default(),
        },
    );
    serde_json::to_value(response.compiled.unwrap()).unwrap()
}

#[test]
fn chunks_preserve_native_live_reports_seeds_statistics_and_admission_failures() {
    let artifact = compiled("while .time < 3 { mint attack; wait(1); }");
    call("start_source_trials", artifact.clone()).unwrap();
    let first = call(
        "step_source_trials",
        json!({"offset":0,"count":1,"elapsed_ms":0}),
    )
    .unwrap();
    assert_eq!(first["samples_delta"].as_array().unwrap().len(), 1);
    let execution: ntesim_content::simulation::SourceScenarioReport =
        serde_json::from_value(first["first_execution"].clone()).unwrap();
    let native = ntesim_content::simulation::run_game_scenario_report(
        &execution.request,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(native).unwrap(),
        serde_json::to_value(execution.report).unwrap()
    );
    assert!(
        call(
            "step_source_trials",
            json!({"offset":0,"count":1,"elapsed_ms":0})
        )
        .is_err()
    );
    let tail = call(
        "step_source_trials",
        json!({"offset":1,"count":2,"elapsed_ms":0}),
    )
    .unwrap();
    let sample = call("run_source_sample", json!({"compiled":artifact,"seed":43})).unwrap();
    assert_eq!(sample["report"]["seed"], 43);
    assert_eq!(sample["source"], artifact["input"]["source"]);
    call("start_source_trials", artifact.clone()).unwrap();
    let whole = call(
        "step_source_trials",
        json!({"offset":0,"count":3,"elapsed_ms":0}),
    )
    .unwrap();
    assert_eq!(tail["aggregate"], whole["aggregate"]);
    let mut samples = first["samples_delta"].as_array().unwrap().clone();
    samples.extend(tail["samples_delta"].as_array().unwrap().iter().cloned());
    assert_eq!(samples, *whole["samples_delta"].as_array().unwrap());
    let restored = call(
        "resume_source_trials",
        json!({"compiled":artifact,"samples":first["samples_delta"],"elapsed_ms":0}),
    )
    .unwrap();
    assert_eq!(restored, first);
    let resumed = call(
        "step_source_trials",
        json!({"offset":1,"count":1,"elapsed_ms":0}),
    )
    .unwrap();
    assert_eq!(resumed["samples_delta"][0], whole["samples_delta"][1]);
    let finished = call(
        "step_source_trials",
        json!({"offset":2,"count":1,"elapsed_ms":0}),
    )
    .unwrap();
    assert_eq!(finished["aggregate"], whole["aggregate"]);
    let mut tampered = first["samples_delta"].clone();
    tampered[0]["events"] = json!(999);
    let mut tampered_shp_superseded = first["samples_delta"].clone();
    tampered_shp_superseded[0]["shp"] = json!(12.34);
    let mut tampered_shp_value = first["samples_delta"].clone();
    tampered_shp_value[0]["shp"]["relative"] = json!(9999.0);
    let mut tampered_dps_value = first["samples_delta"].clone();
    tampered_dps_value[0]["dps"]["absolute"] = json!(9999.0);
    let mut stale = artifact.clone();
    stale["engine_source_sha256"] = json!("stale");
    for payload in [
        json!({"compiled":artifact,"samples":tampered,"elapsed_ms":0}),
        json!({"compiled":artifact,"samples":tampered_shp_superseded,"elapsed_ms":0}),
        json!({"compiled":artifact,"samples":tampered_shp_value,"elapsed_ms":0}),
        json!({"compiled":artifact,"samples":tampered_dps_value,"elapsed_ms":0}),
        json!({"compiled":stale,"samples":first["samples_delta"],"elapsed_ms":0}),
        json!({"compiled":artifact,"samples":[],"elapsed_ms":0}),
        json!({"compiled":artifact,"samples":whole["samples_delta"],"elapsed_ms":0}),
        json!({"compiled":artifact,"samples":first["samples_delta"],"elapsed_ms":-1}),
    ] {
        assert!(call("resume_source_trials", payload).is_err());
        assert!(
            call(
                "step_source_trials",
                json!({"offset":0,"count":1,"elapsed_ms":0})
            )
            .is_err()
        );
    }
    call("start_source_trials", compiled("mint burst;")).unwrap();
    let failed = call(
        "step_source_trials",
        json!({"offset":0,"count":1,"elapsed_ms":0}),
    )
    .unwrap();
    assert!(failed["aggregate"].is_null());
    assert_eq!(failed["failure"]["diagnostic"]["code"], "admission");
    assert!(failed["samples_delta"].as_array().unwrap().is_empty());
    assert!(
        call(
            "step_source_trials",
            json!({"offset":0,"count":1,"elapsed_ms":0})
        )
        .is_err()
    );
    assert!(call("start_source_trials", json!({})).is_err());
    assert!(
        call(
            "step_source_trials",
            json!({"offset":0,"count":1,"elapsed_ms":0})
        )
        .is_err()
    );

    // On the real compile/start path, a warm prepared program must not make
    // the original source text a substitute for validating the artifact.
    let response = super::source_api::dispatch(
        "compile_source",
        &serde_json::to_vec(&artifact["input"]).unwrap(),
    )
    .unwrap();
    assert_eq!(response["compiled"], artifact);
    call("start_source_trials", artifact.clone()).unwrap();
    let mut tampered_request = artifact.clone();
    tampered_request["request"]["seed"] = json!(91);
    assert!(
        call("start_source_trials", tampered_request)
            .unwrap_err()
            .contains("authored input")
    );
    let mut invalid_limits = artifact.clone();
    invalid_limits["input"]["limits"]["maxSteps"] = json!(0);
    assert!(
        call("start_source_trials", invalid_limits)
            .unwrap_err()
            .contains("compilation limits")
    );
    assert!(call("snapshot_source_trials", json!(0)).is_err());

    let mut bounded_artifact = artifact.clone();
    bounded_artifact["input"]["limits"]["maxSteps"] = json!(1);
    call("start_source_trials", bounded_artifact).unwrap();
    let bounded = call(
        "step_source_trials",
        json!({"offset":0,"count":1,"elapsed_ms":0}),
    )
    .unwrap();
    assert!(
        bounded["failure"]["diagnostic"]["message"]
            .as_str()
            .unwrap()
            .contains("limit 1")
    );
    assert!(bounded["samples_delta"].as_array().unwrap().is_empty());
}

#[test]
fn source_report_transport_preserves_native_exports_replay_and_rejected_comparisons() {
    let artifact = compiled("mint attack; wait(1);");
    let invoke = |action: &str, payload| {
        super::source_reports::dispatch(action, &serde_json::to_vec(&payload).unwrap())
    };
    let exported = invoke(
        "export_source_report",
        json!({"compiled":artifact,"seed":42}),
    )
    .unwrap();
    let native: ntesim_content::simulation::export::SourceReportExport =
        serde_json::from_str(exported["report_json"].as_str().unwrap()).unwrap();
    native.replay().unwrap();
    assert_eq!(
        exported["csv_tables"],
        serde_json::to_value(native.csv_tables().unwrap()).unwrap()
    );
    let replayed = invoke(
        "replay_source_report",
        json!({"report_json":exported["report_json"]}),
    )
    .unwrap();
    assert_eq!(replayed["reconciled"], true);
    let mut altered = artifact.clone();
    altered["engine_source_sha256"] = json!("changed");
    assert!(
        invoke(
            "export_source_report",
            json!({"compiled":altered,"seed":42})
        )
        .is_err()
    );
    let baseline = artifact["input"]["source"].as_str().unwrap();
    let request = json!({"schema_version":1,"seeds":[42],"max_events_per_run":1000,
        "max_total_events":4000,"max_receipt_bytes":1048576});
    for (candidate, complete) in [
        (baseline.to_string(), true),
        (baseline.replace("mint attack;", "mint burst;"), false),
    ] {
        let result = invoke(
            "compare_source_reports",
            json!({"baseline":baseline,"candidate":candidate,"request":request}),
        )
        .unwrap();
        assert_eq!(result["comparison"]["complete"], complete);
        assert_eq!(result["comparison"]["mean_dps_delta"].is_null(), !complete);
    }
    let search_request = json!({"schema_version":1,"experiment":request,
        "objective":{"metric":"mean_dps","direction":"maximize"},
        "candidates":[{"id":"baseline","source":baseline},
            {"id":"failed","source":baseline.replace("mint attack;", "mint burst;")}]});
    let prefix = invoke(
        "search_source_reports",
        json!({"request":search_request,"max_new_trials":1}),
    )
    .unwrap();
    assert_eq!(prefix["search"]["complete"], false);
    let resumed = invoke(
        "search_source_reports",
        json!({"request":search_request,
        "checkpoint_json":prefix["checkpoint_json"],"max_new_trials":1}),
    )
    .unwrap();
    let native = ntesim_research::trials::source::search::run(
        serde_json::from_value(search_request.clone()).unwrap(),
        None,
        2,
    )
    .unwrap();
    assert_eq!(resumed["search"], serde_json::to_value(native).unwrap());
    assert_eq!(resumed["search"]["complete"], true);
    assert_eq!(resumed["search"]["success"], false);
    assert_eq!(resumed["search"]["ranking"].as_array().unwrap().len(), 1);
    assert_eq!(resumed["search"]["ranking"][0]["candidate_id"], "baseline");
    assert_eq!(
        serde_json::from_str::<Value>(resumed["report_json"].as_str().unwrap()).unwrap(),
        resumed["search"]
    );
    let mut incompatible = prefix["search"]["checkpoint"].clone();
    incompatible["engine_source_sha256"] = json!("stale");
    assert!(
        invoke(
            "search_source_reports",
            json!({"request":search_request,
        "checkpoint_json":serde_json::to_string(&incompatible).unwrap(),"max_new_trials":1})
        )
        .is_err()
    );
}
