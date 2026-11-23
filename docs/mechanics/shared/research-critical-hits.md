# Explicit research critical-hit policy

The explicit policy enables seed-sensitive experiments in the existing partial runner without inventing a game RNG/crit binding. Full scenario quality remains `incomplete`; strict readiness and game DPS flags remain false.

`GameScenarioRequest.research_crit` is optional. Existing requests omit it and retain non-critical settlement. Supplying it requires `mode: research`, a nonempty `hypothesis`, explicit per-character `eligible_effects` (including the supported `*` wildcard for current/future owner packets), matching `actor_sequences` and `roll_policy: independent_per_target_hit_v1`. Unknown policies/fields, missing actors, empty effect lists, reused or invalid sequence selectors and invalid rates are rejected; nonnegative raw rates above one retain their provenance and clamp to one for sampling. Crit-rate denominators must be in 1..=2^32.

Each eligible delivered hit samples uniformly from [0, denominator) using the named actor's versioned PCG32 stream with rejection sampling, then compares against the numerator. Separate actors cannot shift each other's stream. Endpoints 0%/100% consume no words; unlisted effects do not roll. This policy declares independent draws per delivered target/hit; it does not assert that NTE uses those rules. The current adapter schedules target 0 only, so additional target rows do not create AoE hits.

Hit receipts retain actor/target/effect/time, raw `crit_rate`, non-critical and critical exact fractions, selected branch, all accepted/rejected raw RNG words, rounded damage and actual HP loss. `crit_rate` retains the summed stat or fixed-rate replacement; sampling uses its effective probability clamped to `[0,1]`, so a raw rate above one is guaranteed critical without consuming RNG words. Settlement rounds to nearest integer with ties upward using integer arithmetic. This is an explicit preview rounding policy; game quantization still needs evidence. Total exact damage, seed, horizon, policy and final RNG snapshot travel with the report. RNG snapshots are not a restoration API.

Admission failures consume no crit draw. Events strictly beyond the horizon remain pending, are not settled and consume no draw; a hit exactly on the horizon is included. Pending execution cannot establish a complete trial.

From `ntesim/`:

```powershell
python tools/cargo.py build --locked --offline -p ntesim-cli
target/debug/ntesim.exe run-game examples/research-crit.json
python tools/check.py -- python tools/verify_research_crit.py
```

Rust checks use published PCG golden words, probability endpoints, distinct seed replay, rejected policies/streams, admission failures, unhit targets, horizon boundaries and large exact-rounding endpoints. The independent Python PCG/Fraction oracle checks six seeds against explicit example inputs, exact per-hit totals/HP, report provenance, byte-identical replay and strict/policy/hypothesis rejection. These tests verify implementation, not underlying game evidence or full Q10 acceptance.

Use `tools/verify_research_crit.py` and the Rust tests as the current numerical and causal checks when changing the model.
