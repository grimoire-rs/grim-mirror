# Security Policy

## Reporting a vulnerability

Report privately through GitHub's [private vulnerability reporting][pvr] —
the **Report a vulnerability** button on the [Security tab][security-tab].
That keeps the report and the fix in one place and lets us credit you when it
lands.

Please do not open a public issue, discussion, or pull request for a suspected
vulnerability, and do not post it to social media before there is a fix.

grim-mirror is maintained by one person. Expect a first response within about a
week; there is no formal SLA, and inventing one nobody can hold to would be
worse than saying this. If a report goes unanswered for two weeks, feel free to
nudge it.

If you are unsure whether something is a vulnerability, report it anyway.
Over-communication is welcome — a duplicate costs a few minutes, a missed
report costs users.

A weakness in `grim` itself, or in the `grim-sdk` crate, belongs in the
[grimoire repository][grimoire-security] instead.

## Supported versions

There are no releases yet. The tool is Unstable and source only. Fixes land on
`main`; pin the commit you install from and move forward for a fix.

## What grim-mirror guarantees, and what it does not

grim-mirror republishes other people's content under your registry namespace
and credentials, so its trust boundaries are worth stating explicitly. The full
model is in `.claude/rules/arch-threat-model.md`.

**It never runs upstream code.** Entries are read with `git ls-tree` and blobs
with `git cat-file`; there is no checkout, so no filter driver, hook or
attribute of the upstream runs. Symlinks, submodules, Git LFS pointers and
template sources refuse the unit. Every path from upstream is contained before
it is joined, and every read has a size cap.

**It gates on licences, conservatively.** A unit with no recognised licence,
or one the allowlist does not admit, is refused. The gate is a text matcher,
not legal advice.

**It does not verify who wrote the content.** A mirrored artifact is
upstream-authored and operator-vouched: by republishing, you become its
publisher of record. The mirror pins one upstream commit per publish and
records it, which proves what was copied, not who wrote it. It cannot judge
whether the instructions in a skill are wise.

**These are not vulnerabilities in grim-mirror:**

- Malicious but licensed content in an upstream repository you chose to list.
  Choosing the list is your review step.
- A skill that installs successfully but instructs an agent to behave badly.
- A defect in `git`, `grim` or the registry itself.
- Anything that needs commit access to this repository or to your `mirror.toml`.

## In scope

These are the boundaries where a report is a genuine security issue:

- **Upstream code execution.** Any path that makes the mirror run, build,
  check out or interpret content from an upstream repository, including a git
  config, attribute, hook, filter or template.
- **Path escape.** Upstream content that makes the mirror read or write outside
  the staging directory it chose.
- **Credential exposure.** Registry or forge credentials appearing in logs,
  error messages, the run report, or any file the mirror writes; or reaching
  the mirror's argv or environment.
- **Gate bypass.** A unit published without a licence the gate admits, a
  republished version whose content changed (`upstream-moved`,
  `version-collision`), or a refusal that reads as a success.
- **Resource exhaustion.** Upstream or registry content that makes the mirror
  consume unbounded memory, disk or time despite the size caps and deadlines.
- **Supply chain of this repository.** A workflow that runs untrusted input
  with write permissions, or an unpinned action.

## Known limitations

Not vulnerabilities, but worth knowing:

- **Windows is not built or tested.** The mirror suites skip there; an escape
  you can demonstrate on Windows is in scope all the same.
- **Descendant processes are not killed at a deadline.** A call to `git` or
  `grim` that exceeds its deadline is killed, but a grandchild it spawned is not.

<!-- external -->
[pvr]: https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability
[security-tab]: https://github.com/grimoire-rs/grim-mirror/security
[grimoire-security]: https://github.com/grimoire-rs/grimoire/security
