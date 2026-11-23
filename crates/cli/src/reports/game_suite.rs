use crate::report_context::constant_report;
use ntesim_content::simulation::{GameScenarioRequest, run_game_scenario};
use ntesim_core::execution::ExecutionLimits;
use serde_json::{Value, json};

pub(crate) fn game_suite_report(input: &[u8]) -> Result<Value, String> {
    if input.len() > 16 * 1024 * 1024 {
        return Err("game suite input exceeds 16 MiB".into());
    }
    let requests: Vec<GameScenarioRequest> =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    if requests.is_empty() || requests.len() > 512 {
        return Err("game suite requires 1..=512 scenarios".into());
    }
    constant_report(input, &ntesim_data::team::TABLES, |_tables, _coverage| {
        let limits = ExecutionLimits::default();
        let reports = requests
            .iter()
            .enumerate()
            .map(|(index, request)| {
                run_game_scenario(request, &limits)
                    .map_err(|error| format!("scenario {index}: {error}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let complete = reports.iter().all(|report| {
            report["success"] == true
                && report["execution_completed"] == true
                && report["invariants_passed"] == true
        });
        Ok(json!({ "reports": reports, "execution_completed": complete }))
    })
}
