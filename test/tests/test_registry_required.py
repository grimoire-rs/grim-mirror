# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 The Grimoire Authors
"""A run with no reachable registry must abort, never skip to green.

Every registry-backed case would skip and the session would still exit 0, a
green gate over nothing. The nested session below points the suite at a
closed port and must stop at once with exit 4 and the abort message.
"""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

TEST_DIR = Path(__file__).resolve().parent.parent
# Inherited switches that would lift the gate or reshape the nested run.
_DROPPED = {"GRIM_ALLOW_NO_REGISTRY", "_GRIM_REGISTRY_VERIFIED", "GRIM_TEST_REGISTRY_HOST"}
_USAGE_ERROR = 4


def test_an_unreachable_registry_aborts_the_session_with_exit_4() -> None:
    env = {
        key: value
        for key, value in os.environ.items()
        if key not in _DROPPED and not key.startswith(("PYTEST_ADDOPTS", "PYTEST_XDIST_"))
    }
    env["GRIM_TEST_REGISTRY_HOST"] = "127.0.0.1:1"

    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "pytest",
            "-p",
            "no:cacheprovider",
            "--rootdir",
            str(TEST_DIR),
            f"{Path(__file__)}::test_the_trivial_case_the_nested_session_never_reaches",
        ],
        cwd=TEST_DIR,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=60,
    )

    assert result.returncode == _USAGE_ERROR, result.stdout + result.stderr
    assert "no OCI registry answers at 127.0.0.1:1" in result.stderr + result.stdout


def test_the_trivial_case_the_nested_session_never_reaches() -> None:
    assert True
