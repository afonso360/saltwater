//! Read all the test files in the testsuite and generate a separate test for each one.
//!
//! By generating a separate `#[test]` test for each file, we allow cargo test
//! to automatically run the files in parallel.

use std::env;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = PathBuf::from(
        env::var_os("OUT_DIR").expect("The OUT_DIR environment variable must be set"),
    );
    let mut out = String::new();

    build_tests_for_dir(&mut out, "tests/runner-tests");

    // Write out our auto-generated tests and opportunistically format them with
    // `rustfmt` if it's installed.
    let output = out_dir.join("runtests_tests.rs");
    fs::write(&output, out).unwrap();
    drop(Command::new("rustfmt").arg(&output).status());
}

fn build_tests_for_dir(
    out: &mut String,
    path: impl AsRef<Path>,
) {
    let path = path.as_ref();
    let modname = extract_name(path);

    out.push_str("mod ");
    out.push_str(&modname);
    out.push_str(" {\n");

    let mut dir_entries: Vec<_> = path
        .read_dir()
        .expect(&format!("failed to read {:?}", path))
        .map(|r| r.expect("reading testsuite directory entry"))
        .filter_map(|dir_entry| {
            let p = dir_entry.path();
            if let Some(ext) = p.extension() {
                // Only look at c files.
                if ext != "c" {
                    return None;
                }
            };


            // Ignore files starting with `.`, which could be editor temporary files
            if p.file_stem()?.to_str()?.starts_with(".") {
                return None;
            }
            Some(p)
        })
        .collect();

    dir_entries.sort();

    for entry in dir_entries.iter() {
        if entry.is_dir() {
            build_tests_for_dir(out, entry);
        } else {
            build_test(out, entry);
        }
    }

    out.push_str("}\n");
}


fn build_test(
    out: &mut String,
    path: impl AsRef<Path>,
) {
    let path = path.as_ref();
    let testname = extract_name(path);


    writeln!(out, "#[test]").unwrap();
    writeln!(out, "fn r#{}() {{", &testname).unwrap();
    writeln!(out,"    crate::run_test(r#\"{}\"#).unwrap();", path.display()).unwrap();
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();
}


/// Extract a valid Rust identifier from the stem of a path.
fn extract_name(path: impl AsRef<Path>) -> String {
    let mut name = path.as_ref()
        .file_stem()
        .expect("filename should have a stem")
        .to_str()
        .expect("filename should be representable as a string")
        .replace("-", "_")
        .replace("/", "_")
        .replace("if", "_if");

    let starts_with_digit = name.chars().nth(0).map(|c| c.is_numeric()).unwrap_or(false);
    if starts_with_digit {
        name = format!("_{}", name);
    }

    name
}
