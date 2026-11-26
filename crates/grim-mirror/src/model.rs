// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 The Grimoire Authors

//! Types shared by every pipeline stage.

use std::fmt;

pub use grim_sdk::{DriverError, Probe};
pub use semver::Version;

use crate::report::Reason;

/// A full git object id of a revision: 40 lowercase hex digits.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Commit(String);

impl Commit {
    /// # Errors
    ///
    /// `s` is not exactly 40 lowercase hex digits.
    pub fn new(s: &str) -> Result<Self, String> {
        if s.len() == 40 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            Ok(Self(s.to_owned()))
        } else {
            Err(format!("not a 40-hex commit id: {s:?}"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An artifact kind as it appears in the report's `kind` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    Skill,
    Rule,
    Agent,
    Bundle,
    Command,
    Hook,
    Mcp,
    Settings,
    Component,
}

/// An upstream tag that passed the eligibility filter, with its peeled commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EligibleTag {
    pub name: String,
    pub version: Version,
    pub commit: Commit,
}

/// Git facts a snapshot version needs, computed by `run.rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotInput {
    /// `(major, minor)` of the highest eligible tag merged into `commit`.
    pub base: Option<(u64, u64)>,
    pub commit: Commit,
    /// `%ct` of `commit`, seconds since the Unix epoch.
    pub commit_time: i64,
}

/// A unit or group the mirror declines to publish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub reason: Reason,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_accepts_only_40_lowercase_hex() {
        assert!(Commit::new(&"a".repeat(40)).is_ok());
        assert!(Commit::new(&"A".repeat(40)).is_err());
        assert!(Commit::new(&"a".repeat(39)).is_err());
        assert!(Commit::new(&"g".repeat(40)).is_err());
    }
}
