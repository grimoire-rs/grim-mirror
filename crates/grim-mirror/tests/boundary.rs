// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 The Grimoire Authors

//! The mirror reaches grim through `grim-sdk` only: no `grimoire` or hook
//! crate in its tree, and it is never published or distributed.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn dependency_tree_has_no_grimoire_package() {
    let out = Command::new(env!("CARGO"))
        .args([
            "tree",
            "-p",
            "grim-mirror",
            "-e",
            "normal",
            "--prefix",
            "none",
            "--offline",
        ])
        .current_dir(root())
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let tree = String::from_utf8(out.stdout).unwrap();
    let names: Vec<&str> = tree.lines().filter_map(|l| l.split_whitespace().next()).collect();
    assert!(names.contains(&"grim-mirror"), "{tree}");
    assert!(names.contains(&"grim-sdk"), "{tree}");
    for banned in ["grimoire", "grim-hook", "grimoire-hook-core"] {
        assert!(
            !names.contains(&banned),
            "grim-mirror must not depend on {banned}:\n{tree}"
        );
    }
}

#[test]
fn manifest_is_not_published_or_distributed() {
    let manifest: toml::Table = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(manifest["package"]["publish"].as_bool(), Some(false));
    assert_eq!(manifest["package"]["metadata"]["dist"]["dist"].as_bool(), Some(false));
}
