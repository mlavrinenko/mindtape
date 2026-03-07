use std::fs;

fn main() {
    println!("cargo::rerun-if-changed=lib/typst.toml");

    let toml = fs::read_to_string("lib/typst.toml").expect("failed to read lib/typst.toml");

    let version = toml
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("version")
                .and_then(|rest| rest.trim_start().strip_prefix('='))
                .map(|rest| rest.trim().trim_matches('"').to_string())
        })
        .expect("no version field in lib/typst.toml");

    println!("cargo::rustc-env=TYPST_PACKAGE_VERSION={version}");
}
