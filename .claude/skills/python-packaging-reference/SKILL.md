---
name: python-packaging-reference
description: Checks scripts behind the python-packaging rule, loaded on demand. Use when the python-packaging rule routes you to a checks script.
license: Apache-2.0
metadata:
  keywords: python,pyproject,packaging,uv,reference
  summary: Checks scripts behind the python-packaging rule
user-invocable: false
---

# python-packaging-reference

Checks for the `python-packaging` rule, installed with it. This skill carries no depth files, only the `context.awk` script the rule's verification block runs.

## checks/

Script the rule's verification block runs. Set `CHECKS=.claude/skills/python-packaging-reference/checks` (`$CLAUDE_CONFIG_DIR/skills/python-packaging-reference/checks` when installed globally).
