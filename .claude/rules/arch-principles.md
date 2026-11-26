---
paths:
  - crates/**/*.rs
---

# grim-mirror Architecture Principles

Auto-loads on every Rust file edit under `crates/`. Provides stable
architectural context — the "why" behind design. For dynamic discovery of
current code state, launch an architecture-explorer worker (hex spawns one
during Discover).

## Crate Layout

A **cargo workspace** with one member, `crates/grim-mirror` (binary
`grim-mirror`, source-only, `publish = false`, Unstable). Dependencies on grim
go through `grim-sdk` (git dependency on the grimoire repository): the one
crate that spawns `grim`. The mirror links no other grim crate
(`tests/boundary.rs`). Acceptance tests live under `test/` (pytest).

| Module (`crates/grim-mirror/src/`) | Responsibility |
|---|---|
| `main.rs`, `cli.rs` | Entry, clap parse, usage errors to exit 64 |
| `spec.rs`, `model.rs` | `mirror.toml` grammar, shared types (`Kind`, re-exports of `grim-sdk` `DriverError` and `Probe`) |
| `git.rs` | Hardened git calls (config and environment set before every verb) |
| `enumerate.rs`, `marketplace.rs`, `stage.rs`, `bundle.rs` | Units from a git tree or a plugin marketplace, staged without a checkout |
| `license.rs` | Licence evidence per level, allow-list gate |
| `version.rs`, `retire.rs` | Snapshot versions, reconcile, retirement |
| `publish.rs` | `publish.toml` generation and the mirror side of grim calls (`Published`, `map_item`, `output_drift`) |
| `run.rs`, `report.rs` | Run orchestration, the report contract, exit mapping |

## Design Principles

| Principle | Intent |
|-----------|--------|
| **Never run upstream code** | Read git objects, never check out or execute; every path from upstream is contained before it is joined |
| **Reconcile, never force** | A moved upstream tag or a version collision is refused, never republished |
| **Grim through the CLI only** | One spawn point (`grim-sdk`); the mirror reads JSON, never grim's source |
| **Closed vocabularies** | Item `status` and `reason` are closed sets; add a value, never rename or remove one |
| **Caps before content** | Every upstream-read path has a size cap checked before bytes are read |
| **Domain types over `String`** | Kind, digest, version and reference use dedicated types round-tripping through their canonical string |

## Command Flow

```
CLI (clap parse)
  → Env (GRIM_BIN, GRIM_OFFLINE) and capability probe through grim-sdk
  → spec: read mirror.toml
  → per upstream: enumerate units (git ls-tree) → stage (git cat-file) → licence gate
  → version + reconcile against the registry (grim describe)
  → grim convert / grim publish (dry-run for check)
  → report (plain / JSON) and exit code
```

## ADR Index

Architecture decisions are recorded as `.agents/adr/adr_*.md`. Read
the relevant ADRs before making decisions in the same domain.

| ADR | Decision |
|-----|----------|
| [adr_artifact_mirroring.md](../../.agents/adr/adr_artifact_mirroring.md) | Mirror skills, rules, agents and bundles from git repositories and Claude plugin marketplaces as grim artifacts at a pinned upstream commit, through a separate `grim-mirror` binary that drives grim's CLI; licence-gated, reconcile-never-force, Unstable. Moved here from the grimoire repository with the tool |

## Code Style Conventions

Project-wide conventions enforced by review:

| Convention | Rule | Deviation = Bug |
|------------|------|-----------------|
| **Type names** | Full descriptive names, not abbreviations | Abbreviated type names |
| **Module structure** | One concept per file; named module files, no `mod.rs` | Monolithic files, `mod.rs` files |
| **Internal enum exhaustiveness** | Omit `#[non_exhaustive]` on internal non-error enums so matches stay total; the binary is the only consumer | `#[non_exhaustive]` on a closed internal enum |
| **Domain types over `String`** | A field that names a domain concept uses a dedicated type with a canonical string form | A bare `String` for a digest, version or reference |

## Where Features Land

| Feature type | Location | Notes |
|--------------|----------|-------|
| New refusal reason or report field | `report.rs` (+ the emitting module) | Unit test, acceptance case and `docs/mirroring.md` in the same change |
| New `mirror.toml` key | `spec.rs` | Same three artifacts; keys are additive |
| New upstream-read path | `git.rs` / `enumerate.rs` | Needs a size cap and a containment check |
| New grim call | `grim-sdk` in the grimoire repository | The mirror never spawns grim itself |
| New acceptance test | `test/tests/test_mirror_*.py` | Use fixtures, maintain test isolation |

## Utility Discipline

**Before writing a small helper inside a module, check whether `std`,
`tokio`, or an existing crate-level utility already covers it.** A helper
reinvented in one module is wasted effort and a drift risk. Check `std`
first, then existing utilities, then invent.
