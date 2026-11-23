use super::GameAdmissionReason;
use ntesim_core::kernel::action::ActionError;

pub(crate) struct AdmissionFailure {
    pub(crate) reason: GameAdmissionReason,
    pub(crate) message: String,
}

pub(super) fn timing_reason(error: &ActionError) -> GameAdmissionReason {
    match error {
        ActionError::RecoveryLock { until } => GameAdmissionReason::Recovery { until_us: *until },
        ActionError::Cooldown { until, .. } => GameAdmissionReason::Cooldown { until_us: *until },
        ActionError::TimeReversal { last, .. } => {
            GameAdmissionReason::TimeReversal { last_us: *last }
        }
        ActionError::Overflow => GameAdmissionReason::TimingOverflow,
        other => GameAdmissionReason::Other {
            message: format!("{other:?}"),
        },
    }
}
