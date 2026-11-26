---
name: meta-validate-context
description: Use when auditing the freshness of `.claude/rules/subsystem-*.md` files against current codebase state, or when a subsystem has undergone significant change.
user-invocable: true
argument-hint: "all | subsystem-name"
triggers:
  - "validate context"
  - "audit subsystem rules"
  - "check subsystem rules"
  - "freshness of rules"
---

# Validate Context Rules

Check `.claude/rules/subsystem-*.md` files match current codebase.

## Workflow

For each `subsystem-*.md` rule file:

1. **Read the rule** — Extract type names, module paths, function names, error variants
2. **Grep the codebase** — Verify each reference exists
3. **Check for new additions** — Find new public types/modules not in rule
4. **Report** — List stale references + missing additions

## Subsystem Rules to Check

| Rule | Key References to Verify |
|------|------------------------|
| `subsystem-mirror.md` | Module and constant names (`MAX_BLOB_BYTES`, `GLOBAL_CONFIG`), invariants vs `crates/grim-mirror/src/`, `tests/boundary.rs` |
| `subsystem-cli.md` | Flags in `cli.rs`, `Env`, exit mapping in `report::exit_for` |
| `subsystem-cli-api.md` | `Report`, `Item`, `Status`, `Reason` in `report.rs` |
| `subsystem-tests.md` | Fixture names in conftest.py, test file names, runner methods, helper params |
| `subsystem-ci.md`, `subsystem-taskfiles.md` | Workflow job names, task names, verify phases |

## Verification Commands

```bash
# Check if a type still exists
grep -r "pub struct TypeName" crates/
grep -r "pub enum TypeName" crates/

# Check if a module still exists
ls crates/grim-mirror/src/

# Check for new public types not in the rule
grep -rn "^pub struct\|^pub enum\|^pub trait" crates/ | grep -v test
```

## Output Format

```markdown
## Context Validation Report

### subsystem-cli.md
- OK: [type] still present
- STALE: [type] — renamed to [new_name] or removed
- MISSING: [new_type] — not documented in rule

### subsystem-mirror.md
...
```

## When to Run

- After big refactors
- Before major feature branches
- Part of `/code-check` audits
- Monthly