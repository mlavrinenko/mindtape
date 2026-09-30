//! Hyperlink-output decision: `--hyperlinks` flag and the OSC 8 sequence.
//!
//! Mirrors `render::color`'s shape: a choice enum driving a pure decision
//! function, a process-global one-shot cache, and a render-time helper
//! (`Linker` here, `Painter` there) that a disabled state turns into a no-op.
//! One difference: [`init`] is called from [`crate::util::open`], not `main`,
//! because the decision also consults `[ls] hyperlinks` once a project's
//! config is loaded — `--color` never had a config counterpart to wait on.
//!
//! There is no terminal capability query for OSC 8 support, so `auto` is a
//! heuristic rather than a detection, and the default is `never` rather than
//! `auto`: an unsupported terminal renders the escape payload as visible
//! garbage instead of degrading to nothing the way colour does
//! (`truth/tasks/hyperlink-the-file-column-with-osc-8.typ`).

use std::fmt;
use std::io::IsTerminal;
use std::sync::OnceLock;

/// The user's hyperlink preference, mirroring `--hyperlinks <auto|always|never>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum HyperlinkChoice {
    /// Enable hyperlinks only when stdout is a TTY and `TERM` is not `dumb` —
    /// a heuristic guess, since no capability query exists for OSC 8 support.
    Auto,
    /// Always emit OSC 8 hyperlink sequences.
    Always,
    /// Never emit hyperlink sequences. The default: an unsupported terminal
    /// can render the escape payload as visible garbage, so opting in stays
    /// explicit rather than automatic.
    #[default]
    Never,
}

impl fmt::Display for HyperlinkChoice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => formatter.write_str("auto"),
            Self::Always => formatter.write_str("always"),
            Self::Never => formatter.write_str("never"),
        }
    }
}

impl From<mindtape::HyperlinkChoice> for HyperlinkChoice {
    /// Bridge `[ls] hyperlinks` (`mindtape::HyperlinkChoice`, in the domain
    /// crate, which cannot depend on `clap`) to this CLI-side choice.
    fn from(choice: mindtape::HyperlinkChoice) -> Self {
        match choice {
            mindtape::HyperlinkChoice::Auto => Self::Auto,
            mindtape::HyperlinkChoice::Always => Self::Always,
            mindtape::HyperlinkChoice::Never => Self::Never,
        }
    }
}

/// Resolve whether hyperlink output should be used for stdout.
///
/// Rules (in priority order):
/// 1. `always` → hyperlinks on.
/// 2. `never` → hyperlinks off.
/// 3. `auto` → on iff stdout is a TTY **and** `TERM` is not `dumb`.
///
/// `stdout_is_tty` and `term_is_dumb` are both taken as plain booleans, rather
/// than this function reading `$TERM` itself the way `color_enabled` reads
/// `NO_COLOR`, so the `auto` arm stays deterministic under test regardless of
/// the test process's own ambient `$TERM`.
#[must_use]
pub fn hyperlink_enabled(choice: HyperlinkChoice, stdout_is_tty: bool, term_is_dumb: bool) -> bool {
    match choice {
        HyperlinkChoice::Always => true,
        HyperlinkChoice::Never => false,
        HyperlinkChoice::Auto => stdout_is_tty && !term_is_dumb,
    }
}

/// Returns `true` if the `TERM` environment variable is exactly `dumb`.
#[must_use]
pub fn term_is_dumb() -> bool {
    std::env::var("TERM").is_ok_and(|term| term == "dumb")
}

// --- Process-global hyperlink state ---------------------------------------

static HYPERLINKS_ENABLED: OnceLock<bool> = OnceLock::new();

/// Resolve and cache the hyperlink decision for this process.
///
/// Must be called exactly once, from [`crate::util::open`] (after the
/// project's `[ls] hyperlinks` is known — see the module docs). Subsequent
/// calls read the cached value without re-checking the TTY or environment.
pub fn init(choice: HyperlinkChoice) {
    let stdout_is_tty = std::io::stdout().is_terminal();
    let _ =
        HYPERLINKS_ENABLED.get_or_init(|| hyperlink_enabled(choice, stdout_is_tty, term_is_dumb()));
}

/// Returns whether hyperlink output is active for this process.
///
/// Falls back to `false` if [`init`] was not called (e.g. in tests, or a
/// command that never opens a project).
#[must_use]
pub fn is_hyperlinks_active() -> bool {
    HYPERLINKS_ENABLED.get().copied().unwrap_or(false)
}

