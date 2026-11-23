use crate::simulation::GameScenarioReport;

pub(crate) fn validate_completed(report: &GameScenarioReport) -> Result<(), String> {
    if !report.success
        || !report.execution_completed
        || !report.invariants_passed
        || report.halt_diagnostic.is_some()
        || report
            .final_actor_states
            .iter()
            .any(|actor| !actor.energy_conserved)
    {
        return Err(format!(
            "execution failed or incomplete: {:?}",
            report.halt_diagnostic
        ));
    }
    Ok(())
}
