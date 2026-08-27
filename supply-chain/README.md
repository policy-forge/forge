# Supply-chain posture

`cargo vet --locked` gates CI (`.github/workflows/ci.yml`, `audit` job).

- `config.toml` imports the Mozilla, Embark-Studios, and (historically)
  community cargo-vet audit registries; `imports.lock` pins them.
- Most of the graph is still covered by blanket `safe-to-deploy` exemptions
  rather than real audits. That is a deliberate transitional posture, not an
  endorsement: exemptions attest author-side review only.
- Conversion backlog, highest priority first (F1049): crates that parse
  attacker-controlled bytes at runtime — `pdf-extract`, `lopdf`,
  `cff-parser`, `type1-encoding-parser`, `postscript`, `quick-xml`, `zip`,
  `unsafe-libyaml`, `wasmparser`, `wit-parser`, `wit-component`,
  `fancy-regex`, `nom`. Imported registries already cover some of these;
  run `cargo vet prune` after dependency bumps to drop exemptions that
  imported audits now satisfy.
- Application-side defense is independent of vet status: ingest enforces
  `max_size_bytes` (`src/ingest/mod.rs`), and DOCX decompression is bounded.
