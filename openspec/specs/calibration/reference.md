# Tabular verification contract

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

`verify-tabular [FIXTURE.json]` invokes the partial formula verifier in [research](../../../crates/research/src/verification/tabular/mod.rs). The optional CLI fixture is a JSON array of cases, capped at 1 MiB; it has no schema/cases wrapper. Current case/report types live in [model.rs](../../../crates/research/src/verification/tabular/model.rs). Source kinds distinguish published calculators, community spreadsheets, published baselines and independent deterministic oracles. Labels alone do not establish independent game evidence.

## Inputs and arithmetic

Cases provide attacker level, base/percentage/flat ATK, CR/CD, damage bonus, hit ratio/flat term, target DEF/RES, separate DEF-shred/ignore, the negative-resistance model, expected noncritical/optional critical damage, tolerance and optional discrepancy note. Each also requires a nonempty reference, applicability limits and tolerance rationale. IDs must be nonempty and unique within a battery. Actual field names and serialization are owned by the Rust types, not a copied pseudocode schema.

The evaluator uses the shared direct-damage formula with an ordinary physical packet, noncritical and optional guaranteed-critical branches. It compares `raw_damage`, before packet quantization, even though the evaluation request also specifies floor rounding. It does not execute native actions, healing, shields, reactions or rotations. [Combat formulas](../../../docs/mechanics/shared/damage-and-stat-zones.md) own zones and unresolved game interpretations.

Reference values and `tolerance_pct` must be finite and nonnegative; the evaluator has no universal 0.01% upper tolerance. For positive references, `diff_pct = abs(engine - reference) / reference × 100`. A zero reference matches only zero engine damage; otherwise percentage tolerance is undefined and the case errors. Empty case lists reject.

## Status and report interpretation

The maximum noncritical/critical difference determines status in this order:

- `exact_match` when both computed raw values equal their references.
- `within_tolerance` when it is within the caller's tolerance.
- `discrepancy_documented` when it exceeds tolerance and a discrepancy note exists.
- `failed` otherwise.

The case's `passed` boolean and aggregate `passed_cases` always agree: only `exact_match` and `within_tolerance` pass. Even a tiny discrepancy fails a zero tolerance. Documented discrepancies are counted separately; they are not successful exact checks. CLI exit 0 requires `failed_cases=0` and `passed_cases=total_cases`; documented discrepancies therefore also produce exit 2 without incrementing `failed_cases`. Evaluation/input errors reject the request with exit 1.

Version 2 reports retain each complete input `case` with the computed values and status, so saved or relocated results can be independently reevaluated with their original context and tolerances. Reports bind engine version/source identity and retain `INCOMPLETE`, incomplete quality and false strict/DPS/acceptance flags. The in-memory report type has no per-table or request hashes. The CLI audits the package through the shared constant-report wrapper and adds `input_hashes`, `request_sha256` and `audit`; the request hash covers the supplied fixture bytes, or `default-tabular-battery` when no fixture is supplied. Keep the CLI wrapper and in-memory API distinct.

The default battery uses exact decimal/rational oracles with a declared `1e-10` percent allowance for binary float arithmetic. Its nominal packet is `1151 * 2.014725 * 1.542 * 1.1 * .5 * .8 = 1573.360161318`, and guaranteed crit multiplies by `1.94` to give `3052.31871295692`. The former `1573.461933` reference used an incorrect intermediate ATK; a loose `.01%` tolerance hid the error. The corrected ATK is `2318.948475`. The flat-ATK packet is an independent synthetic oracle (`988/1976`), with no acquired published timeline; the DEF-shred oracle likewise has no spreadsheet attribution. Calculator comparisons bind the pinned source in the [acquisition audit](../../../docs/DRIVE_CALCULATOR_RESEARCH.md).

JSON parsing enables `float_roundtrip` so saved floating point reference and result values retain their exact binary values; a zero tolerance cannot silently change meaning after serialization.

## Evidence limits

The report includes descriptive discrepancy records for negative RES, Nightmare expiry and Scorch triggers. These strings do not execute the mechanics and must not be read as native adapter certification. The negative branch remains unresolved between `standard_rational` and `linear_scaled`. Nightmare and Scorch now have executable selected lifecycle/activation models and independent Rust owner checks; their bounded rules do not establish native game behavior. See the [Lacrimosa specification](../characters/lacrimosa/spec.md), [selected Scorch model](../../../docs/mechanics/shared/cycles-and-break.md#selected-adler-scorch-dependency), [Drive Calculator findings](../../../docs/DRIVE_CALCULATOR_RESEARCH.md) and [formula contract](../../../docs/mechanics/shared/damage-and-stat-zones.md).

Use compatible spreadsheets and independent causal/numerical checks for each supported DPS model under [working rules](../../../docs/DEVELOPMENT.md). Passing this arithmetic harness does not complete plan E or validate a team's DPS.
