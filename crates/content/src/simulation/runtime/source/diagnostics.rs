//! Preserve structured causes and dynamic occurrence identity without parsing messages.
use super::*;
use crate::simulation::SourceFailureContext;

impl Trial<'_> {
    pub(super) fn retain_source_halt(&mut self) {
        if self.execution.rejected_action.is_some() {
            return;
        }
        let Some(diagnostic) = self.diagnostic.clone() else {
            return;
        };
        let halt = crate::simulation::GameSourceHalt {
            time_us: self.execution.state.current_time_us,
            kind: if let Some(crate::simulation::GameExecutionLimit::EventCount { limit }) =
                self.execution.limit_reached
            {
                crate::simulation::GameSourceHaltKind::EventLimit { limit }
            } else if let Some(crate::simulation::GameExecutionLimit::AbsoluteTime { limit_us }) =
                self.execution.limit_reached
            {
                crate::simulation::GameSourceHaltKind::AbsoluteLimit { limit_us }
            } else if self.execution.success {
                crate::simulation::GameSourceHaltKind::Pending
            } else {
                crate::simulation::GameSourceHaltKind::Failed
            },
            diagnostic,
            context: self.failure_context().expect("source diagnostic"),
        };
        if self.execution.limit_reached.is_none() {
            self.execution.apply_source_halt(&halt);
        }
        self.request.source_halt = Some(halt);
    }

    pub(super) fn ensure_failure_diagnostic(&mut self) {
        if self.diagnostic.is_some()
            || (self.execution.success && self.execution.execution_completed)
        {
            return;
        }
        let code = if self.execution.rejected_action.is_some() {
            DiagnosticCode::Admission
        } else if matches!(
            self.execution.limit_reached,
            Some(crate::simulation::GameExecutionLimit::EventCount { .. })
        ) {
            DiagnosticCode::EventLimit
        } else if matches!(
            self.execution.limit_reached,
            Some(crate::simulation::GameExecutionLimit::AbsoluteTime { .. })
        ) || self.request.stop == crate::simulation::GameStopPolicy::Victory
        {
            DiagnosticCode::Horizon
        } else {
            self.failure_code.clone()
        };
        let loc = self
            .occurrences
            .last()
            .filter(|record| record.completed_us.is_none())
            .map_or(&self.program.options_location, |record| &record.loc)
            .clone();
        self.diagnostic = Some(Diagnostic {
            code,
            message: self
                .execution
                .halt_diagnostic
                .clone()
                .unwrap_or_else(|| "Source input did not complete.".into()),
            loc,
        });
    }

    pub(super) fn failure_context(&self) -> Option<SourceFailureContext> {
        self.diagnostic.as_ref()?;
        let command_index = self
            .execution
            .rejected_action
            .as_ref()
            .map(|rejected| rejected.command_index);
        let occurrence_index = command_index
            .and_then(|index| {
                self.occurrences
                    .iter()
                    .find(|record| record.command_indices.contains(&index))
            })
            .or_else(|| {
                self.occurrences
                    .last()
                    .filter(|record| record.completed_us.is_none())
            })
            .map_or(self.occurrences.len(), |record| record.index);
        Some(SourceFailureContext {
            occurrence_index,
            command_index,
        })
    }
}
