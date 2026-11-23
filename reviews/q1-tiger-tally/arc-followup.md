# Commander Tiger Talisman: talismans and Arc follow-up

This addendum extends the preserved [frozen owner receipt](receipt.json). It supersedes that receipt's “not wired” and “unmodeled” status for the cast dispatch, Left/Right talisman window, and Boss Arc clauses. The receipt itself remains unchanged so its original evidence identity and hashes stay intact.

## Frozen source bound

The source is the existing Global cutoff package `global-2026-10-04`, identified in `receipt.json`. The passive description says Redirect Skill grants Left, Ultimate grants Right, the opposite talisman within `buff_TigerTally_CD4` unlocks Commander Tiger Talisman, and Arc increases the wearer's damage to Bosses by `buff_TigerTally_Qup` for `buff_TigerTally_CD3`.

The refinement rows `DT_ForkUpgradeStarDataTable/upgradestar_pack_fork_TigerTally_1..5` bind the talisman window to 15 seconds at every tier and the Boss damage ratios to 10%, 12.5%, 15%, 17.5%, and 20%. `CT_ForkBuff` binds the Arc duration to 10 seconds at every tier. The existing normal buff remains 15 seconds, up to two stacks, with a frozen 15% to 30% ratio curve. The projection preserves all of these parameters in `native_parameter_report` and compiles the supported subset into `tiger_tally_atk`.

## Executed model

`gear::on_cast` dispatches to `tiger_tally::on_cast`; the actor's equipment runtime stores Left and Right deadlines. Redirect Skill grants Left and Ultimate grants Right. A new same-side grant refreshes that side's deadline. A cast of the opposite side strictly before the stored deadline consumes the pair and activates a wearer-only Arc bonus. The Arc damage bonus resets one 10-second buff on retrigger and adds into `M_bonus` against a target carrying the existing content-owned Boss status tag. It applies to the wearer's damage packets, including non-Normal packets; the separate two-stack normal bonus remains restricted to Normal-role packets. Static ATK still scales from base ATK.

The distinct runtime identity is `weapons.tiger_tally_v2`. Strict requests must include its exact description. The previous `weapons.tiger_tally_v1` does not describe the added clauses and is not treated as an alias.

The pair's automatic Arc activation, consumption, same-side refresh, Arc refresh, and Boss-tag eligibility are bounded implementation hypotheses: the frozen description does not specify them. The 15-second pairing comparison uses the runtime's exclusive expiry boundary (`now_us >= expires_at_us` is expired). They are covered by owner-level tests, including both cast orders, same-side refresh, Arc reset and the exact expiry boundary.

## Verification and limits

- `python tools/cargo.py test --locked --offline -p ntesim-content tiger_tally --lib` passed, covering T1–T5 projection, dispatch, both talisman orders, same-side window refresh, Arc refresh/expiry, Boss-only damage, additive-zone arithmetic, and strict hypothesis identity.
- `python tools/cargo.py clippy --locked --offline -p ntesim-content --tests` passed.
- `node tools/web.mjs check` passed for the Studio scenario contract.
- `python tools/cargo.py build --locked --offline -p ntesim-cli` and `python tools/cargo.py build --locked --offline -p ntesim-wasm --release --target wasm32-unknown-unknown` passed; `node tools/build_studio_stats.mjs` regenerated the catalog for 23 characters and 51 weapons.
- `node tools/web.mjs test tests/weapon_passives.test.mjs` passed against the rebuilt WASM.

Critical Riposte remains unmodeled because the engine has no corresponding action role or damage packet representation. The Boss gate uses the existing modeled target status tag; native Boss identity binding remains separate work. Global release availability and native in-game timing/refresh behavior remain unverified. This addendum does not claim full Commander Tiger Talisman, weapon, or character coverage.
