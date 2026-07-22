//! E2E test for `mt _cli`: runs the compiled binary in a subprocess.
//!
//! The only thing this can show that a unit test cannot is that `run` actually
//! writes the dump to stdout — `replace run with Ok(())` is invisible in
//! process, because the side effect is a `println!`. The dump's *contents*
//! (the `schema` key, the `bin` name) are asserted in `cli_dump.rs` against
//! `dump_json()` directly, so they are deliberately not re-asserted here.
#![allow(clippy::unwrap_used)]

use assert_cmd::Command;

#[test]
fn cli_dump_prints_the_json_to_stdout() {
    let output = Command::cargo_bin("mt")
        .unwrap()
        .arg("_cli")
        .arg("--json")
        .output()
        .unwrap();

    assert!(output.status.success());
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("stdout must carry the JSON dump, not nothing");
}
