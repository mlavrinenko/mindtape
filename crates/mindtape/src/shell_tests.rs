#![allow(clippy::unwrap_used)]

use super::{ConfigCommand, run};

fn argv(args: &[&str]) -> ConfigCommand {
    ConfigCommand::Argv(args.iter().map(|arg| (*arg).to_owned()).collect())
}

/// A `run` value is a string or an array of strings, and nothing else.
#[test]
fn a_run_reads_as_a_line_or_an_array() {
    #[derive(serde::Deserialize)]
    struct Holder {
        run: ConfigCommand,
    }
    let read = |text: &str| toml::from_str::<Holder>(text).map(|holder| holder.run);

    assert_eq!(
        read("run = \"just check\"").unwrap(),
        ConfigCommand::Line("just check".to_owned())
    );
    assert_eq!(
        read("run = [\"cargo\", \"test\"]").unwrap(),
        argv(&["cargo", "test"])
    );
    assert!(read("run = 3").is_err());
    assert!(read("run = [\"cargo\", 3]").is_err());
}

/// An array shows in TOML's array syntax, so it reads apart from a line.
#[test]
fn an_array_shows_as_quoted_items() {
    assert_eq!(
        argv(&["cargo", "say \"hi\""]).to_string(),
        r#"["cargo", "say \"hi\""]"#
    );
    assert_eq!(ConfigCommand::Line("a b".to_owned()).to_string(), "a b");
}

/// A filled value lands inside one argument and never splits it.
#[test]
fn map_fills_each_argument_alone() {
    let filled = argv(&["gh", "pr", "{id}"]).map(|arg| arg.replace("{id}", "a b"));
    assert_eq!(filled, argv(&["gh", "pr", "a b"]));
    let line = ConfigCommand::Line("gh {id}".to_owned()).map(|arg| arg.replace("{id}", "7"));
    assert_eq!(line, ConfigCommand::Line("gh 7".to_owned()));
}

#[test]
fn a_blank_line_or_an_array_with_no_program_is_blank() {
    assert!(ConfigCommand::Line("  ".to_owned()).is_blank());
    assert!(argv(&[]).is_blank());
    assert!(argv(&[" ", "x"]).is_blank());
    assert!(!argv(&["x"]).is_blank());
}

/// An array spawns its program with no shell. The program is this test binary
/// itself, which every platform can run: `--list` prints the test names, this
/// one among them, and an argument no shell has read reaches it whole.
#[test]
fn an_array_runs_its_program_directly() {
    let exe = std::env::current_exe().unwrap();
    let exe = exe.to_str().unwrap();
    let dir = std::env::temp_dir();

    let listed = run(&argv(&[exe, "--list"]), &dir, &[]).unwrap();
    assert_eq!(listed.code, Some(0), "{listed:?}");
    assert!(
        listed.stdout.contains("an_array_runs_its_program_directly"),
        "{listed:?}"
    );

    let filtered = run(&argv(&[exe, "--list", "no such test & exit 0"]), &dir, &[]).unwrap();
    assert_eq!(filtered.code, Some(0), "{filtered:?}");
    assert!(!filtered.stdout.contains("an_array_runs"), "{filtered:?}");
}

#[test]
fn an_array_with_no_program_is_refused_before_any_spawn() {
    let err = run(&argv(&[]), &std::env::temp_dir(), &[]).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}
