// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 The Grimoire Authors

//! What the mirror adds to `grim-sdk`: the capability probe for the `mirror`
//! table, the publish environment, the group `publish.toml` and the mapping
//! of a publish item onto the report.

use std::ffi::{OsStr, OsString};
use std::path::Path;

use grim_sdk::{ConvertOutcome as SdkConvert, GrimDriver, PUBLISH_DEADLINE, PublishItem, PublishStatus};
use serde_json::Value;

use crate::model::{Commit, DriverError, Kind, Probe};
use crate::report::{LicenseSource, Reason, Status, Warning};

/// What the registry already holds.
pub trait Published {
    /// `grim describe <reference>`; `Ok(None)` when it is not published (exit 79).
    ///
    /// # Errors
    ///
    /// Any other grim failure.
    fn probe(&self, reference: &str) -> Result<Option<Probe>, DriverError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    /// Output of `grim --version`.
    pub grim_version: String,
}

/// `grim convert` outcome with the report already reduced to the mirror's warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvertOutcome {
    Converted {
        warnings: Vec<Warning>,
    },
    /// grim exited 65; the message is grim's.
    Refused(String),
}

impl Published for GrimDriver {
    fn probe(&self, reference: &str) -> Result<Option<Probe>, DriverError> {
        Ok(self.describe(reference)?.map(|r| r.probe))
    }
}

/// True when the package-entry definition of the publish schema has a `mirror` property.
fn defines_mirror(schema: &Value) -> bool {
    ["$defs", "definitions"]
        .iter()
        .filter_map(|k| schema.get(k)?.get("PublishEntrySpec"))
        .any(|d| d.pointer("/properties/mirror").is_some())
}

/// Probes `grim --version` and the commands the mirror needs;
/// `needs_convert` is true when any upstream has `dialect = "claude"`.
///
/// # Errors
///
/// grim missing or lacking a capability (exit 69).
pub fn capabilities(driver: &GrimDriver, needs_convert: bool) -> Result<Capabilities, DriverError> {
    let missing = |what: &str| DriverError {
        code: Some(69),
        message: format!("grim lacks a capability the mirror needs: {what}; upgrade grim"),
    };
    let grim_version = driver.version()?;
    let schema = match driver.schema("publish") {
        Ok(s) => Some(s),
        // A spawn failure or a deadline is not a missing capability.
        Err(e) if e.code.is_none() || e.code == Some(75) => return Err(e),
        Err(_) => None,
    };
    if !schema.as_ref().is_some_and(defines_mirror) {
        return Err(missing("the publish entry `mirror` table (grim schema --kind publish)"));
    }
    if needs_convert && !driver.has_command("convert")? {
        return Err(missing("`grim convert` (needed for dialect = \"claude\")"));
    }
    Ok(Capabilities { grim_version })
}

/// `grim convert` for one unit; grim infers every `.md` as a rule, so the kind is passed.
///
/// # Errors
///
/// A kind grim cannot convert, or any exit other than 0 or 65.
pub fn convert(driver: &GrimDriver, src: &Path, dest: &Path, kind: Kind) -> Result<ConvertOutcome, DriverError> {
    let kind = match kind {
        Kind::Skill => "skill",
        Kind::Rule => "rule",
        Kind::Agent => "agent",
        other => {
            return Err(DriverError {
                code: None,
                message: format!("grim convert cannot convert a {other:?} unit"),
            });
        }
    };
    Ok(match driver.convert(src, dest, kind)? {
        SdkConvert::Converted { report } => ConvertOutcome::Converted {
            warnings: convert_warnings(&report),
        },
        SdkConvert::Refused(message) => ConvertOutcome::Refused(message),
    })
}

/// `grim publish --format json [--dry-run]` in `group_dir`. Exit 65 without a
/// publish report is `Err` with `code: Some(65)`: the caller reports the
/// group's units `refused` (`grim-refused`).
///
/// # Errors
///
/// Spawn failure, deadline, or output that is not a publish report.
pub fn publish_group(
    driver: &GrimDriver,
    group_dir: &Path,
    dry_run: bool,
    source_date_epoch: i64,
    ceiling: &Path,
) -> Result<Vec<PublishItem>, DriverError> {
    let epoch = OsString::from(source_date_epoch.to_string());
    let env: [(&str, &OsStr); 2] = [
        ("SOURCE_DATE_EPOCH", epoch.as_os_str()),
        ("GIT_CEILING_DIRECTORIES", ceiling.as_os_str()),
    ];
    driver.publish_group(group_dir, dry_run, &env, PUBLISH_DEADLINE)
}

