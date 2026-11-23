# Research inputs

These files are consumed presets, not a raw acquisition catalog or verified optimal builds.

- `variants.csv`: `archetype,tier,slot1,slot2,slot3,slot4`. Four distinct registered character slugs define each variant; rows preserve independent variants and input order. The 45 curated rows retain the published tier-list snapshot collected on 2026-10-04 from [Prydwen's team list](https://www.prydwen.gg/neverness-to-everness/team-tier-list/). Tier labels are historical research annotations, not current accuracy or readiness claims.
- `recommended-gear.csv`: `character,kind,name,requested_T,level,catalogue_match,source_url,optimizer_selected`. Rows retain the historical recommendations and their individual source URLs. The gear binder validates canonical identities and explicit assumptions; recommendations do not imply optimizer selection or legal native allocations.
- `conditions.json`: authored acceptance request settings corresponding to the existing `AcceptanceConditionsSummary` model. These explicit dummy/experiment assumptions are not recovered enemy profiles or raw scraped conditions.

Native research commands and WASM bundle these maintained files through `ntesim-data::research_inputs`. Explicit custom CSV/JSON paths are read as supplied and missing or malformed inputs fail. Packaged operation needs no checkout or historical package directory. Shared test-only CSVs live under `crates/research/tests/fixtures/`; independent numeric and causal expectations remain in their owning tests.
