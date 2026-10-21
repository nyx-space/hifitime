//! Reads the version of the root hifitime crate, so that `version()` and the
//! npm package carry it. This crate holds no version of its own.

use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=../Cargo.toml");
    let manifest = fs::read_to_string("../Cargo.toml").expect("read ../Cargo.toml");
    let version = manifest
        .split("\n[")
        .find(|section| section.trim_start_matches('[').starts_with("package]"))
        .and_then(|package| {
            package.lines().find_map(|line| {
                let value = line
                    .strip_prefix("version")?
                    .trim_start()
                    .strip_prefix('=')?;
                Some(value.trim().trim_matches('"').to_string())
            })
        })
        .expect("no [package] version in ../Cargo.toml");
    println!("cargo:rustc-env=HIFITIME_VERSION={version}");
}
