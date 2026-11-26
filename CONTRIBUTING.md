# Contributing to grim-mirror

## Prerequisites

- **Rust** (edition 2024) — install via [rustup](https://rustup.rs/)
- **[task](https://taskfile.dev)** — primary task runner (`brew install go-task` or see docs)
- **[uv](https://docs.astral.sh/uv/)** — Python toolchain for acceptance tests

## Layout

A cargo workspace with one member:

| Path | Purpose |
|------|---------|
| `crates/grim-mirror/` | The `grim-mirror` binary and its Rust tests |
| `test/` | Python (pytest) black-box acceptance suites |
| `docs/` | Operator documentation (`mirroring.md`) |
| `.claude/`, `.agents/` | AI-assisted development config and planning records |
| `taskfiles/` | Task automation modules |

## Building

```sh
cargo check                  # fast syntax/type check
cargo build                  # debug build
cargo build --release        # release `grim-mirror` binary
```

## Running Tests

**Unit tests:**

```sh
cargo nextest run
```

**Acceptance tests** need a `grim` binary and a registry. Set `GRIM_SOURCE_BINARY`
to a `grim` built from the pinned grimoire commit; the harness starts zot on a free port.

```sh
task test:build        # copy grim, build grim-mirror into test/bin/
task test:quick        # run the pytest suites without rebuilding
task test:parallel     # run tests in parallel with pytest-xdist
```

Acceptance tests live in `test/` and exercise the built `grim-mirror` against a real `grim`.

## Code Style

```sh
cargo fmt              # format (max_width=120, see rustfmt.toml)
cargo clippy --workspace --all-targets
```

Format before every commit. CI enforces both.

## Commit Conventions

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add a retire report
fix: refuse a moved upstream tag
refactor: extract the licence matcher
ci: pin a workflow action by SHA
```

Scopes are optional. cocogitto validates commit messages in CI.

Every commit also needs a `Signed-off-by` line — see [License](#license).
`git commit -s` adds it for you.

## Branch Model

- Branch from `main` — never commit directly to `main`.
- Keep commits atomic and complete — no WIP commits on shared branches.

## Before Submitting

```sh
task verify    # fmt check + clippy + structural tests + workflow lint + docs checks + build + unit tests + acceptance tests
```

All checks must pass before opening a pull request.

## License

grim-mirror is licensed under the [Apache License, Version 2.0](LICENSE), and
contributions are accepted under that same license — inbound matches outbound,
as Apache-2.0 §5 already presumes. Nothing you contribute is relicensed, and
you keep the copyright in your own work.

**There is no CLA.** Instead, sign off your commits under the
[Developer Certificate of Origin](https://developercertificate.org/) — a
one-line statement that you wrote the patch, or otherwise have the right to
submit it under this license:

```sh
git commit -s          # appends: Signed-off-by: Your Name <you@example.com>
```

The name and email must be real, and the sign-off address must match the one
that authored the commit. If you are contributing work owned by an employer,
make sure you have their permission before you sign off.

CI checks this on every pull request. If you forget, `git rebase --signoff
main..HEAD` fixes the whole branch at once. Run it yourself with:

```sh
task git:dco                       # checks main..HEAD
```

The copyright holder named in `LICENSE` is **The Grimoire Authors** — that is
every person with a commit in this repository, as listed by:

```sh
git shortlog -sne
```

No separate contributor list is maintained; git history is the record.
