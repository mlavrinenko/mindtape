//! Hyperlink-output decision: `--hyperlinks` flag and the OSC 8 sequence.
//!
//! Mirrors `render::color`'s shape: a choice enum driving a pure decision
//! function and a render-time helper (`Linker` here, `Painter` there) that a
//! disabled state turns into a no-op. One difference: [`resolve`] runs per
//! project from [`crate::util::open`], not once in `main`, because the decision
//! also consults `[ls] hyperlinks` — `--color` never had a config counterpart.
//! The result rides in [`super::RenderCtx`].
//!
//! There is no terminal capability query for OSC 8 support, so `auto` is a
//! heuristic rather than a detection, and the default is `never` rather than
//! `auto`: an unsupported terminal renders the escape payload as visible
//! garbage instead of degrading to nothing the way colour does
//! (`truth/tasks/hyperlink-the-file-column-with-osc-8.typ`).

use std::fmt;
use std::io::IsTerminal;

/// The user's hyperlink preference, mirroring `--hyperlinks <auto|always|never>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum HyperlinkChoice {
    /// Links when the output is a terminal and `TERM` isn't `dumb`. Some
    /// terminals show the link codes as garbage.
    Auto,
    /// Always print links.
    Always,
    /// Never print links.
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

/// Resolve the hyperlink decision for one project: its `choice` against the
/// real stdout and `TERM`. The result rides in [`super::RenderCtx`], so each
/// project carries its own.
#[must_use]
pub fn resolve(choice: HyperlinkChoice) -> bool {
    hyperlink_enabled(choice, std::io::stdout().is_terminal(), term_is_dumb())
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
    use super::{HyperlinkChoice, Linker, hyperlink_enabled};

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
