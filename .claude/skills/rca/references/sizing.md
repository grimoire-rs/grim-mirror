# Fix size rubric

Shared by `/rca` and `/fix`. Size by blast radius, not hours. State the basis beside the letter.

| Size | Criteria |
|---|---|
| **S** | One function or file. No interface surface. An existing test pattern covers it. Zero siblings. |
| **M** | Up to 5 files in one crate, or the cause plus 1–3 sibling sites. New tests, no design choice. |
| **L** | Crosses crates or layers, **or** touches any interface surface below, **or** needs a choice between designs. |
| **XL** | Needs an ADR, the architectural assumption itself is wrong, a migration, or a concurrency/locking redesign. Split before building. |

**Bump +1** when the repro is flaky or Windows-only, or confidence is below medium.

## Interface surfaces

`grim-mirror` CLI flags and subcommands · exit codes · the `--format json` run report (`schema`, `status`, `reason` vocabularies) · the `mirror.toml` grammar · the `grim` surfaces it calls (`describe`, `convert`, `publish`, the `publish.toml` `mirror` table, the three annotation keys it writes) · `GRIM_BIN` and `GRIM_OFFLINE`.

Touching one is a decision, not a refactor. The mirror's own surfaces are Unstable but evolve additively (`subsystem-cli-api.md`); the grim surfaces are frozen grim contracts: a mirror change never works around one.
