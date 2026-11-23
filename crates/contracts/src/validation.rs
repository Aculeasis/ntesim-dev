use crate::SCHEMA_VERSION;
use crate::models::{Dependency, Diagnostic, Quality, Scenario, Validation};
use std::collections::{BTreeMap, BTreeSet};

impl Scenario {
    pub fn validate(
        &self,
        content_id: &str,
        dependencies: &BTreeMap<String, Dependency>,
    ) -> Validation {
        let mut errors = Vec::new();
        let mut push = |code: &str, path: String, message: String| {
            errors.push(Diagnostic {
                code: code.into(),
                path,
                message,
            })
        };
        if self.schema_version != SCHEMA_VERSION {
            push(
                "SCHEMA",
                "schema_version".into(),
                "unsupported scenario schema".into(),
            );
        }
        if self.content_id != content_id {
            push(
                "PACKAGE",
                "content_id".into(),
                "scenario/data version mismatch".into(),
            );
        }
        if self.duration_us == 0 {
            push(
                "DURATION",
                "duration_us".into(),
                "duration must be positive".into(),
            );
        }
        if self.dependencies.is_empty() {
            push(
                "DEPENDENCIES",
                "dependencies".into(),
                "empty execution dependency graph is not a validated scenario".into(),
            );
        }
        let mut seen = BTreeSet::new();
        let mut bounded = false;
        let mut unbounded = false;
        for id in &self.dependencies {
            if !seen.insert(id) {
                push("DUPLICATE", id.clone(), "duplicate dependency".into());
            }
            let Some(dependency) = dependencies.get(id) else {
                push(
                    "UNKNOWN_DEPENDENCY",
                    id.clone(),
                    "dependency is absent from model registry".into(),
                );
                continue;
            };
            if !dependency.implemented {
                push(
                    "NOT_IMPLEMENTED",
                    id.clone(),
                    "execution model is absent; a scalar assumption cannot replace it".into(),
                );
            }
            if !dependency.confirmed {
                if !self.assumptions.contains_key(id) {
                    push(
                        "MISSING_EVIDENCE",
                        id.clone(),
                        dependency
                            .gap
                            .clone()
                            .unwrap_or_else(|| "essential evidence is unknown".into()),
                    );
                } else if self.assumptions[id].interval.is_some() {
                    bounded = true;
                } else {
                    unbounded = true;
                }
            }
        }
        for (id, a) in &self.assumptions {
            if !seen.contains(id) {
                push(
                    "UNUSED_ASSUMPTION",
                    id.clone(),
                    "assumption is outside scenario dependencies".into(),
                );
            }
            if dependencies.get(id).is_some_and(|d| d.confirmed) {
                push(
                    "CONFIRMED_OVERRIDE",
                    id.clone(),
                    "hypothesis cannot silently replace verified data".into(),
                );
            }
            if !a.value.is_finite() || a.unit.trim().is_empty() || a.justification.trim().is_empty()
            {
                push(
                    "INVALID_ASSUMPTION",
                    id.clone(),
                    "finite value, unit and justification required".into(),
                );
            }
            if let Some([low, high]) = a.interval
                && (!low.is_finite() || !high.is_finite() || low > a.value || a.value > high)
            {
                push(
                    "INVALID_INTERVAL",
                    id.clone(),
                    "finite interval must contain assumed value".into(),
                );
            }
        }
        let quality = if !errors.is_empty() {
            Quality::Incomplete
        } else if unbounded {
            Quality::UnboundedHypothesis
        } else if bounded {
            Quality::BoundedHypothesis
        } else {
            Quality::Confirmed
        };
        Validation {
            valid: errors.is_empty(),
            quality,
            diagnostics: errors,
        }
    }
}
