---
paths:
  - "crates/**"
---

# grim-mirror Threat Model

**The trust boundary, stated once.** Every security finding in this project is scoped by this
file. Without a written boundary, reviews churn: the same finding gets raised as a Block by one
reviewer and dismissed as a non-goal by the next, and neither is wrong because nobody said which
attacker we defend against.

Read this **before** filing or triaging a security finding, and before designing anything that
reads upstream content, spawns a process or publishes to a registry. Companion:
[`quality-security.md`](./quality-security.md) (the OWASP/STRIDE checklist and attack surfaces).
This file says *whom* we defend against; that one says *how*. The wider grim trust model
(publisher of record, digest pinning) is the grimoire repository's
`adr_artifact_trust_model.md`; this file states only what the mirror adds.

## What grim-mirror is

A command-line tool, run by an operator or a CI job, that reads **untrusted git content**
(repositories and plugin marketplaces named in `mirror.toml`), runs it through a licence gate and
republishes it as grim artifacts to a registry **under the operator's credentials**. Its risk class
is a supply-chain relay: whatever it publishes carries the operator as publisher of record.

## In scope — attackers we defend against

| # | Attacker | Why it is in scope |
|---|---|---|
| **T1** | **Malicious upstream content.** A repository or marketplace the operator lists, or one an upstream plugin points to, carries hostile files, names, symlinks, submodules, oversized blobs, template sources or crafted git config and attributes. | The mirror reads it with the operator's privileges. It must never execute it, follow it out of its directory, or exhaust memory on it. |
| **T2** | **A mutable upstream identity.** A moved or force-pushed tag, a branch that now points at other bytes. | The mirror pins a commit, refuses a moved tag (`upstream-moved`) and a version collision (`version-collision`). A silent republish would change what consumers pull under an old version. |
| **T3** | **A hostile or compromised registry or `grim` on `PATH`.** Replies the mirror parses (describe, publish, schema) or a binary named `grim` found by lookup. | `GRIM_BIN` is honoured only when absolute; the report and describe JSON are parsed strictly and an unexpected shape is exit 69, never a verdict. |
| **T4** | **Another local unprivileged process or user on a shared machine.** Reading `/proc/<pid>/cmdline`, `/proc/<pid>/environ`, logs or world-readable work directories. | Credentials reach `grim` by its own login store, never through the mirror's argv or environment. |
| **T5** | **A pull request that changes `mirror.toml` or the workflows.** | A mirror file widens what is published; a workflow change widens what CI can do. Both are reviewed as security changes. |

## Out of scope — non-goals

A finding whose only attacker is one of these is a **non-goal**, not an open defect — record it
as such and move on.

| # | Non-goal | Why |
|---|---|---|
| **N1** | **An insider with commit access to this repository or to the operator's `mirror.toml`.** | They already decide what is published and can change the mirror itself. |
| **N2** | **The operator's machine compromised at or above the mirror's privilege.** | A process at that level reads the registry credential directly. |
| **N3** | **The authenticity of upstream authorship.** | A mirrored artifact's publisher of record is the operator; upstream authorship is outside grim's integrity claim. The mirror records the source and commit, it does not vouch for the author. |
| **N4** | **Defects inside the licence texts' legal meaning.** | The gate is a conservative text matcher with a tri-state evidence model; it is not legal advice. A wrong *refusal* is a defect, a wrong *publish* of recognised-disallowed text is Block. |
| **N5** | **Git itself.** | The mirror hardens how it calls git (`subsystem-mirror.md`, invariant 4) but does not defend against a memory-safety bug in git. |

## Invariants

| # | Invariant |
|---|---|
| **I1** | Nothing from upstream is executed: no checkout, no hook, no filter driver, no template engine, no build step. |
| **I2** | Every path derived from upstream content is contained under its destination before it is joined. |
| **I3** | Every upstream read has a size cap checked before the content is read. |
| **I4** | The mirror never turns a grim or registry failure into a verdict: infra failures keep grim's exit code or become 69. |
| **I5** | Credentials never travel in the mirror's argv, environment or report. |
| **I6** | A published version is never republished with different content (`upstream-moved`, `version-collision`). Retirement republishes one patch up, deprecated, and never deletes. |

## How to use this file

1. Name the attacker (T1–T5) a finding needs. No attacker, or only N1–N5: it is a non-goal.
2. Check the invariant it breaks. No invariant broken: Suggest at most.
3. Change to an invariant is a design change: ADR first (`.agents/adr/`).
