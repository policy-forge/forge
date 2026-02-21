# WI-22 Edge-Case Fixture Map

This directory contains golden-file fixtures for WI-22 edge-case coverage.

Parent edge cases:

- `ec01-no-headings/`: headingless input -> descriptive failure (`expected-error.txt`)
- `ec02-compound-atomic/`: compound split + atomic preservation (`expected-catalog.json`, `expected-component-definition.json`)
- `ec03-empty-sections/`: empty sections represented without conversion failure (`expected-catalog.json`, `expected-warnings.txt`)
- `ec04-missing-metadata/`: metadata defaults (`title`, `version`, `author`) and per-field warnings (`expected-catalog.json`, `expected-warnings.txt`)
- `ec05-whitespace-only/`: paired inputs for stable ID equality
- `ec06-substantive-change/`: paired inputs for stable ID rotation (`expected-warnings.txt`)
- `ec07-malformed-citation/`: malformed URL retained with `url-status=unvalidated` (`expected-catalog.json`)
- `ec09-file-not-found/`: missing source path failure contract (`expected-error.txt`)
- `ec10-multiple-errors/`: validation fixture containing schema + semantic issues (`input.md`, `expected-errors.txt`)

Supplemental Should-Have scenarios:

- `ec-citation-unusual-positions/`: citation extraction from unusual placements
- `ec-parameter-like-content/`: parameter-like prose preservation

Shared support files:

- `source-profile.json`: existing-file reference for component strategy test runs.

Style guardrails:

- Fixtures should resemble plausible policy prose.
- Expected error/warning files contain required substrings, not full message equality.
- EC-8 and performance benchmarking artifacts are intentionally excluded.

Fixture realism review outcomes (WI-21 style guardrails):

- [x] Inputs use policy-like language and section structure (not synthetic token strings)
- [x] Edge fixtures preserve realistic metadata/frontmatter patterns where applicable
- [x] Failure fixtures keep minimal structure needed to isolate the target behavior
- [x] No benchmark/performance artifacts were added under WI-22 scope
