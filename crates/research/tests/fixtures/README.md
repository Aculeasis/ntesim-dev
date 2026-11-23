# Research test inputs

These small CSV files exercise different consumer boundaries, rather than duplicating the maintained production table in source literals:

- `profile-variants.csv`: three ordered profiles used by scenario/profile tests.
- `mixed-variants.csv`: seven variants covering complete and unsupported/failed research results, repeated archetypes and both Shinku roster alternatives.
- `mint-variant.csv`: one variant for focused composition, search and deliverable tests.
- `built-roster.csv`: an explicitly authored four-character build projection.
- `finalists.csv`: two baseline candidates for finalist comparison tests.

The schema is the same six-column schema documented in [maintained research inputs](../../../../presets/research/README.md). These rows are test cases, not claims of current tier placement or executable optimal builds. Tests keep independent expected counts, arithmetic and failure causes at the owning Rust boundary.
