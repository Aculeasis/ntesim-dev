# Echoes on Hold — passive

- Native identity: `weapon:fork_elyms`.
- Release: Global version 1.5. [Official patch notes](https://nte.perfectworld.com/en/article/news/gamenews/20261109/264948.html)
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve `Fork_rank5_1` and breakthrough group `fork_elyms_rank5`:

| Breakthrough Stage | Max Level | Curve ATK at Max Level | Breakthrough ATK Add | CRIT Rate |
|---|---:|---:|---:|---:|
| 0 | 20 | 128 | +0 | 9.60% |
| 1 | 30 | 176 | +26 | 12.00% |
| 2 | 40 | 224 | +51 | 14.40% |
| 3 | 50 | 272 | +77 | 16.80% |
| 4 | 60 | 320 | +102 | 19.20% |
| 5 | 70 | 368 | +128 | 21.60% |
| 6 | 80 | 416 | +154 | 24.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **570.0** (416.0 + 154.0); Secondary Stat = **CritBase 24.00%**.

## Refinement scaling

**Passive Name**: Voicemail #Redial

> Increases ATK by <lv>{0}</>. After the Wearer casts their Ultimate, dealing Follow-up Attack DMG 3 times grants the whole team <lv>{1}</> DEF Ignore and grants the Wearer an additional <lv>{2}</> DEF Ignore for <lv>{4}</>s. Resets the duration when triggered again. This effect is unique within the team.
> Grants the whole team <lv>{3}</> Cosmos Resistance Penetration for <lv>{4}</>s when the Wearer casts a Redirect Skill. Resets the duration when triggered again. This effect is unique within the team.
> Triggers all the above effects immediately when the Wearer enters combat.

Parameters across refinement ranks T1 to T5:

| Rank | ATK Bonus (`ntesim.echoes_on_hold.atk`) | Team DEF Ignore (`team_def_ignore`) | Wearer DEF Ignore (`wearer_def_ignore`) | Cosmos Penetration (`cosmos_penetration`) | Duration (`duration`) |
|---|---|---|---|---|---|
| T1 | 15.00% | 12.00% | 10.00% | 8.00% | 20.00s |
| T2 | 18.75% | 14.00% | 12.50% | 11.00% | 20.00s |
| T3 | 22.50% | 16.00% | 15.00% | 14.00% | 20.00s |
| T4 | 26.25% | 18.00% | 17.50% | 17.00% | 20.00s |
| T5 | 30.00% | 20.00% | 20.00% | 20.00% | 20.00s |

The refinement parameter identifiers are simulator-owned. Native scaling rows and the ordering of competing refreshes still need client verification.

## Activation and trigger mechanics

- **Combat Start (t = 0)**:
  - Both 20-world-second windows activate immediately upon entering combat:
    1. Team DEF Ignore (`echoes_on_hold.team_def_ignore`, `BuffRecipient::TeamAll`) + activating Wearer DEF Ignore (`echoes_on_hold.wearer_def_ignore`, `BuffRecipient::SelfActor`).
    2. Team Cosmos Resistance Penetration (`echoes_on_hold.cosmos_penetration`, `BuffRecipient::TeamAll`).
- **Cosmos Resistance Penetration Window**:
  - Refreshes for 20 seconds whenever the wearer casts a Redirect Skill (`InputActionRole::RedirectSkill`).
  - Scheduled in the cast buff phase before same-time hits (`CausalPriority::BuffTickOrExpiry`).
  - Live source modifiers apply the penetration to Cosmos hits and Cosmos Break contributions.
- **DEF Ignore Window**:
  - Armed by an admitted Ultimate cast (`InputActionRole::Ultimate`).
  - The counter advances when the wearer delivers qualifying Follow-up Attack packets (`State.Damage.Followup`) causing positive HP loss (`applied_hp_loss > 0`), including off-field attacks.
  - Multi-target hits share one causal packet group and count once across all targets. Separate packets at the same time count independently.
  - On the 3rd qualifying packet, the counter disarms and DEF Ignore refreshes for 20 seconds after damage settlement (`CausalPriority::ProcCallback`).
  - An admitted replacement Ultimate resets the counter and arms a fresh count.
- **Stacking and Shared Expiry**:
  - Both windows use `BuffStackingScope::Shared`. Reapplication replaces the previous snapshot and resets its deadline. The wearer bonus moves with the DEF Ignore window.
  - Each window expires independently at its world-time deadline before damage.
