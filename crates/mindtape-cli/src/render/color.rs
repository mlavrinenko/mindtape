//! Colour-output decision: `--color` flag, `NO_COLOR`, and TTY detection.

use std::fmt;
use std::io::IsTerminal;
use std::sync::OnceLock;

use anstyle::{AnsiColor, Color, Reset, RgbColor, Style};

/// The user's colour preference, mirroring `--color <auto|always|never>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum ColorChoice {
    /// Color when the output is a terminal and `NO_COLOR` isn't set.
    #[default]
    Auto,
    /// Always color.
    Always,
    /// Never color.
    Never,
}

impl fmt::Display for ColorChoice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => formatter.write_str("auto"),
            Self::Always => formatter.write_str("always"),
            Self::Never => formatter.write_str("never"),
        }
    }
}

/// Resolve whether colour output should be used for stdout.
///
/// Rules (in priority order):
/// 1. `always` → colour on.
/// 2. `never` → colour off.
/// 3. `auto` → colour on iff stdout is a TTY **and** `NO_COLOR` is unset.
#[must_use]
pub fn color_enabled(choice: ColorChoice, stdout_is_tty: bool) -> bool {
    match choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => stdout_is_tty && !no_color_set(),
    }
}

/// Returns `true` if the `NO_COLOR` environment variable is set (any value).
///
/// See <https://no-color.org/>.
#[must_use]
pub fn no_color_set() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

// --- Process-global colour state -----------------------------------------

static COLOR_ENABLED: OnceLock<bool> = OnceLock::new();
static STDERR_COLOR_ENABLED: OnceLock<bool> = OnceLock::new();

/// Resolve and cache the colour decisions for this process, one for stdout
/// and one for stderr, each against its own stream's TTY.
///
/// Must be called exactly once (from `main`). Subsequent calls from the same
/// process read the cached value without re-checking the TTY or environment.
pub fn init(choice: ColorChoice) {
    let stdout_is_tty = std::io::stdout().is_terminal();
    let _ = COLOR_ENABLED.get_or_init(|| color_enabled(choice, stdout_is_tty));
    let stderr_is_tty = crate::diag::stderr_is_terminal();
    let _ = STDERR_COLOR_ENABLED.get_or_init(|| color_enabled(choice, stderr_is_tty));
}

/// Returns whether colour output is active for this process.
///
/// Falls back to `false` if [`init`] was not called (e.g. in tests).
#[must_use]
pub fn is_color_active() -> bool {
    COLOR_ENABLED.get().copied().unwrap_or(false)
}

/// Returns whether the prefix of a stderr line is coloured. Before [`init`],
/// as when the argv fails to parse, it decides as `--color auto` would.
#[must_use]
pub fn is_stderr_color_active() -> bool {
    *STDERR_COLOR_ENABLED
        .get_or_init(|| color_enabled(ColorChoice::Auto, crate::diag::stderr_is_terminal()))
}

/// A styled string builder backed by `anstyle`.
///
/// When `enabled` is `false` every call is a no-op and no ANSI sequences are
/// emitted.
#[derive(Debug, Clone, Copy)]
pub struct Painter {
    enabled: bool,
}

impl Painter {
    /// Create a new painter that emits (or suppresses) ANSI sequences.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Wrap `text` in ANSI escape codes for the given `style`.
    ///
    /// When the painter is disabled returns `text` unchanged.
    #[must_use]
    pub fn paint(self, text: &str, style: Style) -> String {
        if self.enabled {
            format!("{style}{text}{Reset}")
        } else {
            text.to_owned()
        }
    }

    /// Convenience: paint with a foreground `AnsiColor`.
    #[must_use]
    pub fn fg(self, text: &str, color: AnsiColor) -> String {
        self.paint(text, Style::new().fg_color(Some(Color::Ansi(color))))
    }
}

/// Parse a colour spec — a named ANSI colour or `#RRGGBB` — into a foreground
/// [`Style`]. Returns `None` for an unrecognised name or malformed hex.
#[must_use]
pub fn parse_color(spec: &str) -> Option<Style> {
    let color = match spec.strip_prefix('#') {
        Some(hex) => parse_hex(hex)?,
        None => Color::Ansi(parse_named(spec)?),
    };
    Some(Style::new().fg_color(Some(color)))
}

fn parse_named(name: &str) -> Option<AnsiColor> {
    let color = match name.to_ascii_lowercase().as_str() {
        "black" => AnsiColor::Black,
        "red" => AnsiColor::Red,
        "green" => AnsiColor::Green,
        "yellow" => AnsiColor::Yellow,
        "blue" => AnsiColor::Blue,
        "magenta" => AnsiColor::Magenta,
        "cyan" => AnsiColor::Cyan,
        "white" => AnsiColor::White,
        "bright-black" | "gray" | "grey" => AnsiColor::BrightBlack,
        "bright-red" => AnsiColor::BrightRed,
        "bright-green" => AnsiColor::BrightGreen,
        "bright-yellow" => AnsiColor::BrightYellow,
        "bright-blue" => AnsiColor::BrightBlue,
        "bright-magenta" => AnsiColor::BrightMagenta,
        "bright-cyan" => AnsiColor::BrightCyan,
        "bright-white" => AnsiColor::BrightWhite,
        _ => return None,
    };
    Some(color)
}