/// Wraps a cell's already-rendered (width-fitted, possibly coloured) text in
/// the OSC 8 hyperlink sequence: `ESC ] 8 ; ; <uri> ST <text> ESC ] 8 ; ; ST`.
/// Disabled is a no-op, exactly like [`super::color::Painter`] when its
/// colour is off.
#[derive(Debug, Clone, Copy)]
pub struct Linker {
    enabled: bool,
}

impl Linker {
    /// Create a new linker that emits (or suppresses) OSC 8 sequences.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Wrap `text` as a hyperlink to `uri`. Returns `text` unchanged when
    /// disabled.
    #[must_use]
    pub fn link(self, text: &str, uri: &str) -> String {
        if self.enabled {
            format!("\x1b]8;;{uri}\x1b\\{text}\x1b]8;;\x1b\\")
        } else {
            text.to_owned()
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{HyperlinkChoice, Linker, hyperlink_enabled, init, is_hyperlinks_active};

    #[test]
    fn always_enables_regardless_of_tty_or_term() {
        assert!(hyperlink_enabled(HyperlinkChoice::Always, false, true));
        assert!(hyperlink_enabled(HyperlinkChoice::Always, true, false));
    }

    #[test]
    fn never_disables_regardless_of_tty_or_term() {
        assert!(!hyperlink_enabled(HyperlinkChoice::Never, false, true));
        assert!(!hyperlink_enabled(HyperlinkChoice::Never, true, false));
    }

    #[test]
    fn auto_requires_both_tty_and_non_dumb_term() {
        assert!(hyperlink_enabled(HyperlinkChoice::Auto, true, false));
        assert!(!hyperlink_enabled(HyperlinkChoice::Auto, false, false));
        assert!(!hyperlink_enabled(HyperlinkChoice::Auto, true, true));
        assert!(!hyperlink_enabled(HyperlinkChoice::Auto, false, true));
    }

    /// The `Display` impl writes each choice's slug, as `--help` and the
    /// config-facing name rely on. Replacing the body with `Ok(())` would
    /// render every choice as the empty string.
    #[test]
    fn display_writes_each_choice_slug() {
        assert_eq!(HyperlinkChoice::Auto.to_string(), "auto");
        assert_eq!(HyperlinkChoice::Always.to_string(), "always");
        assert_eq!(HyperlinkChoice::Never.to_string(), "never");
    }

    #[test]
    fn config_choice_maps_to_the_matching_cli_choice() {
        assert_eq!(
            HyperlinkChoice::from(mindtape::HyperlinkChoice::Auto),
            HyperlinkChoice::Auto
        );
        assert_eq!(
            HyperlinkChoice::from(mindtape::HyperlinkChoice::Always),
            HyperlinkChoice::Always
        );
        assert_eq!(
            HyperlinkChoice::from(mindtape::HyperlinkChoice::Never),
            HyperlinkChoice::Never
        );
    }

    // `init` sets a process-global `OnceLock` exactly once. Asserting "unset
    // before init" is only true in a fresh process, so this re-execs the test
    // binary for just this test name rather than sharing global state with
    // every other test in the binary — mirrors `color`'s `init_then_is_color_active`.

    const CHILD: &str = "_MT_HYPERLINKS_CHILD";

    #[test]
    fn init_then_is_hyperlinks_active() {
        if std::env::var(CHILD).is_err() {
            let out = crate::test_util::run_self_test("init_then_is_hyperlinks_active", CHILD);
            crate::test_util::assert_child_ok(&out);
            return;
        }
        assert!(!is_hyperlinks_active(), "must be false before init");
        init(HyperlinkChoice::Always);
        assert!(is_hyperlinks_active(), "must be true after init(Always)");
    }

    #[test]
    fn linker_disabled_returns_plain_text() {
        let linker = Linker::new(false);
        assert_eq!(
            linker.link("truth/tasks/x.typ", "file:///abs/x.typ"),
            "truth/tasks/x.typ"
        );
    }

    #[test]
    fn linker_enabled_wraps_the_exact_osc8_sequence() {
        let linker = Linker::new(true);
        let out = linker.link("truth/tasks/x.typ", "file:///abs/x.typ");
        assert_eq!(
            out,
            "\x1b]8;;file:///abs/x.typ\x1b\\truth/tasks/x.typ\x1b]8;;\x1b\\"
        );
    }
}
