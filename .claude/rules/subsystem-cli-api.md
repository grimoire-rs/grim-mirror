---
paths:
  - crates/**
---

# CLI API: the Run Report

The report is the mirror's machine contract (`--format json`) and the single
source for the plain output. It lives in `crates/grim-mirror/src/report.rs`
and is built by `run.rs` from operation results. It is an Unstable surface
(`product-context.md`), but additive evolution is the working rule: a consumer
that parses a report should keep parsing the next one.

## Shape

`Report`: `schema`, `mode` (`check|sync`), `grim`, `grim_mirror` (versions),
`upstreams[]`, `items[]`, `summary`.

- `upstreams[]`: `name`, `repo`, `commit`, `status` (`ok|failed`), `error`,
  `warnings[]`.
- `items[]`: `upstream`, `plugin`, `bundle`, `kind`, `name`, `path`, `ref`,
  `version`, `revision`, `status`, `reason`, `message`, `license`
  (`expression`, `source` = `detected|declared`), `members`, `warnings[]`.
- `summary`: one counter per status.

Optional fields are always present, `null` when absent: a consumer must be able
to tell "not applicable" from "older mirror".

## Closed Vocabularies

| Field | Values |
|---|---|
| `status` | `published`, `would-publish`, `unchanged`, `refused`, `skipped`, `failed` |
| `reason` | The kebab-case `Reason` enum (`license-*`, `unsupported-*`, `unsafe-path`, `invalid-name`, `name-collision`, `upstream-moved`, `version-collision`, `bundle-drift`, `empty-bundle`, `convert-failed`, `grim-refused`, `upstream-unavailable`, `kind-unsupported`, `component-unsupported`, `source-unsupported`, `plugin-conflict`, `commit-unreachable`, `ref-unknown`) |

Add a value, never rename or remove one. Bump `schema` only for a shape
change. A new `reason` needs a unit test, an acceptance case and a
`docs/mirroring.md` update in the same change.

## Rules

- **Typed enums over strings** for every bounded set; `#[serde(rename_all = "kebab-case")]`.
- **Report actual results.** Build items from operation return values (digest
  from grim's dry-run, status from the reconcile), never from the mirror file alone.
- **Preserve input order**: upstreams in file order, items in enumeration order.
- **Plain output** is one table rendered from the same `Report`; it never
  carries information the JSON lacks.
- **Strict reads of grim.** `grim-sdk` parses grim's JSON; an unexpected shape
  is exit 69 and is never mapped to `refused` or `failed` by guess.

## Adding a Report Field

1. Add the field to `report.rs` (always-present, `null` default) with a doc comment.
2. Fill it where the operation knows the value; never recompute in the renderer.
3. Unit test in `report.rs`; acceptance case in `test/tests/test_mirror_cli.py`
   (or the suite for the area); update `docs/mirroring.md`.
