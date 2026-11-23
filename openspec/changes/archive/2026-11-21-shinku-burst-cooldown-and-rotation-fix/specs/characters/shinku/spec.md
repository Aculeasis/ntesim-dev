# Spec Delta: characters/shinku

## MODIFIED Requirements

### Requirement: Dragonflame Verdict Finisher
Once all Crimson Judgment stacks are consumed or upon Surging Crimson expiration, a subsequent Ultimate input SHALL execute Dragonflame Verdict (`UltraSkill_Rage_Damage` 10 hits followed by `UltraSkill2_Rage_Damage` finisher). Ultimate ability cooldown commences at initial Crimson Fury commitment (20.0s duration); Dragonflame Verdict execution SHALL preserve the remaining commitment cooldown rather than imposing an additional subsequent full cooldown penalty.

#### Scenario: Dragonflame Verdict execution
- **GIVEN** Shinku is in Surging Crimson with 0 Crimson Judgment stacks
- **WHEN** Ultimate input is committed
- **THEN** Dragonflame Verdict executes dealing 10 rapid strikes and the 400% ATK finisher nuke
- **AND** Surging Crimson stance concludes
- **AND** Shinku's Ultimate ability cooldown retains the remaining duration from the initial Crimson Fury commitment.
