# Spec Delta

## MODIFIED Requirements

### Requirement: Relative damage-per-second accounting

Canonical team, actor, source and target DPS SHALL use each run's actual relative elapsed seconds in exact native analysis, experiments and comparisons. Studio SHALL additionally receive native absolute DPS for report selection without changing canonical relative values or eligibility. An undefined rate SHALL remain unavailable without fabricated denominators or a success-only subset average. Exports SHALL retain the basis of each value.

#### Scenario: Damage within stopped time
- **GIVEN** a completed trial deals 120000 damage over 120 relative seconds and 140 absolute seconds
- **WHEN** its damage rates and contributions are calculated
- **THEN** canonical team DPS is exactly 1000 and all canonical actor, source and target contributions use the same 120-second denominator
- **AND** the absolute report view uses the 140-second denominator for its displayed rates without changing canonical relative DPS

#### Scenario: Zero relative elapsed duration
- **GIVEN** all targets are defeated during a timer pause before any relative time has elapsed
- **WHEN** the completed combat is reported or compared
- **THEN** damage, absolute duration, pause provenance and completion remain available but relative DPS is explicitly unavailable
- **AND** no infinity, epsilon denominator or absolute-time fallback is presented as relative DPS
- **AND** comparisons or multi-seed DPS statistics requiring that undefined rate remain unavailable
- **AND** a defined absolute report rate does not restore relative comparison eligibility

### Requirement: Compact dual-clock observations

Detailed reports SHALL retain one absolute event stream and compact actual timer-pause intervals from which relative event times can be reconstructed. Batch observations SHALL retain both elapsed totals and native summary values for both bases without detailed timelines for every seed. Additional summary data SHALL NOT duplicate detailed reports, event streams, timeline series or static contribution identities. Timer recording SHALL remain event-driven.

#### Scenario: Many trials with one inspectable sample
- **WHEN** a multi-seed run includes repeated timer pauses
- **THEN** each seed retains relative and absolute duration and summary values for statistics, while only the existing detailed sample path retains its complete playback observations
- **AND** the report contains neither a duplicated relative event stream nor periodic frames created solely to sample the timer or shield capacity
- **AND** a basis selection requires no new combat execution or relative damage timeline

#### Scenario: Restore and replay recorded clocks
- **WHEN** a detailed report is exported, restored and replayed
- **THEN** its intervals reconcile absolute minus paused time to relative elapsed time exactly, including a pause clipped by failure
- **AND** malformed or inconsistent clock data is rejected before replacing a valid displayed report

## ADDED Requirements

### Requirement: Native report summary time bases

Native report summaries SHALL calculate DPS, retained/granted EPS and gross HPS from the same settled totals divided by each trial's actual relative or absolute seconds. Native SHP SHALL weight active living actor capacity by advancing time in the corresponding basis. Statistics SHALL summarize per-trial values in each basis. A zero denominator SHALL make that basis-dependent value unavailable, even for zero activity.

#### Scenario: Per-trial rates precede aggregation
- **GIVEN** two completed trials each deal 1200 damage, last 10 relative seconds, and last 20 and 30 absolute seconds respectively
- **WHEN** native summary statistics are calculated
- **THEN** relative mean DPS is 120 and absolute mean DPS is 50, from absolute per-trial rates 60 and 40
- **AND** the absolute mean is not calculated as mean damage divided by mean absolute duration
- **AND** minima, maxima, standard deviation and quartiles use the corresponding per-trial values

#### Scenario: Relative shield weighting excludes stopped intervals
- **GIVEN** a trial lasts 10 relative seconds and 20 absolute seconds, with 10000 available shield HP only during its 10 paused absolute seconds
- **WHEN** native summary observations are finalized
- **THEN** absolute SHP is 5000 HP and relative SHP is zero HP
- **AND** grant, depletion, replacement, active-recipient switch, defeat and expiry retain their actual causal boundaries

#### Scenario: Zero relative support denominator
- **GIVEN** a completed trial has positive absolute duration and zero relative duration
- **WHEN** support summary observations are finalized
- **THEN** relative EPS, granted EPS, HPS and SHP are unavailable and supported absolute values remain inspectable
- **AND** an aggregate requiring an unavailable value does not omit that seed or substitute an epsilon denominator

#### Scenario: No pause control
- **WHEN** a supported completed trial contains no mode-timer pause and has positive duration
- **THEN** both summary bases agree for DPS, EPS, HPS and SHP
- **AND** changing the report basis cannot change damage, grants, healing, admitted inputs, event order or completion
