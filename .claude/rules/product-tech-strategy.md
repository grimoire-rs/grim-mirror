# Tech Strategy - Golden Paths

**SINGLE SOURCE OF TRUTH** for tech choices this project.

## Compliance

1. **Follow This File**: Use tech listed below
2. **No Deviations**: No alternatives unless told
3. **Latest Stable**: Latest stable version unless pinned

## Language Golden Paths

### Rust (Primary)

| Component | Choice |
|-----------|--------|
| Edition | Rust 2024 (toolchain pinned in `rust-toolchain.toml`) |
| Async | Tokio, minimal feature set (`process`, `rt`, `macros`, `time`, `io-util`, `fs`) |
| CLI | clap derive |
| Report and config | serde, serde_json, toml |
| Grim access | `grim-sdk` (git dependency on the grimoire repository), never a path into grim's source |

**Boundary:** the mirror depends on `grim-sdk` and on no other grim crate.
It drives `grim` through the binary only. `tests/boundary.rs` pins this
(`subsystem-mirror.md`, invariant 1).

### Python (Acceptance Tests)

| Component | Choice |
|-----------|--------|
| Runtime | Python 3.13+ |
| Tooling | uv (Manager), Ruff (Linter) |
| Testing | pytest, pytest-xdist |

## Infrastructure

| Component | Choice |
|-----------|--------|
| Registry for tests | zot, started by the test harness on a free port |
| Secrets | GitHub Secrets |
| Tool pins | `ocx.toml` / `ocx.lock` |

## CI/CD

| Component | Choice |
|-----------|--------|
| Platform | GitHub Actions |
| Workflow lint | actionlint (`task ci:lint`) |
| Security | cargo-deny, hawkeye (licence headers) |
