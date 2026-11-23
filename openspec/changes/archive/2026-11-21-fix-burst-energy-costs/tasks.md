# Tasks

## 1. Engine & Action Projection

- [x] 1.1 Update `crates/content/src/build/public_actions.rs` to derive the default Burst energy cost dynamically from `ntesim_data::character::definition(character).base.energy`, ensuring all 120-capacity characters (Lacrimosa, Zankou, Sakiri, Adler, Aurelia, Haniel, Baicang, Fadia, Daffodil) spend 120 energy. Verify with `python tools/cargo.py test -p ntesim_content --lib build::public_actions`.
- [x] 1.2 Audit and harmonize any hardcoded Burst energy cost constants across character modules in `crates/content/src/characters/` to guarantee consistency with definition capacity. Verify with `python tools/cargo.py check`.

## 2. Test Fixtures and Scenarios Reconciliation

- [x] 2.1 Reconcile affected simulation tests in `crates/content/src/simulation/tests/` (including `aurelia_actions.rs`, `zankou_actions`, `lacrimosa`, `adler`) that asserted 100 energy spend or initialized with 100 energy, ensuring they test 120 energy expenditure. Verify with `python tools/cargo.py test -p ntesim_content`.
- [x] 2.2 Reconcile DSL examples and review receipts (such as `examples/lacrimosa-rose-continuing-energy.nte` and `reviews/q10-energy-ledger/lacrimosa-rose-continuing-energy.json`) to confirm 120 energy cost compiles and settles cleanly. Verify with `python tools/cargo.py test -p ntesim_cli`.

## 3. Documentation & Checkpoint Verification

- [x] 3.1 Update `docs/mechanics/characters/zankou/action-damage.md` and any related mechanics dossiers to replace obsolete mentions of 100 Burst energy with the native 120 cost. Verify documentation consistency with `git diff docs/`.
- [x] 3.2 Run OpenSpec validation and full project checkpoint checks. Verify with `python tools/openspec.py validate fix-burst-energy-costs` and `python tools/checkpoint.py run`.
