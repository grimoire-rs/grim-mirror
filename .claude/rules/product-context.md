---
paths:
  - docs/**
  - .agents/plans/**
  - .agents/adr/**
  - .agents/specs/**
  - .agents/research/**
  - .claude/skills/builder/**
  - .claude/skills/code-check/**
  - .claude/skills/qa-engineer/**
  - .claude/skills/security-auditor/**
---

# grim-mirror Product Context

> Republish skills, rules, agents and bundles from git repositories and
> Claude plugin marketplaces as grim artifacts, at a pinned upstream commit,
> through a licence gate, into any OCI registry.

> **Status: Unstable, source only.** No release archive, no install path.
> The CLI, the `mirror.toml` grammar, the run report and the exit codes may
> change. The `grim` surfaces it calls (`describe`, `convert`, `publish`,
> `schema`) are frozen grim contracts. Statements below are maintained
> positioning — flag drift via the Update Protocol at the bottom of this file.

`grim-mirror` is a plugin for [`grim`](https://github.com/grimoire-rs/grimoire),
the package manager for AI-agent configuration. `grim mirror` dispatches to the
`grim-mirror` binary on `PATH`. It lets an operator offer upstream content that
is not published as OCI artifacts, as grim artifacts under the operator's own
registry namespace, without forking it.

This rule is the canonical product identity. Read it when reasoning about
project direction, trade-offs, ADR motivation, research framing, doc
narratives, or positioning.

## The Problem

Useful skills, rules and agents live in git repositories and plugin
marketplaces, not in registries. A team that wants them in its grim catalog has
to copy them by hand, with no pin, no licence check and no update path.

## What it does

- **Pins** every unit to an upstream commit and publishes it under a snapshot
  version; a moved tag is refused, a version collision is refused.
- **Gates** every unit on licence evidence (unit, plugin, repository), recorded
  as `detected` or `declared`.
- **Never runs upstream code**: no checkout, no build step, no hook.
- **Reconciles** instead of forcing: `check` is a dry run, `sync` publishes,
  retirement republishes one patch up, deprecated, and never deletes.
- **Reports** every item as `published`, `would-publish`, `unchanged`,
  `refused`, `skipped` or `failed`, in a stable JSON contract.

## Target Users

- **Primary**: Operators of a grim registry or catalog who curate third-party
  skills, rules and agents for a team
- **Secondary**: CI maintainers who keep a mirror current on a schedule
- **Non-target**: End users who only install artifacts (they use `grim`)

## Product Principles

1. **Safe by construction** — upstream content is data, never code
2. **Reproducible** — same upstream commit, same artifact digest
3. **Refuse loudly** — a unit that cannot be mirrored safely is reported with a
   reason from a closed vocabulary, never silently dropped
4. **grim stays the publisher** — the mirror stages and calls `grim publish`;
   it never writes to a registry itself
5. **Backend-friendly** — JSON report, clean exit codes, composable in CI

## CLI at a Glance

```bash
grim mirror check                  # Plan and report every unit; publishes nothing
grim mirror sync                   # Publish the planned units
grim mirror check --format json    # Machine-readable run report
grim-mirror --file mirror.toml sync --upstream <name>
```

Global flags: `--file`, `--upstream`, `--work-dir`, `--fail-on refused|infra`,
`--format plain|json`. Operator docs: `docs/mirroring.md`.

## Technical Overview

- **Language**: Rust 2024, one binary crate `grim-mirror` in a one-member workspace
- **Grim access**: `grim-sdk` (from the grimoire repository) spawns `grim`; the
  mirror links no other grim crate
- **Git access**: hardened `git ls-tree` and `git cat-file --batch`, never a checkout
- **Testing**: Rust unit and integration tests beside the code; pytest
  acceptance suites under `test/` against a real OCI registry (zot) and a real `grim`

## Related Repositories

| Repo | Relationship |
|---|---|
| `grimoire-rs/grimoire` | The `grim` CLI; owns `grim-sdk`, `grim convert`, plugin dispatch and the `publish.toml` `mirror` table the mirror relies on |
| `grimoire-rs/grim-gui` | Desktop shell for grim; sibling plugin, shares the scaffold and `grim-sdk` |
| `arcana` | The `hex` and `nox` bundles installed here |
| `ocx-sh/lore` | The rust, python and docs rule bundles installed here |

## Research Keywords

`OCI artifactType` · `claude code plugin marketplace.json` · `agentskills.io` ·
`git ls-tree` / `git cat-file --batch` hardening · SPDX licence matching ·
digest pinning and reproducible publish (`SOURCE_DATE_EPOCH`). Each
`.agents/research/research_*.md` ends with its own durable search-term list —
check there before inventing queries.

## Update Protocol

This file is the single source of truth for grim-mirror product identity.
Stale positioning degrades every downstream decision. Keep it current.

**When to update** — any of these trigger an edit in the same commit:

1. The product vision is fleshed out or revised
2. Target user shift (primary / secondary / non-target list change)
3. A product principle is added, dropped, or reworded
4. A scope decision reframes positioning
5. A CLI-level UX change visible to positioning

**Who must check** — any agent working at product level re-reads this file
when work could shift positioning: a researcher after evaluating a
library/tool; an architect after an ADR or design spec; a doc writer after
user-guide edits; a builder or reviewer if implementation exposes a
capability gap or breaks a stated principle.

**Validation** — `/meta-maintain-config refresh` spot-checks this file
against current CLI help, source code, and recent ADRs.
