# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 The Grimoire Authors
"""Helpers the mirror acceptance suites share: a project config and published artifacts."""
from __future__ import annotations

import io
import tarfile
from pathlib import Path

from src.registry import LAYER_MEDIA_TYPE, PublishedArtifact, push_artifact


def tar_of(files: dict[str, str | bytes]) -> bytes:
    """Build an uncompressed tar from a ``{path: content}`` mapping.

    Paths are written verbatim and entries are emitted in sorted order so
    the produced bytes (and the resulting manifest digest) are stable.
    """
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w") as tar:  # no compression
        for path in sorted(files):
            content = files[path]
            data = content.encode() if isinstance(content, str) else content
            info = tarfile.TarInfo(name=path)
            info.size = len(data)
            info.mode = 0o644
            tar.addfile(info, io.BytesIO(data))
    return buf.getvalue()


def write_config(
    project_dir: Path,
    skills: dict[str, str] | None = None,
    rules: dict[str, str] | None = None,
    bundles: dict[str, str] | None = None,
    agents: dict[str, str] | None = None,
    hooks: dict[str, str] | None = None,
    options: dict[str, str] | None = None,
    settings: dict[str, str] | None = None,
    experimental_settings: bool | None = None,
    mods: dict[str, str] | None = None,
    experimental_mods: bool | None = None,
    vendors: dict[str, list[str]] | None = None,
) -> Path:
    """Write a ``grimoire.toml`` with the given skill/rule/bundle/agent/hook refs.

    Each value is a fully-qualified ``registry/repo:tag`` (or ``@digest``)
    string, exactly as a user would write it. Returns the config path.

    ``[hooks]`` is emitted **only when non-empty**, and last — matching what
    ``grim`` itself writes (``command::add::write_config``). An always-present
    empty ``[hooks]`` table would change the declaration hash of every existing
    fixture, so the emptiness check is load-bearing, not cosmetic.

    ``options`` writes raw ``key = value`` lines into an ``[options]`` table
    (values are passed through verbatim, so a string needs its own quotes).

    ``settings`` is emitted like ``hooks``: only when non-empty, after
    ``[hooks]``, so a settings-free fixture keeps its declaration hash.
    ``experimental_settings`` writes ``settings = <bool>`` under
    ``[options.experimental]``; ``None`` writes nothing, which is the
    flag-unset state.

    ``mods`` is emitted like ``settings``, after it. ``experimental_mods``
    writes ``mods = <bool>`` under ``[options.experimental]`` beside the
    settings flag. ``vendors`` maps a vendor name to raw ``key = value`` lines
    written under ``[options.vendors.<name>]`` (e.g.
    ``{"opencode": ['plugin_api = "v1"']}``). All three write nothing when
    unused, so existing fixtures keep their bytes.
    """
    lines: list[str] = []
    if bundles:
        lines.append("[bundles]")
        for name, ref in bundles.items():
            lines.append(f'{name} = "{ref}"')
    lines.append("[skills]")
    for name, ref in (skills or {}).items():
        lines.append(f'{name} = "{ref}"')
    lines.append("[rules]")
    for name, ref in (rules or {}).items():
        lines.append(f'{name} = "{ref}"')
    lines.append("[agents]")
    for name, ref in (agents or {}).items():
        lines.append(f'{name} = "{ref}"')
    if hooks:
        lines.append("[hooks]")
        for name, ref in hooks.items():
            lines.append(f'{name} = "{ref}"')
    if settings:
        lines.append("[settings]")
        for name, ref in settings.items():
            lines.append(f'{name} = "{ref}"')
    if mods:
        lines.append("[mods]")
        for name, ref in mods.items():
            lines.append(f'{name} = "{ref}"')
    if options:
        lines.append("[options]")
        for key, value in options.items():
            lines.append(f"{key} = {value}")
    if experimental_settings is not None or experimental_mods is not None:
        lines.append("[options.experimental]")
        if experimental_settings is not None:
            lines.append(f"settings = {str(experimental_settings).lower()}")
        if experimental_mods is not None:
            lines.append(f"mods = {str(experimental_mods).lower()}")
    for vendor, vendor_lines in (vendors or {}).items():
        lines.append(f"[options.vendors.{vendor}]")
        lines.extend(vendor_lines)
    cfg = project_dir / "grimoire.toml"
    cfg.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    return cfg


def make_artifact(
    repo: str,
    kind: str,
    files: dict[str, str | bytes],
    tag: str = "latest",
    annotations: dict[str, str] | None = None,
    layer_media_type: str = LAYER_MEDIA_TYPE,
) -> PublishedArtifact:
    """Build and push a single-layer OCI skill/rule artifact.

    ``files`` is the artifact tree exactly as the ``DefaultMaterializer``
    expects it: a *skill* is a directory tree rooted at ``<name>/`` (e.g.
    ``{"code-review/SKILL.md": "..."}``); a *rule* is a single
    ``<name>.md`` file (e.g. ``{"rust-style.md": "..."}``). The caller
    constructs the keys; this helper only tars + pushes them, with the kind
    carried by the OCI ``artifactType``.

    Returns the published reference incl. the manifest digest so tests can
    assert ``@sha256`` pins and retag for rolling-release scenarios.
    """
    tar_bytes = tar_of(files)
    return push_artifact(
        repo, tag, tar_bytes, kind, annotations, layer_media_type=layer_media_type
    )