fn parse_hex(hex: &str) -> Option<Color> {
    if hex.len() != 6 {
        return None;
    }
    let red = u8::from_str_radix(hex.get(0..2)?, 16).ok()?;
    let green = u8::from_str_radix(hex.get(2..4)?, 16).ok()?;
    let blue = u8::from_str_radix(hex.get(4..6)?, 16).ok()?;
    Some(Color::Rgb(RgbColor(red, green, blue)))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use anstyle::{Color, RgbColor};

    use super::{ColorChoice, Painter, color_enabled, init, is_color_active, parse_color};

    #[test]
    fn always_enables_regardless_of_env() {
        assert!(color_enabled(ColorChoice::Always, false));
        assert!(color_enabled(ColorChoice::Always, true));
    }

    #[test]
    fn never_disables_regardless_of_env() {
        assert!(!color_enabled(ColorChoice::Never, false));
        assert!(!color_enabled(ColorChoice::Never, true));
    }

    // `init` sets a process-global `OnceLock` exactly once. Asserting "unset
    // before init" is only true in a fresh process, so this re-execs the test
    // binary for just this test name rather than sharing global state with
    // every other test in the binary.

    const CHILD: &str = "_MT_COLOR_CHILD";

    #[test]
    fn init_then_is_color_active() {
        if mindtape::test_support::rerun_isolated(CHILD).is_some() {
            return;
        }
        assert!(!is_color_active(), "must be false before init");
        init(ColorChoice::Always);
        assert!(is_color_active(), "must be true after init(Always)");
    }

    #[test]
    fn auto_non_tty_returns_false() {
        assert!(!color_enabled(ColorChoice::Auto, false));
    }

    #[test]
    fn auto_tty_returns_true() {
        assert!(color_enabled(ColorChoice::Auto, true));
    }

    #[test]
    fn painter_disabled_returns_plain_text() {
        let painter = Painter::new(false);
        let out = painter.fg("hello", anstyle::AnsiColor::Red);
        assert_eq!(out, "hello");
    }

    #[test]
    fn painter_enabled_wraps_with_ansi() {
        let painter = Painter::new(true);
        let out = painter.fg("hello", anstyle::AnsiColor::Red);
        assert!(out.contains("hello"));
        assert!(out.contains('\x1b'), "expected ANSI escape in output");
    }

    /// Every name the legend accepts, mapped to the colour it must produce.
    /// Asserting `is_some()` per name (as twelve separate tests used to) never
    /// pinned the mapping: a mutant routing "cyan" to `AnsiColor::Red` survived
    /// all of them, because the result was still `Some`.
    #[test]
    fn parse_named_maps_each_name_to_its_colour() {
        use anstyle::AnsiColor::{
            Black, Blue, BrightBlack, BrightBlue, BrightCyan, BrightGreen, BrightMagenta,
            BrightRed, BrightWhite, BrightYellow, Cyan, Green, Magenta, Red, White, Yellow,
        };
        let cases = [
            ("black", Black),
            ("red", Red),
            ("green", Green),
            ("yellow", Yellow),
            ("blue", Blue),
            ("magenta", Magenta),
            ("cyan", Cyan),
            ("white", White),
            ("bright-black", BrightBlack),
            ("gray", BrightBlack),
            ("grey", BrightBlack),
            ("bright-red", BrightRed),
            ("bright-green", BrightGreen),
            ("bright-yellow", BrightYellow),
            ("bright-blue", BrightBlue),
            ("bright-magenta", BrightMagenta),
            ("bright-cyan", BrightCyan),
            ("bright-white", BrightWhite),
        ];
        for (name, expected) in cases {
            assert_eq!(
                parse_color(name).and_then(anstyle::Style::get_fg_color),
                Some(Color::Ansi(expected)),
                "{name} must parse to {expected:?}"
            );
        }
    }

    /// Names are case-insensitive, and a hex spec maps to its exact RGB triple.
    #[test]
    fn parse_color_accepts_uppercase_and_hex() {
        assert_eq!(
            parse_color("CYAN").and_then(anstyle::Style::get_fg_color),
            Some(Color::Ansi(anstyle::AnsiColor::Cyan))
        );
        assert_eq!(
            parse_color("#e5c07b").and_then(anstyle::Style::get_fg_color),
            Some(Color::Rgb(RgbColor(0xe5, 0xc0, 0x7b)))
        );
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse_color("not-a-colour").is_none());
        assert!(parse_color("#zz").is_none());
    }
}