fn convert_warnings(report: &Value) -> Vec<Warning> {
    let text = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or_default().to_owned();
    let mut out = Vec::new();
    for w in report.get("warnings").and_then(Value::as_array).into_iter().flatten() {
        let line = w.get("line").and_then(Value::as_u64).unwrap_or_default();
        out.push(Warning {
            code: "body-token".into(),
            message: format!("{}:{line} {}", text(w, "file"), text(w, "token")),
        });
    }
    for d in report.get("dropped").and_then(Value::as_array).into_iter().flatten() {
        out.push(Warning {
            code: "field-dropped".into(),
            message: format!("{} ({})", text(d, "field"), text(d, "reason")),
        });
    }
    out
}

/// A publish item as the report records it, from what the mirror knows of
/// the version. `recorded` is the re-read of an exact version, consulted for
/// `Skipped` only.
pub fn map_item(
    item: &PublishItem,
    recorded: Option<&Probe>,
    content_commit: &Commit,
    tag_mode: bool,
) -> (Status, Option<Reason>, Option<String>) {
    match item.status {
        PublishStatus::Pushed => (Status::Published, None, None),
        PublishStatus::DryRun => (Status::WouldPublish, None, None),
        PublishStatus::Failed => (Status::Refused, Some(Reason::GrimRefused), item.message.clone()),
        PublishStatus::Skipped => {
            if recorded.and_then(|p| p.revision.as_deref()) == Some(content_commit.as_str()) {
                return (Status::Unchanged, None, None);
            }
            let reason = if tag_mode {
                Reason::UpstreamMoved
            } else {
                Reason::VersionCollision
            };
            let msg = format!("{} is already published at a different revision", item.version);
            (Status::Refused, Some(reason), Some(msg))
        }
    }
}

/// True when a dry run of an `unchanged` unit would pack differently from
/// the published tag. A dry run without a digest cannot be judged.
pub fn output_drift(dry_run_item: &PublishItem, published: &Probe) -> bool {
    dry_run_item.digest.as_deref().is_some_and(|d| d != published.digest)
}

/// Publish-group identity: one `publish.toml` and one grim run per key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GroupKey {
    pub upstream: String,
    pub plugin: Option<String>,
    pub version: String,
    pub content_commit: Commit,
}

/// One entry of a group's `publish.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupEntry {
    pub kind: Kind,
    pub name: String,
    pub version: String,
    /// Path of the staged unit relative to the group directory.
    pub path: String,
    /// Target repository without the registry host.
    pub repository: String,
    pub source: String,
    pub commit: Commit,
    /// `mirror.path`: where the unit sits in the upstream repository.
    pub mirror_path: String,
    pub license: String,
    pub license_source: LicenseSource,
    pub deprecated: Option<String>,
    pub replaced_by: Option<String>,
}

fn table_name(kind: Kind) -> Option<&'static str> {
    Some(match kind {
        Kind::Skill => "skills",
        Kind::Rule => "rules",
        Kind::Agent => "agents",
        Kind::Bundle => "bundles",
        Kind::Mcp => "mcp",
        Kind::Hook => "hooks",
        Kind::Settings => "settings",
        Kind::Command | Kind::Component => return None,
    })
}

fn quote(s: &str) -> String {
    Value::String(s.to_owned()).to_string()
}

fn key(s: &str) -> String {
    if !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
        s.to_owned()
    } else {
        quote(s)
    }
}

