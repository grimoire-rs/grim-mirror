# AGENTS.md

Single project-context file for **every** AI agent working in this repo —
Claude Code, Codex CLI, and any other `AGENTS.md`-aware tool.

> **Edit this file, never `CLAUDE.md`.** `CLAUDE.md` is a one-line pointer
> that imports this one, so a second copy would only drift. The same goes
> for anything it links: fix the source file, not a restatement of it.

Lines beginning `@` are Claude Code file imports and are expanded into context automatically; other agents should open those paths directly.

## What is grim-mirror

`grim-mirror` republishes skills, rules, agents and bundles from git
repositories and Claude plugin marketplaces into an OCI registry, at a pinned
upstream commit. It is a plugin for [`grim`](https://github.com/grimoire-rs/grimoire),
the Grimoire package manager: `grim mirror` dispatches to the `grim-mirror`
binary on `PATH`. The mirror never links grim's code. It drives the `grim`
binary through its CLI and JSON output, by way of the `grim-sdk` crate.

> **Status: Unstable, source only.** No release archive, no install path.
> The CLI, the `mirror.toml` grammar, the run report and the exit codes are
> an Unstable surface. The `grim` surfaces the mirror calls are frozen grim
> contracts: follow them, never work around them.

## Project Identity

Product vision, positioning, related repos and research keywords →
[`product-context.md`](./.claude/rules/product-context.md). Consult when
reasoning about project direction, scope trade-offs, ADR motivation, or
research framing. Canonical — keep current (update protocol at the bottom
of that file).

## Rule Catalog

Before planning, research, or an architectural decision, scan "By concern"
in the catalog below. Path-glob rules fire only when a matching file is
already open; the catalog covers everything before that.

@.claude/rules.md

## Build & Development Commands

**Task runner**: [`task`](https://taskfile.dev) (Taskfile v3) is the
primary runner. **Always check `task --list` before inventing ad-hoc
commands.** Taskfiles are tree-structured: root (`taskfile.yml`), subsystem
dirs (`test/`, `.claude/`), `taskfiles/*.taskfile.yml` for cross-cutting.

**Key workflows:**
```sh
task                           # fast check (format + clippy + cargo check)
task verify                    # full quality gate (lint, then build + tests)
task --force verify            # bypass caching — run everything
task rust:verify               # Rust-only gate
task shell:verify              # shell-only gate (shellcheck + shfmt)
task claude:tests              # AI config structural tests
task docs:check                # docs-quality checks over README.md and docs/
task ci:lint                   # actionlint over .github/workflows/
task test:parallel             # pytest acceptance suites (needs a registry and a grim binary)
```

**Cargo commands** (for finer control): `cargo check`, `cargo build
--release` (binary `grim-mirror`), `cargo fmt`, `cargo clippy`, `cargo test`.

**Always run `task verify` after implementation is done.** Always run
`cargo fmt` before commit. Subsystem verify tasks (`rust:verify`,
`shell:verify`, `claude:verify`) are AI dev-loop gates — run the subsystem
gate for the code changed; full `task verify` is the final gate before
commit. Conventions →
[subsystem-taskfiles.md](./.claude/rules/subsystem-taskfiles.md).

## Architecture

**Layout**: a cargo workspace with one member, `crates/grim-mirror` (binary
`grim-mirror`). It depends on `crates/grim-sdk` from the grimoire repository
(the one place that spawns `grim`) and on no other grim crate. Rust unit and
integration tests live beside the code (`crates/grim-mirror/src`,
`crates/grim-mirror/tests`); the acceptance suites live under `test/`
(pytest, a real OCI registry, a real `grim`). Rust edition 2024.

**Read the matching subsystem rule before working on code in that area** —
each carries invariants and design decisions not obvious from the code.
Claude Code auto-loads them on path match; other agents open them by hand.

| Path | Subsystem rule |
|---|---|
| `crates/grim-mirror/**` | [subsystem-mirror.md](./.claude/rules/subsystem-mirror.md), [subsystem-cli.md](./.claude/rules/subsystem-cli.md), [subsystem-cli-api.md](./.claude/rules/subsystem-cli-api.md) |
| `test/**` (pytest, fixtures) | [subsystem-tests.md](./.claude/rules/subsystem-tests.md) |
| `.github/workflows/**` | [subsystem-ci.md](./.claude/rules/subsystem-ci.md) |
| `taskfile.yml`, `taskfiles/**` | [subsystem-taskfiles.md](./.claude/rules/subsystem-taskfiles.md) |

Beyond the path map, two rules carry weight on any change:
[`arch-principles.md`](./.claude/rules/arch-principles.md) (design
principles, boundaries, ADR index) and `quality-core.md` (SOLID/DRY/YAGNI,
Block/Warn/Suggest severity tiers, refactoring discipline). Language
quality guidance lives in `.claude/rules/quality-*.md` — load the one
matching the language you are editing (`quality-rust.md`,
`quality-python.md`, `quality-bash.md`), and `quality-security.md`
(attack surfaces, OWASP/STRIDE checklist) before any security review.
[`arch-threat-model.md`](./.claude/rules/arch-threat-model.md) says which
attackers are in scope for upstream git content and the registry.

## Environment Variables

| Variable | Purpose | Default |
|---|---|---|
| `GRIM_BIN` | Absolute path of the `grim` binary the mirror drives. grim sets it for a dispatched plugin (`grim mirror`); a relative value is ignored and `grim` is looked up on `PATH` | (unset) |
| `GRIM_OFFLINE` | Truthy (`1\|true\|yes\|on`): the mirror refuses to run, because every step needs the network | false |
| `GRIM_MIRROR_COMMAND` | Test harness only: command the acceptance suites run as `grim-mirror`; else `test/bin/grim-mirror` built by `task test:build` | (unset) |
| `GRIM_SOURCE_BINARY` | Test harness only: the `grim` binary `task test:build` copies into `test/bin/`; a missing file fails the task, naming the variable | `../.grimoire-split/target/release/grim` |

The mirror reads no other `GRIM_*` variable. `SOURCE_DATE_EPOCH` and
`GIT_CEILING_DIRECTORIES` are set by the mirror for each `grim publish` call,
never read from the caller. Git runs under a fixed hardened environment
(`subsystem-mirror.md`, invariant 4).

## Core Principles

Eight principles distill every rule, skill, and standard in the framework.
Follow them and everything else follows.

### 1. Understand First

Read before write. Grep before create. Never modify code not read — before
changing a function, grep all callers. Check what exists before building
new.

### 2. Prove It Works

Write tests for the use case first. Run them before commit. Every bug fix
gets a regression test. All quality gates must pass.

### 3. Keep It Safe

No secrets in code. Validate all external input. Least privilege
everywhere. Flag vulnerabilities immediately. Upstream git content is
untrusted: the mirror never runs it.

### 4. Keep It Simple

Small functions, single responsibility. No premature abstraction. Delete
dead code. Comments explain *why*, never *what*.

### 5. Don't Repeat Yourself

Check `.claude/skills/` before ad-hoc generation. Follow existing patterns.
Single source of truth for logic. Extract only when duplication is real.

### 6. Ship It

Work on a branch, never main. Commit iteratively. **Never push to remote**
— the human decides when to push. Push triggers CI, real cost.

### 7. Leave a Trail

Planning artifacts go under `./.agents/`. Document architectural
decisions in ADRs. Name things so the next person understands.

### 8. Learn and Adapt

When you get user feedback or corrections, evaluate whether the insight
should persist as an AI config update (rules, skills, agents).

## Tech Stack

Golden-path technology choices — no deviations →
[`product-tech-strategy.md`](./.claude/rules/product-tech-strategy.md).
(A frontmatter-less rule: Claude Code already loads it every session, so it
is linked here rather than `@`-imported a second time.)

## Workflow

**Worktrees**: `grim-mirror` is the primary checkout (`main`). Feature work
happens in ad hoc worktrees — human ones as siblings `../grim-mirror-wt-<topic>`
on a matching `<type>/<topic>` branch (e.g. `../grim-mirror-wt-retire-report` on
`feat/retire-report`), agent-spawned ones under
`.agents/worktrees/<topic>` (gitignored). Created and torn down per task,
not a fixed roster; whoever creates one removes it. `git worktree list`
shows what is currently checked out.

**Commits**: [Conventional Commits](https://www.conventionalcommits.org/)
(`feat:`, `fix:`, `ci:`, `chore:`, …), scopes optional, no `Co-Authored-By`.
**Always sign off** (DCO): `git commit -s`; CI blocks without it (`task git:dco`).
`chore:` for AI settings, skills, agent context, tooling kept out of the changelog.

**Landing a feature**: When a feature is done, run `/finalize` to clean
branch history into a sequence of Conventional Commits ready to
fast-forward onto `main`. Two-phase model (`/commit` during dev,
`/finalize` before landing) →
[workflow-git.md](./.claude/rules/workflow-git.md).

**Planning flow**: ADR → Design Spec → Plan → Implementation. Planning docs
are committed under `./.agents/`: `adr/`, `specs/`, `plans/` (incl.
`bugfix_plan_*`, `meta-plan_*`), `research/`, one-off records at the root.
Templates in `./.claude/templates/artifacts/`. The founding records of this
tool (`.agents/adr/adr_artifact_mirroring.md`,
`.agents/specs/design_artifact_mirroring.md`,
`.agents/plans/plan_artifact_mirroring.md`) came from the grimoire repository with it.

## Skills & Personas

Multi-agent orchestration is the **hex** bundle, installed at project scope from the
committed `grimoire.toml`/`grimoire.lock` (commands and swarm memory listed below), beside
the `nox`, `rust-essentials`, `python-essentials` and `docs-essentials` bundles.

Swarm memory: `.agents/memory/hex.md` (pointers + preferences). Commands: `/hex-init`, `/hex-plan`, `/hex-execute`, `/hex-review`, `/hex-architect`.
Project-local skills in `.claude/skills/` cover solo work and this repo's
own conventions: `/builder`, `/qa-engineer`, `/security-auditor`,
`/code-check`, `/bugfix`, `/rca`, `/fix`, `/outside-pr-review`, `/commit`, `/finalize`, `/next`,
`/meta-maintain-config`, `/meta-validate-context`. Full map → "Skills by
task topic" in [.claude/rules.md](./.claude/rules.md). Check
`.claude/skills/` before ad-hoc generation.

## Starting Work

Every task starts with
[workflow-intent.md](./.claude/rules/workflow-intent.md) — classify work
(feature, bug fix, refactoring), check GitHub for related issues/PRs, then
follow the appropriate workflow. Also:
[workflow-feature.md](./.claude/rules/workflow-feature.md),
[workflow-bugfix.md](./.claude/rules/workflow-bugfix.md),
[workflow-refactor.md](./.claude/rules/workflow-refactor.md).

## Adversarial Review Guidance

When a cross-model reviewer is delegated an adversarial pass (hex's
adversary gate, or a direct hand-off to another model):

1. **Load this file and the subsystem rules** for the touched paths before
   flagging anything — the author was likely working against them.
2. **Challenge design choices**, not style. `cargo fmt` handles
   formatting; question whether the approach is right, what assumptions it
   depends on, and where it fails under real-world conditions.
3. **Do not critique load-bearing conventions** stated here or in
   `product-tech-strategy.md` (Tokio, Rust 2024, OCI-backed storage,
   never-push-to-remote, commit format). They are decisions, not
   invitations.
4. Return findings with **concrete file paths and line numbers**, grouped
   by severity (Block / Warn / Suggest — definitions in
   `.claude/rules/quality-core.md`).

---

# General Agent Safety (applies to any external agent)

## Non-interactive shell commands

`cp`, `mv`, `rm` may be aliased to `-i` on some systems, which hangs an
agent waiting for y/n. Always use non-interactive flags:

```bash
cp -f source dest           # NOT: cp source dest
mv -f source dest           # NOT: mv source dest
rm -f file                  # NOT: rm file
rm -rf directory            # NOT: rm -r directory
```

Also: `scp -o BatchMode=yes`, `ssh -o BatchMode=yes`, `apt-get -y`, `HOMEBREW_NO_AUTO_UPDATE=1 brew …`.

## Session hygiene

- **Never push to remote** — the human decides when to push (CI has real cost).
- All changes must be committed locally on a feature branch; **never commit directly to `main`**.
- Run `task verify` after any implementation change.
