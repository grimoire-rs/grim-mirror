---
paths:
  - test/**
---

# Test Subsystem

Pytest acceptance tests run the compiled `grim-mirror` binary, with a real `grim` and a real OCI registry (zot), at `test/`. Rust unit and integration tests live beside the code under `crates/grim-mirror/`.

## Design Rationale

Pytest (not Rust integration tests) because acceptance tests exercise the real compiled binaries against a real registry — they catch what a fake `grim` script cannot (`grim convert`, `grim publish`, `grim describe` as they really answer). The registry host is resolved once per session in `pytest_configure`, before any test module imports `src.registry`. **A registry is required**: when none answers, the session aborts with exit 4 and a message naming the three ways out (let the suite start zot, point `GRIM_TEST_REGISTRY_HOST` at one, or skip with `GRIM_ALLOW_NO_REGISTRY=1`). A silent skip of every suite would read as green; `test_registry_required.py` pins the abort.

## Structure

| Path | Purpose |
|------|---------|
| `test/conftest.py` | Registry lifecycle (`pytest_configure` / `pytest_unconfigure`), session fixtures `grim_binary`, `grim_mirror_binary`, `registry`; function fixtures `grim_home`, `grim`, `grim_mirror`, `unique_repo`; `GrimMirrorRunner` |
| `test/tests/conftest.py` | Function fixtures local to `tests/`: `project_dir`, `grim_at` |
| `test/src/` | `zot.py` (registry process), `registry.py` (stdlib OCI client), `runner.py` (`GrimRunner`), `git_fixture.py` (local upstream repositories), `helpers.py` (`write_config`, `make_artifact`), `assertions.py` |
| `test/tests/test_mirror_*.py` | The ten mirror suites: bundles, cli, dialect, kinds, license, marketplace, monorepo, retire, staging, versions |
| `test/tests/test_publish_mirror_table.py` | `publish.toml` `mirror` table as grim writes and reads it |
| `test/tests/test_registry_required.py`, `test_accept_floor.py`, `test_accept_shard.py`, `test_text_io_hygiene.py` | Meta suites for the harness itself |
| `test/tools/` | `accept_floor.py`, `accept_shard.py`, `require_all_passed.py` (CI floor, sharding, result check) |
| `test/taskfile.yml` | Task runner: `build`, `default`, `quick`, `parallel`, `shard`, `floor`, `floor:warn` |

`test/SUITE_FLOOR`, `SKIP_CEILING` and `XFAIL_CEILING` are enforced in PR CI (`task test:floor`); an unfiltered `task test:parallel` (so `task verify`) only warns locally through `test:floor:warn`. A suite that stops collecting or starts skipping moves a number a reviewer has to see. All mirror suites skip on Windows (no mirror build there), which is why the floor tracks the Linux case count.

Text I/O in tests names its codec and newline: `test_text_io_hygiene.py` fails on any bare `read_text` / `write_text` / text-mode `open` under `test/` (except `.venv/` and `data/`).

## Key Fixtures

| Fixture | Scope | Purpose |
|---------|-------|---------|
| `grim_binary` | session | The `grim` under test: `$GRIM_COMMAND` if set, else `test/bin/grim(.exe)` |
| `grim_mirror_binary` | session | The mirror under test: `$GRIM_MIRROR_COMMAND` if set, else `test/bin/grim-mirror` |
| `registry` | session | Registry host string (e.g. `"127.0.0.1:41875"`) |
| `grim_home` | function | Isolated `tmp_path/grim-home` used as `GRIM_HOME` |
| `grim` | function | `GrimRunner(grim_binary, grim_home)` |
| `grim_mirror` | function | `GrimMirrorRunner`: runs `grim-mirror` with `GRIM_BIN` set to the grim under test; `run_via_grim` goes through plugin dispatch (`grim mirror …`) |
| `unique_repo` | function | UUID-prefixed repo name, so tests never collide on the shared registry |
| `project_dir`, `grim_at` | function | Empty workspace; factory for a project-scope `GrimRunner` |

## Test Isolation

- **Per-test `GRIM_HOME` and `HOME`**: the runner isolates both, so nothing touches the developer's real home.
- **Local upstreams**: git fixtures are created under `tmp_path`; no test reaches the network except the local registry.
- **UUID repo names**: `unique_repo` prevents collisions on the shared registry.
- **Minimal env**: the runner strips ambient environment; only `PATH`, `HOME`, `GRIM_*`, `XDG_CONFIG_HOME` (+ Windows spawn vars) pass.

## Running Tests

```bash
task test:build        # copy grim from $GRIM_SOURCE_BINARY, build grim-mirror into test/bin/
task test:parallel     # rebuild-aware, pytest -n auto
task test:quick        # skip rebuild
task test:floor        # floor and ceilings, as in CI
task test:shard        # one CI shard

# Single test (binaries must already exist in test/bin/, or set GRIM_COMMAND and GRIM_MIRROR_COMMAND):
cd test && uv run pytest tests/test_mirror_cli.py -v
```

`GRIM_SOURCE_BINARY` defaults to the grim built from the pinned grimoire commit; a missing file fails `test:build`, naming the variable.

## Adding a New Test

1. Add a function to the matching `test/tests/test_mirror_*.py` (or create a new file).
2. Build upstreams with `git_fixture.py`, registry content with `make_artifact()` and `unique_repo`.
3. Assert on the JSON report (`--format json`), not on plain output.
4. A new refusal reason, report field or `mirror.toml` key needs its acceptance case here and a `docs/mirroring.md` update in the same change; the floor moves with the case count.

## Quality Gate

During review-fix loops, run `task test:parallel` (or `task test:quick` if the binaries are already fresh) — not full `task verify`. Acceptance tests only; no need to re-run the Rust gates.

## Conventions

- Symlink checks use `assert_symlink_exists` from `test/src/assertions.py`, never `path.is_symlink()` (Windows junctions).
- zot keeps a re-pushed tag, so a test that needs a fresh digest retags through `retag` in `test/src/registry.py`; a pinned-digest test keeps the manifest tagged (`retag(..., "keep", ...)`) because zot drops an untagged manifest.
