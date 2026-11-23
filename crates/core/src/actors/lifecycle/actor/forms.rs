use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(try_from = "FormStateSnapshot")]
pub struct FormState {
    pub(crate) current_form: Option<FormInstance>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FormStateSnapshot {
    current_form: Option<FormInstance>,
}

impl TryFrom<FormStateSnapshot> for FormState {
    type Error = String;

    fn try_from(snapshot: FormStateSnapshot) -> Result<Self, Self::Error> {
        let state = Self {
            current_form: snapshot.current_form,
        };
        state
            .validate()
            .map_err(|error| format!("invalid form state: {error:?}"))?;
        Ok(state)
    }
}

impl FormState {
    pub fn validate(&self) -> Result<(), LifecycleError> {
        if let Some(form) = &self.current_form
            && (form.form_id.is_empty()
                || form
                    .expires_at_us
                    .is_some_and(|expires| expires <= form.entered_at_us))
        {
            return Err(LifecycleError::InvalidFormState);
        }
        Ok(())
    }

    pub fn current_form(&self) -> Option<&FormInstance> {
        self.current_form.as_ref()
    }

    pub fn enter_form(
        &mut self,
        form_id: String,
        duration_us: Option<u64>,
        now_us: u64,
    ) -> Result<(), LifecycleError> {
        self.validate()?;
        if form_id.is_empty() || duration_us == Some(0) {
            return Err(LifecycleError::InvalidFormState);
        }
        let expires_at_us = duration_us
            .map(|duration| {
                now_us
                    .checked_add(duration)
                    .ok_or(LifecycleError::DeadlineOverflow)
            })
            .transpose()?;
        self.current_form = Some(FormInstance {
            form_id,
            entered_at_us: now_us,
            expires_at_us,
        });
        self.validate()?;
        Ok(())
    }

    pub fn exit_form(&mut self) -> Option<FormInstance> {
        self.current_form.take()
    }

    pub fn active_form(&self, now_us: u64) -> Option<&str> {
        let form = self.current_form.as_ref()?;
        if form.expires_at_us.is_some_and(|exp| now_us >= exp) {
            return None;
        }
        Some(form.form_id.as_str())
    }

    pub fn advance_time(&mut self, now_us: u64) {
        if self
            .current_form
            .as_ref()
            .and_then(|form| form.expires_at_us)
            .is_some_and(|exp| now_us >= exp)
        {
            self.current_form = None;
        }
    }
}
