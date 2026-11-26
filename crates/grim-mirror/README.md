<!-- doc_type: readme -->
# grim-mirror

Republishes skills, rules, agents and bundles from git repositories and
Claude plugin marketplaces into an OCI registry, using `grim publish`.

Status: Unstable. Source only, no releases.

## Install

```sh
cargo install --locked --git https://github.com/grimoire-rs/grim-mirror --rev <sha> grim-mirror
```

## Use

```sh
grim mirror check              # dry run, publishes nothing
grim mirror sync               # publishes
```

`grim` dispatches `grim mirror` to the `grim-mirror` binary on `PATH`.
See [`examples/mirror.toml`](examples/mirror.toml) for the file format and
the docs page `mirroring` for the full contract.
