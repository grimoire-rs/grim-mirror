# hex — swarm memory

Maintained by the hex skills. Small by contract: pointers and
preferences, not copies. Team-shared — commit it.

## Pointers

- Verification: `AGENTS.md` › "Build & Development Commands" — run
  `task verify` (full gate) before any merge; `task rust:verify`,
  `task shell:verify`, `task claude:verify`, `task ci:lint` and
  `task docs:check` are the per-subsystem dev-loop gates; `task test:parallel`
  runs the pytest acceptance suites (needs a registry and a grim binary).
- Plan / ADR / spec conventions: `AGENTS.md` › "Workflow" › Planning flow.
  ADRs `.agents/adr/`, design specs `.agents/specs/`, plans (incl.
  `bugfix_plan_*`, `meta-plan_*`) `.agents/plans/`, research
  `.agents/research/`; one-off records at `.agents/` root. Templates:
  `.claude/templates/artifacts/` — the project's own, not hex's fallbacks.
- Spec home: `.agents/specs/` — the fold-back target; default ID-marker
  heading shape (no project-specific marker declared).
- Plan Status block: every `.agents/plans/plan_*.md` carries one; schema
  and per-skill mutation table in `.claude/rules/meta-ai-config.md` ›
  "Plan Status Protocol". Fast-path pointer `.claude/state/current_plan.md`
  is gitignored and per-worktree.
- Product knowledge: `.claude/rules/product-context.md` (canonical
  identity, related repos, research keywords; indexed from `AGENTS.md` ›
  "Project Identity").
- Rule catalog: `.claude/rules.md` — "By concern" table routes to the rule
  for any given task before a file is open. Read it before planning.
- Key rules: `.claude/rules/arch-principles.md` (boundaries, ADR index);
  `.claude/rules/subsystem-mirror.md` (the crate's invariants) with
  `subsystem-cli.md` and `subsystem-cli-api.md` (co-fire on `crates/**` by
  design); `.claude/rules/arch-threat-model.md` and
  `.claude/rules/quality-security.md` for security-sensitive surfaces.
- Security-sensitive paths: `crates/grim-mirror/src/git.rs`, `enumerate.rs`,
  `stage.rs`, `marketplace.rs` (untrusted upstream content),
  `license.rs` (publish gate), `publish.rs` and `run.rs` (spawn `grim`, push
  bytes to a registry), `.github/workflows/**`. See the `perspectives.always`
  rules below.
- Constitution: `AGENTS.md` › "Core Principles" — eight binding principles.
- Dependency boundary: the crate depends on `grim-sdk` and no other grim
  crate (`subsystem-mirror.md`, invariant 1; pinned by `tests/boundary.rs`).
- Discussions: `.agents/discussions/` (hex default) — `/hex-discuss`
  artifacts; `State: active` there arms the hex-state no-edit rule.
- Worktrees: agent worktrees at the hex default `.agents/worktrees/`
  (gitignored); human feature worktrees are siblings
  `../grim-mirror-wt-<topic>` (`AGENTS.md` › "Workflow"). Whoever creates one
  removes it.

## Preferences

```yaml
# hex config, vocabulary v2. Unknown keys warn once and are ignored.
models:
  fast-balanced: sonnet
  deep-reasoning: opus
  overrides:
    # Mirrors the owner's global model-routing policy: review and
    # non-mechanical implementation run deep at every tier.
    reviewer:security: deep-reasoning
    reviewer:quality: deep-reasoning
    reviewer:spec: deep-reasoning
    builder:implement: deep-reasoning
adversary: codex:rescue

perspectives:
  always:
    # Reads untrusted upstream content and spawns git.
    - role: reviewer:security
      when: "crates/grim-mirror/src/{git,enumerate,stage,marketplace}.rs"
    # The publish gate: what may be republished at all.
    - role: reviewer:security
      when: "crates/grim-mirror/src/license.rs"
    # Spawns grim and pushes bytes to a registry.
    - role: reviewer:security
      when: "crates/grim-mirror/src/{publish,run,retire}.rs"
    # CI holds registry credentials and builds a pinned grim.
    - role: reviewer:security
      when: ".github/workflows/**"
    # The operator contract lives in the docs page and the run report.
    - role: doc-reviewer
      when: "crates/grim-mirror/src/{report,cli,spec}.rs"

research-axes:
  - Claude plugin marketplace formats and their evolution
  - git plumbing hardening (ls-tree, cat-file, config and environment pins)
  - SPDX licence matching and licence-text edge cases
  - OCI registry publish semantics (digest stability, reproducible publish)
  - grim CLI and JSON contract changes that the mirror depends on
```

- Security review here means untrusted upstream content, git calls, the
  licence gate and the publish path named under Pointers — not the report
  formatting that merely renders results.
- Compatibility is a review perspective in its own right: the run report,
  the `mirror.toml` grammar and the exit codes are Unstable, but the `grim`
  surfaces the mirror calls are frozen grim contracts; a diff touching either
  is checked against that before anything else, regardless of tier.
- Codex reviews run on Sol 6.1 (owner, 2026-10-22): every cross-model
  adversary call passes the model explicitly — `codex exec -m gpt-6.1-sol …`
  direct, `--model gpt-6.1-sol` on `codex:rescue` — because the global
  codex default (`model` in `~/.codex/config.toml`) is a different model;
  that file's `review_model` covers only codex's native `/review`.
  `nox-review` has no model flag and inherits that global default.

## Memory
