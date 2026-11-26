---
paths:
  - crates/**
---

# CLI Subsystem

Thin CLI shell built on clap derive, in `crates/grim-mirror/src/cli.rs`
(parse, `--fail-on` defaults, entry) and `run.rs` (orchestration). Two
subcommands, `check` and `sync`; every flag is global so it parses before or
after the subcommand.

## Design Rationale

Keep the CLI thin: argument parsing, an environment snapshot (`Env`:
`GRIM_BIN`, `GRIM_OFFLINE`), and delegation to `run.rs`. Domain logic lives in
plain modules (`spec`, `enumerate`, `stage`, `license`, `version`, `retire`)
that are testable without the CLI.

## Surface

| Item | Behaviour |
|---|---|
| `check` | Plan every unit, publish nothing (grim dry run); default `--fail-on refused` |
| `sync` | Publish the planned units; default `--fail-on infra` |
| `--file <path>` | Mirror file, default `./mirror.toml` |
| `--upstream <name>` | Repeatable; restricts the run to named upstream entries |
| `--work-dir <dir>` | Keep clones; a temporary directory otherwise |
| `--fail-on refused\|infra` | Which outcomes make the exit non-zero |
| `--format plain\|json` | Plain table or the run report |

`grim` dispatches `grim mirror …` to this binary on `PATH` and sets `GRIM_BIN`
for it. Offline (`GRIM_OFFLINE` truthy) is refused up front: every step needs
the network.

## Exit Codes

The table is `grim_sdk::ExitCode`, the sysexits set grim uses (0, 1, 64, 65, 69,
74, 75, 77, 78, 79, 80, 81). Clap usage errors map to 64; help and version to 0.
The run's exit is decided in `report::exit_for`: 0, else the first infra code
a grim call returned (`infra_code`), else 69 for any `failed` item, else 65
for a `refused` item under `--fail-on refused`. A new exit code needs the
grim table to carry it first. Design discipline:
`quality-rust-exit_codes.md`.

## Configuration Forwarding

The mirror spawns grim and git. Git runs under a fixed hardened environment
(`subsystem-mirror.md`, invariant 4). `grim` receives the caller's
environment unchanged plus `SOURCE_DATE_EPOCH` and `GIT_CEILING_DIRECTORIES`
for a `publish` call, set by the mirror. The mirror adds no `GRIM_*` key of
its own.

## Quality Gate

During the review-fix loop, run `task rust:verify` — not the full
`task verify`. Full `task verify` is the final gate before commit.

See `subsystem-cli-api.md` for the report contract.
