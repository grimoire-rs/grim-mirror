"""Tests for tools/scaffold_manifest.py (checker contract C-038 to C-040)."""
import hashlib
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[2]
TOOL = REPO / "tools" / "scaffold_manifest.py"
GOOD = Path(__file__).parent / "fixtures" / "scaffold_manifest" / "good"


def run(*args):
    return subprocess.run([sys.executable, str(TOOL), *args], capture_output=True, text=True)


@pytest.fixture
def tree(tmp_path):
    dst = tmp_path / "repo"
    shutil.copytree(GOOD, dst)
    return dst


def mutate(tree, old, new):
    m = tree / "scaffold-manifest.toml"
    text = m.read_text(encoding="utf-8")
    assert old in text
    m.write_text(text.replace(old, new, 1), encoding="utf-8")


def test_good_fixture_passes(tree):
    r = run("check", "--root", str(tree))
    assert (r.returncode, r.stdout) == (0, "")


def test_keys_derivation(tree):
    r = run("keys", "--root", str(tree))
    assert r.returncode == 0
    assert r.stdout.splitlines() == [
        "agent_config agent_config§1#1",
        "agent_config agent_config§1#2",
        "agent_config agent_config§1.1#1",
        "testing_and_conventions A1.1",
        "testing_and_conventions A1.2",
        "testing_and_conventions testing_and_conventions§A5#1",
    ]


ROW1 = 'key = "agent_config§1#1"\ndisposition = "taken"\ntargets = ["out/copied.txt"]'
ROW2 = 'key = "agent_config§1#2"\ndisposition = "rewritten"\ntargets = ["out/new.txt"]\nreason = "new content"'

DEFECTS = {
    "missing row": ('key = "A1.2"', 'key = "A9.9"', "missing row: testing_and_conventions A1.2"),
    "unknown key": ('key = "A1.2"', 'key = "A9.9"', "unknown key"),
    "missing reason": (ROW2, ROW2.replace('\nreason = "new content"', ""), "missing reason"),
    "missing targets": (ROW1, ROW1.replace('\ntargets = ["out/copied.txt"]', ""), "missing targets"),
    "target absent": ('targets = ["out/new.txt"]\nreason', 'targets = ["out/none.txt"]\nreason', "target does not exist"),
    "invalid disposition": ('disposition = "rewritten"', 'disposition = "bogus"', "invalid disposition"),
    "sha mismatch": ('mode = "copied"\nsha256 = "', 'mode = "copied"\nsha256 = "ff', "sha256 mismatch"),
    "bad commit": ("72d3fce16fa93890f0cddef4a8d7443c980716ab", "72d3fce", "not 40-hex"),
    "duplicate path": ('path = "out/new.txt"', 'path = "out/copied.txt"', "duplicate file path"),
    "duplicate row": ('key = "A1.2"', 'key = "A1.1"', "duplicate row"),
}


@pytest.mark.parametrize("name", DEFECTS)
def test_defect_fails_and_is_named(tree, name):
    old, new, expect = DEFECTS[name]
    mutate(tree, old, new)
    r = run("check", "--root", str(tree))
    assert r.returncode == 1, r.stdout
    assert expect in r.stdout


def test_copied_file_missing(tree):
    (tree / "out" / "copied.txt").unlink()
    r = run("check", "--root", str(tree))
    assert r.returncode == 1 and "target missing" in r.stdout


def test_bundle_frontmatter_rejected(tree):
    (tree / "out" / "new.txt").write_text("---\nlicense: MIT\nrepository: x/y\n---\nbody\n")
    r = run("check", "--root", str(tree))
    assert r.returncode == 1 and "bundle content" in r.stdout


def test_one_frontmatter_key_allowed(tree):
    (tree / "out" / "new.txt").write_text("---\nlicense: MIT\n---\nbody\n")
    assert run("check", "--root", str(tree)).returncode == 0


def test_usage_error_exits_2():
    assert run("bogus").returncode == 2
    assert run("drift", "--checkout", "nopath").returncode == 2


def git(cwd, *args):
    subprocess.run(
        ["git", "-c", "user.name=t", "-c", "user.email=t@example.com", "-C", str(cwd), *args],
        check=True, capture_output=True,
    )


def test_drift_reports_changed_source(tree, tmp_path):
    """S-012: a changed source file is listed, unchanged and untracked are not drift."""
    src = tmp_path / "src"
    (src / "src").mkdir(parents=True)
    git(src, "init", "-q", "-b", "main")
    (src / "src" / "copied.txt").write_bytes((tree / "out" / "copied.txt").read_bytes())
    git(src, "add", "-A")
    git(src, "commit", "-q", "-m", "one")
    arg = f"grimoire-rs/grimoire={src}"
    r = run("drift", "--root", str(tree), "--checkout", arg)
    assert r.returncode == 0
    assert "drift" not in r.stdout and "skipped out/new.txt" in r.stdout

    (src / "src" / "copied.txt").write_text("changed\n")
    git(src, "commit", "-q", "-am", "two")
    r = run("drift", "--root", str(tree), "--checkout", arg)
    assert r.returncode == 0 and "drift out/copied.txt" in r.stdout

    r = run("drift", "--root", str(tree))
    assert "skipped out/copied.txt" in r.stdout and "drift" not in r.stdout


def test_real_manifest():
    """Contract: the repository's own manifest accounts for every inventory row."""
    r = run("check", "--root", str(REPO))
    assert r.returncode == 0, r.stdout
