<!-- doc_type: readme -->

# grim-mirror

Republishes skills, rules, agents and bundles from git repositories and
Claude plugin marketplaces into an OCI registry, using `grim publish`. Each
unit is pinned to an upstream commit, checked for a licence, and published
under your namespace with its source recorded.

Status: Unstable. Source only, no releases. The CLI, the `mirror.toml` file
and the run report may change in any minor version.

## Install

```sh
cargo install --locked --git https://github.com/grimoire-rs/grim-mirror --rev <sha> grim-mirror
```

Replace `<sha>` with the commit you reviewed. The tool calls `git` and
[`grim`](https://github.com/grimoire-rs/grimoire), so both must be on `PATH`.

## Use

```sh
grim mirror check              # dry run, publishes nothing
grim mirror sync               # publishes
```

`grim` dispatches `grim mirror` to the `grim-mirror` binary on `PATH`. The
binary also runs standalone as `grim-mirror`. See
[`crates/grim-mirror/examples/mirror.toml`](crates/grim-mirror/examples/mirror.toml)
for the file format and [`docs/mirroring.md`](docs/mirroring.md) for the full
contract: every key, the report, the exit codes, CI use and takedown.

## What it will not do

It never runs upstream code, never publishes a unit without a recognised
licence, and never republishes a moved upstream tag. Hooks, MCP servers,
settings and mods are skipped by policy.

## Develop

```sh
task verify        # format, clippy, structural tests, workflow lint, docs checks, build, tests
```

The acceptance suites need a registry (the harness starts zot) and a `grim`
binary. See [CONTRIBUTING.md](CONTRIBUTING.md). Project context for agents
lives in [AGENTS.md](AGENTS.md).

## License

Apache-2.0. See the `LICENSE` file.
