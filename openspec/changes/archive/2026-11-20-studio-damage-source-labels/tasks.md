# Tasks

## 1. Curated and Fallback Source Label Engine

- [x] 1.1 In `web/src/domain/reports/source-label.ts`, add curated mappings for known skill effects and DoTs: Zankou (Heartwrench DoT, Vile Ash DoT, Sanguine Dash ordinary/enhanced stages, Soulcross ordinary/enhanced/finisher, Bloodfeast Reverie, enhanced Inferno Flamenco), weapon procs (Harsh Sound, Nagging Sound, Resonance Finisher, Counter Slash, Chaos Thorn), and secondary actions (Air Attack / Plunge, Dodge Counter). Verify with unit tests in `web/tests/report_controls.test.mjs`.
- [x] 1.2 Implement an algorithmic humanization fallback function in `web/src/domain/reports/source-label.ts` that systematically strips internal prefixes (`GE_Player_`, `GE_`) and suffixes (`_Damage`, `_Damage_New`), and formats combo hits, branches, stances, and stages into readable labels. Verify with unit tests in `web/tests/report_controls.test.mjs`.
- [x] 1.3 Update `sourceLabel` composition logic to assemble `${abilityTitle} · ${effectLabel}` cleanly without redundant duplicate names, ensuring no multi-packet ability produces identical duplicate rows. Verify with unit tests in `web/tests/report_controls.test.mjs`.

## 2. Verification and Checkpoint

- [x] 2.1 Run focused web tests via `node tools/web.mjs test` to verify all existing and new report control tests pass without regressions.
- [x] 2.2 Run `python tools/checkpoint.py run` to verify overall repo health, WASM builds, and browser smoke checks before delivery.
