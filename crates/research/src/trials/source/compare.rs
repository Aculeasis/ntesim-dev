use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SignedFraction {
    /// Decimal strings preserve signed values beyond JavaScript's integer range.
    pub numerator: String,
    pub denominator: String,
}

impl SignedFraction {
    pub(super) fn difference(
        candidate: ExactFraction,
        baseline: ExactFraction,
    ) -> Result<Self, String> {
        let negative = candidate
            .try_cmp(baseline)
            .map_err(|e| e.to_string())?
            .is_lt();
        let magnitude = if negative {
            baseline.sub(candidate)?
        } else {
            candidate.sub(baseline)?
        };
        Ok(Self {
            numerator: format!("{}{}", if negative { "-" } else { "" }, magnitude.numerator),
            denominator: magnitude.denominator.to_string(),
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pair {
    pub seed: u64,
    pub eligible: bool,
    pub damage_delta: Option<SignedFraction>,
    pub dps_delta: Option<SignedFraction>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Comparison {
    pub input_sha256: String,
    pub direction: String,
    pub complete: bool,
    pub baseline: Batch,
    pub candidate: Batch,
    pub pairs: Vec<Pair>,
    pub mean_damage_delta: Option<SignedFraction>,
    pub mean_dps_delta: Option<SignedFraction>,
}

pub fn compare(baseline: &str, candidate: &str, request: &Request) -> Result<Comparison, String> {
    let left = request.prepare(baseline)?;
    let right = request.prepare(candidate)?;
    if context(&left)? != context(&right)? {
        return Err("comparison dummy/stop/combat assumption contexts differ".into());
    }
    let mut left_request = request.clone();
    // One combined budget, including trials that fail or do not retain a receipt.
    left_request.max_total_events = request.max_total_events / 2;
    left_request.max_receipt_bytes = request.max_receipt_bytes / 2;
    let baseline_report = batch(baseline, &left_request)?;
    let mut right_request = request.clone();
    right_request.max_total_events -= baseline_report.events_charged;
    right_request.max_receipt_bytes -= baseline_report.retained_receipt_bytes;
    let candidate_report = batch(candidate, &right_request)?;
    let mut pairs = Vec::new();
    for seed in &request.seeds {
        let left = baseline_report.runs.iter().find(|run| run.seed == *seed);
        let right = candidate_report.runs.iter().find(|run| run.seed == *seed);
        let pair = match (left, right) {
            (Some(left), Some(right)) if left.eligible && right.eligible => {
                let left = left
                    .execution
                    .as_ref()
                    .ok_or("missing baseline execution")?;
                let right = right
                    .execution
                    .as_ref()
                    .ok_or("missing candidate execution")?;
                Pair {
                    seed: *seed,
                    eligible: true,
                    damage_delta: Some(SignedFraction::difference(
                        right.report.total_damage_exact,
                        left.report.total_damage_exact,
                    )?),
                    dps_delta: Some(SignedFraction::difference(dps(right)?, dps(left)?)?),
                }
            }
            _ => Pair {
                seed: *seed,
                eligible: false,
                damage_delta: None,
                dps_delta: None,
            },
        };
        pairs.push(pair);
    }
    let complete = baseline_report.complete && candidate_report.complete;
    let mean_damage_delta = match (candidate_report.mean_damage, baseline_report.mean_damage) {
        (Some(right), Some(left)) if complete => Some(SignedFraction::difference(right, left)?),
        _ => None,
    };
    let mean_dps_delta = match (candidate_report.mean_dps, baseline_report.mean_dps) {
        (Some(right), Some(left)) if complete => Some(SignedFraction::difference(right, left)?),
        _ => None,
    };
    Ok(Comparison {
        input_sha256: identity(&[baseline.as_bytes(), candidate.as_bytes(), &encode(request)?]),
        direction: "candidate-minus-baseline".into(),
        complete,
        baseline: baseline_report,
        candidate: candidate_report,
        pairs,
        mean_damage_delta,
        mean_dps_delta,
    })
}
