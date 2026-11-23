use super::*;
use crate::simulation::GamePlungeInput;

impl RotationCompiler<'_> {
    pub(super) fn plunge_input(
        &mut self,
        action: &ActionInvocation,
    ) -> Result<Option<GamePlungeInput>, Diagnostic> {
        if action.action != "plunge" {
            return Ok(None);
        }
        let mut landing_delay_us = None;
        let mut height_bonus = None;
        for (key, expression) in &action.params {
            match key.as_str() {
                "target" => {}
                "hold" => {
                    return Err(failure(
                        "Plunge hold requires native live source execution.".into(),
                        &expression.loc,
                    ));
                }
                "airtime" if landing_delay_us.is_none() => {
                    let duration = self.duration(expression)?;
                    if duration == 0 {
                        return Err(failure(
                            "plunge airtime must be positive.".into(),
                            &expression.loc,
                        ));
                    }
                    landing_delay_us = Some(duration);
                }
                "height_bonus" if height_bonus.is_none() => {
                    let (numerator, denominator) = self
                        .eval(expression)?
                        .number()
                        .and_then(ExactNumber::to_safe_ratio)
                        .map_err(|message| failure(message, &expression.loc))?;
                    if numerator > denominator {
                        return Err(failure(
                            "plunge height_bonus must be within0..1.".into(),
                            &expression.loc,
                        ));
                    }
                    height_bonus = Some(crate::formula::Ratio {
                        numerator,
                        denominator,
                    });
                }
                _ => {
                    return Err(failure(
                        format!("Unsupported or repeated plunge parameter '{key}'."),
                        &expression.loc,
                    ));
                }
            }
        }
        Ok(Some(GamePlungeInput {
            landing_delay_us: landing_delay_us
                .ok_or_else(|| failure("plunge requires airtime.".into(), &action.loc))?,
            height_bonus: height_bonus
                .ok_or_else(|| failure("plunge requires height_bonus.".into(), &action.loc))?,
        }))
    }
}