/// The group's `publish.toml`.
///
/// # Errors
///
/// An entry whose kind grim cannot publish.
pub fn group_publish_toml(registry_host: &str, entries: &[GroupEntry]) -> Result<String, DriverError> {
    let mut out = format!("registry = {}\n", quote(registry_host));
    for e in entries {
        let table = table_name(e.kind).ok_or_else(|| DriverError {
            code: None,
            message: format!("kind {:?} has no publish table", e.kind),
        })?;
        let head = format!("{table}.{}", key(&e.name));
        let license_source = match e.license_source {
            LicenseSource::Detected => "detected",
            LicenseSource::Declared => "declared",
        };
        out.push_str(&format!(
            "\n[{head}]\nversion = {}\npath = {}\nrepository = {}\n\n[{head}.mirror]\nsource = {}\ncommit = {}\npath = {}\nlicense = {}\nlicense-source = {}\n",
            quote(&e.version),
            quote(&e.path),
            quote(&e.repository),
            quote(&e.source),
            quote(e.commit.as_str()),
            quote(&e.mirror_path),
            quote(&e.license),
            quote(license_source),
        ));
        if let Some(d) = &e.deprecated {
            out.push_str(&format!("deprecated = {}\n", quote(d)));
        }
        if let Some(r) = &e.replaced_by {
            out.push_str(&format!("replaced-by = {}\n", quote(r)));
        }
    }
    Ok(out)
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use super::*;

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
    const SCHEMA_OK: &str = r#"{"$defs":{"PublishEntrySpec":{"properties":{"mirror":{"type":"object"},"path":{}}}}}"#;
    const SCHEMA_OLD: &str = r#"{"$defs":{"PublishEntrySpec":{"properties":{"path":{}}}}}"#;

    /// A fake grim: per subcommand, `<dir>/<sub>.out` is stdout, `<dir>/<sub>.code`
    /// the exit code (default 0), `<dir>/<sub>.sleep` seconds to sleep,
    /// `<dir>/<sub>.codes` a queue of exit codes consumed one per call.
    struct Fake {
        dir: tempfile::TempDir,
    }

    impl Fake {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let script = r#"#!/bin/sh
d="$(dirname "$0")"
sub="$1"
{ echo "== $sub"; echo "args: $*"; echo "cwd: $(pwd)"; echo "sde: ${SOURCE_DATE_EPOCH-unset}"; echo "ceil: ${GIT_CEILING_DIRECTORIES-unset}"; } >> "$d/calls.log"
[ -f "$d/$sub.sleep" ] && exec sleep "$(cat "$d/$sub.sleep")"
code=0
if [ -f "$d/$sub.codes" ]; then
  code="$(head -n1 "$d/$sub.codes")"
  tail -n +2 "$d/$sub.codes" > "$d/$sub.codes.tmp"; mv "$d/$sub.codes.tmp" "$d/$sub.codes"
elif [ -f "$d/$sub.code" ]; then code="$(cat "$d/$sub.code")"; fi
[ -f "$d/$sub.out" ] && cat "$d/$sub.out"
[ -f "$d/$sub.err" ] && cat "$d/$sub.err" >&2
exit "$code"
"#;
            let bin = dir.path().join("grim");
            fs::write(&bin, script).unwrap();
            fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
            let f = Self { dir };
            f.set("--version.out", "grim 0.17.0\n");
            f.set("schema.out", SCHEMA_OK);
            f
        }

        fn set(&self, file: &str, body: &str) {
            fs::write(self.dir.path().join(file), body).unwrap();
        }

        fn driver(&self) -> GrimDriver {
            GrimDriver {
                bin: self.dir.path().join("grim"),
            }
        }

        fn log(&self) -> String {
            fs::read_to_string(self.dir.path().join("calls.log")).unwrap_or_default()
        }
    }

    #[test]
    fn capabilities_report_version_when_everything_is_present() {
        let f = Fake::new();
        f.set("convert.out", "");
        let c = capabilities(&f.driver(), true).unwrap();
        assert_eq!(c.grim_version, "0.17.0");
        assert!(f.log().contains("args: schema --kind publish"));
        assert!(f.log().contains("args: convert --help"));
    }

    #[test]
    fn capabilities_without_mirror_table_exit_69_and_name_it() {
        let f = Fake::new();
        f.set("schema.out", SCHEMA_OLD);
        let e = capabilities(&f.driver(), false).unwrap_err();
        assert_eq!(e.code, Some(69));
        assert!(e.message.contains("mirror"), "{}", e.message);
    }

    #[test]
    fn capabilities_reject_mirror_on_another_definition() {
        let f = Fake::new();
        f.set(
            "schema.out",
            r#"{"$defs":{"Other":{"properties":{"mirror":{}}},"PublishEntrySpec":{"properties":{"path":{}}}}}"#,
        );
        assert_eq!(capabilities(&f.driver(), false).unwrap_err().code, Some(69));
    }

    #[test]
    fn capabilities_without_convert_fail_only_when_needed() {
        let f = Fake::new();
        f.set("convert.code", "64");
        assert!(capabilities(&f.driver(), false).is_ok());
        let e = capabilities(&f.driver(), true).unwrap_err();
        assert_eq!(e.code, Some(69));
        assert!(e.message.contains("convert"), "{}", e.message);
    }

    #[test]
    fn capabilities_with_grim_missing_have_no_code() {
        let d = GrimDriver {
            bin: PathBuf::from("/nonexistent/grim"),
        };
        assert_eq!(capabilities(&d, false).unwrap_err().code, None);
    }

    #[test]
    fn publish_group_hands_grim_the_epoch_and_the_git_ceiling() {
        let f = Fake::new();
        f.set("publish.out", r#"{"items":[]}"#);
        publish_group(&f.driver(), f.dir.path(), true, 1234, Path::new("/ceil")).unwrap();
        let log = f.log();
        assert!(log.contains("args: publish --format json --dry-run"), "{log}");
        assert!(log.contains("sde: 1234") && log.contains("ceil: /ceil"), "{log}");
    }

    #[test]
    fn convert_0_yields_warnings_65_is_refused_other_is_error() {
        let f = Fake::new();
        f.set(
            "convert.out",
            r#"{"warnings":[{"file":"SKILL.md","line":12,"token":"${CLAUDE_PLUGIN_ROOT}"}],"dropped":[{"field":"hooks","reason":"unknown-field"}],"moved":[]}"#,
        );
        let out = convert(&f.driver(), Path::new("/s"), Path::new("/d"), Kind::Skill).unwrap();
        assert_eq!(
            out,
            ConvertOutcome::Converted {
                warnings: vec![
                    Warning {
                        code: "body-token".into(),
                        message: "SKILL.md:12 ${CLAUDE_PLUGIN_ROOT}".into()
                    },
                    Warning {
                        code: "field-dropped".into(),
                        message: "hooks (unknown-field)".into()
                    },
                ]
            }
        );

        let r = Fake::new();
        r.set("convert.code", "65");
        r.set("convert.err", "bad frontmatter\n");
        assert_eq!(
            convert(&r.driver(), Path::new("/s"), Path::new("/d"), Kind::Skill).unwrap(),
            ConvertOutcome::Refused("bad frontmatter".into())
        );

        let o = Fake::new();
        o.set("convert.code", "74");
        assert_eq!(
            convert(&o.driver(), Path::new("/s"), Path::new("/d"), Kind::Skill)
                .unwrap_err()
                .code,
            Some(74)
        );
    }

    #[test]
    fn convert_passes_the_unit_kind_and_refuses_the_rest() {
        for (kind, want) in [(Kind::Agent, "agent"), (Kind::Skill, "skill"), (Kind::Rule, "rule")] {
            let f = Fake::new();
            f.set("convert.out", "{}");
            convert(&f.driver(), Path::new("/s"), Path::new("/d"), kind).unwrap();
            assert!(f.log().contains(&format!("--kind {want}")), "{want}: {}", f.log());
        }
        let f = Fake::new();
        let e = convert(&f.driver(), Path::new("/s"), Path::new("/d"), Kind::Bundle).unwrap_err();
        assert!(e.message.contains("cannot convert a Bundle unit"), "{}", e.message);
        assert!(f.log().is_empty(), "grim must not be spawned: {}", f.log());
    }

    fn pitem(status: PublishStatus, digest: Option<&str>) -> PublishItem {
        PublishItem {
            name: "pdf".into(),
            kind: "skill".into(),
            version: "1.0.0".into(),
            status,
            digest: digest.map(Into::into),
            message: None,
        }
    }

    #[test]
    fn status_mapping_and_skipped_reread() {
        let c = Commit::new(SHA).unwrap();
        let same = Probe {
            digest: "d".into(),
            revision: Some(SHA.into()),
            tags: vec![],
        };
        let other = Probe {
            digest: "d".into(),
            revision: Some("f".repeat(40)),
            tags: vec![],
        };
        let map = |s, p: Option<&Probe>, tag| map_item(&pitem(s, None), p, &c, tag);
        assert_eq!(map(PublishStatus::Pushed, None, true).0, Status::Published);
        assert_eq!(map(PublishStatus::DryRun, None, true).0, Status::WouldPublish);
        assert_eq!(map(PublishStatus::Failed, None, true).1, Some(Reason::GrimRefused));
        assert_eq!(map(PublishStatus::Skipped, Some(&same), true).0, Status::Unchanged);
        assert_eq!(
            map(PublishStatus::Skipped, Some(&other), true).1,
            Some(Reason::UpstreamMoved)
        );
        assert_eq!(
            map(PublishStatus::Skipped, Some(&other), false).1,
            Some(Reason::VersionCollision)
        );
        assert_eq!(
            map(PublishStatus::Skipped, None, false).1,
            Some(Reason::VersionCollision)
        );
    }

    #[test]
    fn output_drift_compares_digests() {
        let p = Probe {
            digest: "sha256:aa".into(),
            revision: None,
            tags: vec![],
        };
        assert!(!output_drift(&pitem(PublishStatus::DryRun, Some("sha256:aa")), &p));
        assert!(output_drift(&pitem(PublishStatus::DryRun, Some("sha256:bb")), &p));
        assert!(!output_drift(&pitem(PublishStatus::DryRun, None), &p));
    }

    fn entry(kind: Kind, name: &str) -> GroupEntry {
        GroupEntry {
            kind,
            name: name.into(),
            version: "0.0.20261120101500".into(),
            path: "skills/pdf".into(),
            repository: "acme/mirror/anthropic-skills/skills/pdf".into(),
            source: "https://github.com/anthropics/skills".into(),
            commit: Commit::new(SHA).unwrap(),
            mirror_path: "skills/pdf".into(),
            license: "Apache-2.0".into(),
            license_source: LicenseSource::Detected,
            deprecated: None,
            replaced_by: None,
        }
    }

    #[test]
    fn group_publish_toml_is_golden_and_parses() {
        let mut second = entry(Kind::Rule, "style \"x\"");
        second.deprecated = Some("gone".into());
        second.replaced_by = Some("ghcr.io/a/b:1".into());
        let toml_text = group_publish_toml("ghcr.io", &[entry(Kind::Skill, "pdf"), second]).unwrap();
        let golden = format!(
            "registry = \"ghcr.io\"\n\n[skills.pdf]\nversion = \"0.0.20261120101500\"\npath = \"skills/pdf\"\nrepository = \"acme/mirror/anthropic-skills/skills/pdf\"\n\n[skills.pdf.mirror]\nsource = \"https://github.com/anthropics/skills\"\ncommit = \"{SHA}\"\npath = \"skills/pdf\"\nlicense = \"Apache-2.0\"\nlicense-source = \"detected\"\n"
        );
        assert!(toml_text.starts_with(&golden), "{toml_text}");
        let parsed: toml::Table = toml_text.parse().unwrap();
        let rule = &parsed["rules"]["style \"x\""]["mirror"];
        assert_eq!(rule["deprecated"].as_str(), Some("gone"));
        assert_eq!(rule["replaced-by"].as_str(), Some("ghcr.io/a/b:1"));
        assert_eq!(
            parsed["skills"]["pdf"]["mirror"]["license-source"].as_str(),
            Some("detected")
        );
    }

    #[test]
    fn group_publish_toml_refuses_kinds_grim_cannot_publish() {
        assert!(group_publish_toml("h", &[entry(Kind::Command, "c")]).is_err());
    }

    #[test]
    fn group_key_orders_by_upstream_plugin_version_commit() {
        let k = |v: &str| GroupKey {
            upstream: "u".into(),
            plugin: None,
            version: v.into(),
            content_commit: Commit::new(SHA).unwrap(),
        };
        assert!(k("1.0.0") < k("1.0.1"));
        assert_ne!(
            k("1.0.0"),
            GroupKey {
                content_commit: Commit::new(&"b".repeat(40)).unwrap(),
                ..k("1.0.0")
            }
        );
    }
}
