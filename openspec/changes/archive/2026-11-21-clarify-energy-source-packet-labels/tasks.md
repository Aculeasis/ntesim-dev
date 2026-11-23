# Tasks

## 1. Energy Source Catalog & Grouping

- [x] 1.1 Register Riposte, Support, and Plunge event kinds in `directPacketSources` in `web/src/domain/reports/energy-sources.ts` and verify with focused assertions in `web/tests/source_labels.test.ts`
- [x] 1.2 Update `web/src/features/results/EnergyPanels.tsx` to key the aggregation Map by `[source.source, details.label]` and verify that identically-labeled rows merge cleanly

## 2. Verification & Checkpoint

- [x] 2.1 Add component-level test in `web/tests/studio.test.tsx` verifying that Energy Per Source renders distinct specialized packet rows and merges rows sharing a label
- [x] 2.2 Run focused web tests and `python tools/checkpoint.py run` to ensure all linters, formatting, and test suites pass
