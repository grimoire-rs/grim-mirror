# Acceptance tests

Black-box pytest suites that run the built `grim-mirror` as an external
process, with a real `grim` as the system it drives, against a private zot
registry. They assert only on stdout, stderr and exit codes.

## Layout

- `conftest.py`: registry lifecycle, the `grim_binary`, `grim_mirror_binary`,
  `grim_home`, `grim`, `grim_mirror` and `unique_repo` fixtures.
- `tests/`: `test_mirror_<area>.py` suites plus the harness meta tests.
  `tests/conftest.py` adds `project_dir` and `grim_at`.
- `src/`: support code (`zot.py`, `registry.py`, `runner.py`, `git_fixture.py`,
  `helpers.py`, `assertions.py`).
- `tools/`: `accept_shard.py`, `accept_floor.py`, `require_all_passed.py`.
- `bin/` (gitignored): the staged `grim` and `grim-mirror` binaries.

## Running

```sh
task test:build                  # stage grim (GRIM_SOURCE_BINARY) and build grim-mirror
task test:quick                  # parallel run, no restaging
task test:parallel               # parallel run, then warn-only floor check
```

`GRIM_COMMAND` and `GRIM_MIRROR_COMMAND` override the binaries under test.
`zot` and `uv` come from `ocx.toml` through `ocx exec`.

## Isolation

Every test gets its own `GRIM_HOME` and `HOME`. The pytest controller starts one
zot on a random loopback port; workers inherit it through
`GRIM_TEST_REGISTRY_HOST`. Each test pushes under its own `unique_repo`. A run
with no reachable registry aborts with exit 4 instead of skipping to green;
`GRIM_ALLOW_NO_REGISTRY` is not a quality gate.

## Limits

`SUITE_FLOOR`, `SKIP_CEILING` and `XFAIL_CEILING` hold the suite size and the
most skips and xfails CI accepts. `task test:floor` enforces them over the
merged shard reports; local runs only warn.

Text I/O under `test/` names its encoding and newline
(`tests/test_text_io_hygiene.py` enforces it).
