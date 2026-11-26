#!/usr/bin/env python3
"""Check scaffold-manifest.toml against the two inventory files (stdlib only).

Subcommands: check, keys, drift. Exit codes: 0 clean, 1 defects, 2 usage error.

Row keys: a table row whose first cell (backticks and bold stripped) matches
^[AB]\\d+\\.\\d+$ keys on that ID. Every other data row keys on
`<inventory>§<heading number>#<ordinal>`: the number is the leading optional capital letter plus
`\\d+(\\.\\d+)*` of the nearest preceding heading (slugified heading text when
it has none), the ordinal counts every data row of every table under that
heading, 1-based. Tables inside code fences are ignored.
"""
import argparse
import hashlib
import re
import subprocess
import sys
import tomllib
from pathlib import Path

INVENTORIES = {
    "agent_config": ".agents/research/research_repository_split_agent_config.md",
    "testing_and_conventions": ".agents/research/research_repository_split_testing_and_conventions.md",
}
INVENTORY_PATHS = frozenset(INVENTORIES.values())
DISPOSITIONS = {"taken", "rewritten", "not-applicable"}
MODES = {"copied", "rewritten"}
ID_RE = re.compile(r"^[AB]\d+\.\d+$")
HEX40 = re.compile(r"^[0-9a-f]{40}$")
SEP_CELL = re.compile(r"^:?-+:?$")
FILE_KEYS = ("path", "source_repo", "source_path", "source_commit", "mode", "sha256")


def slug(text):
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-") or "top"


def cells(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]


def derive_keys(inventory, text):
    keys = []
    heading, ordinal, in_fence, prev_table = "top", 0, False, False
    for line in text.splitlines():
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
        if in_fence:
            continue
        m = re.match(r"^#{1,6}\s+(.*)$", line)
        if m:
            title = m.group(1).strip()
            n = re.match(r"[A-Z]?\d+(\.\d+)*", title)
            heading, ordinal, prev_table = (n.group(0) if n else slug(title)), 0, False
            continue
        if not line.lstrip().startswith("|"):
            prev_table = False
            continue
        row = cells(line)
        if all(SEP_CELL.match(c) for c in row if c) and any(row):
            continue
        if not prev_table:  # first line of a table is its header
            prev_table = True
            continue
        ordinal += 1
        first = row[0].replace("`", "").replace("*", "").strip()
        keys.append(first if ID_RE.match(first) else f"{inventory}§{heading}#{ordinal}")
    return keys


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def has_bundle_frontmatter(path):
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    if not lines or lines[0].strip() != "---":
        return False
    fm = []
    for ln in lines[1:]:
        if ln.strip() == "---":
            break
        fm.append(ln)
    return any(l.startswith("license:") for l in fm) and any(l.startswith("repository:") for l in fm)


def inventory_keys(root):
    out, problems = {}, []
    for name, rel in INVENTORIES.items():
        p = root / rel
        if not p.is_file():
            problems.append(f"inventory file missing: {rel}")
            continue
        out[name] = derive_keys(name, p.read_text(encoding="utf-8"))
    return out, problems


def check(root):
    manifest = root / "scaffold-manifest.toml"
    if not manifest.is_file():
        return ["scaffold-manifest.toml missing"]
    try:
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as e:
        return [f"scaffold-manifest.toml invalid: {e}"]
    inv, defects = inventory_keys(root)

    seen_paths = set()
    for f in data.get("file", []):
        p = f.get("path", "<no path>")
        for k in FILE_KEYS:
            if k not in f:
                defects.append(f"file {p}: missing field {k}")
        if p in seen_paths:
            defects.append(f"duplicate file path: {p}")
        seen_paths.add(p)
        if f.get("mode") not in MODES:
            defects.append(f"file {p}: invalid mode {f.get('mode')!r}")
        sc = f.get("source_commit", "")
        if sc == "untracked":
            if p not in INVENTORY_PATHS:
                defects.append(f"file {p}: source_commit untracked is valid only for the inventory copies")
        elif not HEX40.match(sc):
            defects.append(f"file {p}: source_commit is not 40-hex: {sc!r}")
        target = root / p
        if not target.is_file():
            defects.append(f"file {p}: target missing")
            continue
        if f.get("mode") == "copied" and sha256(target) != f.get("sha256"):
            defects.append(f"file {p}: sha256 mismatch with target bytes")
        if has_bundle_frontmatter(target):
            defects.append(f"file {p}: bundle content (frontmatter has license and repository), never copy by hand")

    rows, seen = {}, set()
    for r in data.get("row", []):
        pair = (r.get("inventory"), r.get("key"))
        label = f"row {pair[0]} {pair[1]}"
        if pair in seen:
            defects.append(f"duplicate row: {pair[0]} {pair[1]}")
        seen.add(pair)
        rows[pair] = r
        if pair[0] not in INVENTORIES:
            defects.append(f"{label}: unknown inventory")
        elif inv.get(pair[0]) is not None and pair[1] not in inv[pair[0]]:
            defects.append(f"{label}: unknown key")
        disp = r.get("disposition")
        if disp not in DISPOSITIONS:
            defects.append(f"{label}: invalid disposition {disp!r}")
        if disp != "taken" and not r.get("reason"):
            defects.append(f"{label}: missing reason")
        if disp != "not-applicable":
            targets = r.get("targets")
            if not targets:
                defects.append(f"{label}: missing targets")
            for t in targets or []:
                if not (root / t).exists():
                    defects.append(f"{label}: target does not exist: {t}")

    for name, keys in inv.items():
        for k in keys:
            if (name, k) not in rows:
                defects.append(f"missing row: {name} {k}")
    return defects


def drift(data, checkouts):
    for f in data.get("file", []):
        p, repo = f.get("path"), f.get("source_repo")
        if f.get("source_commit") == "untracked" or repo not in checkouts:
            print(f"skipped {p}")
            continue
        r = subprocess.run(["git", "-C", checkouts[repo], "show", f"HEAD:{f['source_path']}"], capture_output=True)
        if r.returncode != 0:
            print(f"drift {p}: source unreadable at HEAD: {f['source_path']}")
        elif hashlib.sha256(r.stdout).hexdigest() != f.get("sha256"):
            print(f"drift {p}: {f['source_path']} changed at HEAD")


def main(argv=None):
    ap = argparse.ArgumentParser(prog="scaffold_manifest")
    ap.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for n in ("check", "keys"):
        sub.add_parser(n).add_argument("--root", type=Path, dest="root2")
    d = sub.add_parser("drift")
    d.add_argument("--root", type=Path, dest="root2")
    d.add_argument("--checkout", action="append", default=[], metavar="REPO=PATH")
    try:
        a = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    root = (a.root2 or a.root).resolve()
    if a.cmd == "check":
        defects = check(root)
        for line in defects:
            print(line)
        return 1 if defects else 0
    if a.cmd == "keys":
        inv, problems = inventory_keys(root)
        for line in problems:
            print(line, file=sys.stderr)
        for name, keys in inv.items():
            for k in keys:
                print(name, k)
        return 1 if problems else 0
    checkouts = {}
    for c in a.checkout:
        if "=" not in c:
            print(f"bad --checkout {c!r}, want REPO=PATH", file=sys.stderr)
            return 2
        k, v = c.split("=", 1)
        checkouts[k] = v
    try:
        data = tomllib.loads((root / "scaffold-manifest.toml").read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as e:
        print(f"cannot read manifest: {e}", file=sys.stderr)
        return 2
    drift(data, checkouts)
    return 0


if __name__ == "__main__":
    sys.exit(main())
