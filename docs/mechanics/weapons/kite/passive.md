# Watch Your Heads! — passive

- Native identity: `weapon:fork_Kite`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 31.0 | Stage 0 | Max Lv 20 | +0.0 | CritDamageBase: 16.00% |
| Lv 20 | 108.0 | Stage 1 | Max Lv 30 | +21.0 | CritDamageBase: 20.00% |
| Lv 40 | 189.0 | Stage 2 | Max Lv 40 | +41.0 | CritDamageBase: 24.00% |
| Lv 60 | 270.0 | Stage 3 | Max Lv 50 | +62.0 | CritDamageBase: 28.00% |
| Lv 80 | 351.0 | Stage 4 | Max Lv 60 | +82.0 | CritDamageBase: 32.00% |
| Lv 80 | 351.0 | Stage 5 | Max Lv 70 | +103.0 | CritDamageBase: 36.00% |
| Lv 80 | 351.0 | Stage 6 | Max Lv 80 | +124.0 | CritDamageBase: 40.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **475.0** (351.0 + 124.0); Secondary Stat = **CritDamageBase 40.00%**.

## Refinement scaling

**Passive Name**: Ghostly Kite

> Increases ATK by <lv>{0}</> and Lakshana DMG to targets affected by Remora and Stain by <lv>{1}</> for <lv>{2}</>s after the wearer casts a Redirect Skill. Effect does not stack.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Kite_AtkUp | buff_Kite_Up | buff_Kite_CD |
|---|---|---|---|
| T1 | 10.00% | 10.00% | 15.00 |
| T2 | 12.00% | 12.00% | 15.00 |
| T3 | 14.00% | 14.00% | 15.00 |
| T4 | 16.00% | 16.00% | 15.00 |
| T5 | 18.00% | 18.00% | 15.00 |

## Activation and trigger mechanics

`weapons.kite_v2` retains the fifteen-second wearer-only Redirect Skill ATK window. It does not stack or refresh while active; another admitted Skill after expiry starts a new window. ATK% scales base ATK, preserves field transitions and never transfers to teammates.

The native `Buff_Fork_Kite_Effect` CDO places `DamageUpLakshanaBase` in `CustomApplicationModifierInfos[0]` with requirement `Con_Kite`; its ordinary modifier is additive `AtkUp`. `Con_Kite` CDO lists `HunXiang` and `GuangXiang`. Its `CanApplyGameplayEffectContent` statements30–111 call `HasAnyMatchingGameplayTags` on the **target** and return true for either tag. Thus simultaneous Stain/Remora is not required, and the two statuses must not double the bonus.

The executable target branch queries the actual recipient's live modeled Stain or Remora status at packet delivery. During the wearer window, matching Lakshana damage adds the refinement ratio once to its ordinary damage zone. Unmarked targets and other elements receive the ATK window alone. Half-open target and weapon deadlines both apply. This replaces the historical unconditional unsupported error: both statuses now have shared executable consumers.

**Model limit:** `Con_Kite` statements116–614 also inspect the source's `GetReactionResultType` when neither target tag is present, returning true for ordinals5/6 and false for0–4. The source getter's capture/reset and native custom-modifier evaluation time remain unresolved. The executable v2 scope uses live target statuses, as the shared Stain model does for Devil's Blood; it does not invent a permanent source classification. This increment binds the evidenced target branch rather than claiming complete native condition equivalence.

Carrier export SHA256 `d2f4d8d1160e80e21d8831f740f3e101222a151d8e72b873cd6d5f7d62ecaacc`; condition export `1f572b1526a429cd2968deddada481403e42b5da5d55866e90114a4237228b68`. The native reaction ordinals are mapped in [cycle source findings](../../shared/cycle-source-findings.md#reaction-result-classification). No new source payload or runtime catalog is retained.

## Complete implementation handoff

[The public source](../../../../examples/kite-marked-cycles.nte) starts with a qualified Aurelia-to-Skia Stain switch, admits the first Skill, observes status/window expiry, then spends the remaining authored Skia/Zero Cycle on Remora and a second Skill window. It never resets Cycle, energy, buffs or cooldowns. Rust checks T1–T5 against the same source with only the passive removed; native/WASM/actual Worker compare complete reports, aggregation and replay. A separate overlapping-status case rejects a doubled bonus. Existing checks retain no-refresh, non-Redirect, base-versus-compiled ATK and expiry counterexamples.

Independent normalized T1 arithmetic: base/final ATK1000 plus passive.10 gives1100. With existing general damage.50 and qualifying Lakshana bonus.10, coefficient1 damage is `1100 *(1+.50+.10) =1760` before the independent target Stain factor. The unmarked value is1650; no window gives1500. This is a ratio88/75 against the same1500 baseline, not an independent1.10 final multiplier. At T5 the marked value is1982.4. Two live tags still grant the single.10/.18 ratio.

Compiled level80 weapon475 base ATK and secondary Crit DMG40% remain build stats and are not authored twice. Full Skia kit coverage, native getter capture, exact condition evaluation and Global release availability retain their separate limits.
