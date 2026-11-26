---
paths:
  - ".github/workflows/**"
  - ".github/actions/**"
  - ".github/dependabot.yml"
---

# Security Standards

Deep-dive reference for security reviews. See Core Principle 3 ("Keep It Safe") in AGENTS.md for essentials.

## Security Checklist

- [ ] No hardcoded secrets or credentials
- [ ] All user input validated + sanitized
- [ ] SQL queries use parameterized statements
- [ ] Auth + authorization properly implemented
- [ ] Sensitive data encrypted at rest + in transit
- [ ] Error messages no expose internal details
- [ ] Dependencies up to date + vuln-free

## OWASP Top 10 2021

| Category | Check For |
|----------|-----------|
| Broken Access Control | Missing authorization checks |
| Cryptographic Failures | Unencrypted sensitive data |
| Injection | SQL, Command, XSS vulnerabilities |
| Insecure Design | Missing threat modeling |
| Security Misconfiguration | Default credentials, debug enabled |
| Vulnerable Components | Outdated/CVE-affected packages |
| Auth Failures | Weak passwords, session issues |
| Integrity Failures | Unsigned updates, untrusted deserialization |
| Logging Failures | Missing audit trails |
| SSRF | Unvalidated URLs in server requests |

## Severity Classification

| Severity | Definition | Action |
|----------|------------|--------|
| Critical | Exploitable vulnerability, data loss risk, high impact | MUST fix before merge |
| High | Exploitable vulnerability, breaking change, moderate impact, major bug | MUST fix before merge |
| Medium | Requires conditions to exploit, performance issue, code smell | SHOULD fix, can negotiate |
| Low | Best practice violation, style, minor improvement | COULD fix, optional |

## CWE References

Reference CWE (Common Weakness Enumeration) IDs for standardized vuln classification. Example: `CWE-89` for SQL Injection, `CWE-798` for hardcoded credentials.

## Dependency Safety

- Warn on deprecated/vulnerable deps
- Audit new deps before adding
- Keep deps updated
- Use automated scanning (cargo-deny, Dependabot)

## Output Guidelines

- Never expose actual secrets in analysis output
- Give specific file locations + line numbers
- Include concrete remediation steps
- Check code AND config files

## grim-mirror Attack Surfaces

Recurring attack surfaces in this codebase. Use as STRIDE scoping checklist for any
grim-mirror audit; attackers and non-goals are fixed by
[`arch-threat-model.md`](./arch-threat-model.md).

### Upstream git content (T1, T2)

- **No execution.** Entries come from `git ls-tree -r -z`, blobs from `git cat-file --batch`.
  No checkout, so no filter driver, hook, attribute or `.gitmodules` action runs. Template
  sources and Git LFS pointers refuse as `unsupported-build-step`; symlinks and submodules as
  `unsupported-entry`.
- **Containment.** Every upstream-derived path is checked (`unsafe-path`) before it is joined:
  `..` segments, absolute paths, drive prefixes, backslashes, NUL and control characters.
- **Caps.** `MAX_BLOB_BYTES` (8 MiB) and `MAX_UNIT_BYTES` (64 MiB) before content is read;
  marketplace and manifest files have plugin-count and string-length caps; the licence
  matcher skips text over `MAX_MATCH_BYTES`. A new read path needs a cap.
- **Git hardening.** `git.rs` sets `GLOBAL_CONFIG` and `ENV` before every verb (no prompts, no
  external protocols, no redirects off the first host, no hooks path, no submodule recursion,
  literal pathspecs, no LFS smudge), with null stdin and a deadline per call.
- **Moved identity.** A moved upstream tag (`upstream-moved`) and a version collision
  (`version-collision`) are refusals, never republish.

### Spawned `grim` and the registry (T3, T4)

- **One spawn point**, `grim-sdk`. `GRIM_BIN` counts only when absolute; otherwise `grim` on
  `PATH`. Arguments are a fixed vector, never a shell string.
- **Strict parse.** describe, publish and schema JSON are parsed against the expected shape;
  anything else is exit 69. A grim failure never becomes a verdict (I4).
- **Credentials.** The mirror holds none. `grim` reads its own login store; nothing secret
  travels in the mirror's argv, environment or report.
- **Work directories.** `--work-dir` names where clones are kept; without it a temporary
  directory is used and removed on exit.

### CI and supply chain (T5)

- Actions pinned by full commit SHA with a version comment; permissions minimal per job;
  workflow inputs reach shell through `env:`, never interpolated into `run:`.
- `cargo deny` (advisories, bans, sources, licences) and hawkeye (headers) gate the build.
- The `grim` used by acceptance is built from a pinned grimoire commit (`GRIM_COMMIT`).

## grim-mirror Audit Checklist

- [ ] No upstream content executed or interpolated into a command line
- [ ] Every upstream path contained before join; every read capped
- [ ] Git call carries the full hardened config and environment
- [ ] Report `status` / `reason` values only added, never renamed
- [ ] No credential in argv, environment, logs, report or error text
- [ ] grim output parsed strictly; unexpected shape is exit 69
- [ ] New dependency audited (`cargo deny check`)
- [ ] Workflow change: SHA pins, minimal permissions, `env:` for inputs
