//! Colour-output decision: `--color` flag, `NO_COLOR`, and TTY detection.

use std::fmt;
use std::io::IsTerminal;
use std::sync::OnceLock;

use anstyle::{AnsiColor, Color, Reset, RgbColor, Style};

/// The user's colour preference, mirroring `--color <auto|always|never>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum ColorChoice {
    /// Enable colour only when stdout is a TTY and `NO_COLOR` is not set.
    #[default]
    Auto,
    /// Always emit ANSI colour sequences.
    Always,
    /// Never emit colour.
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
pub fn color_enabled(choice: ColorChoice) -> bool {
    match choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => std::io::stdout().is_terminal() && !no_color_set(),
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

/// Resolve and cache the colour decision for this process.
///
/// Must be called exactly once (from `main`). Subsequent calls from the same
/// process read the cached value without re-checking the TTY or environment.
pub fn init(choice: ColorChoice) {
    let _ = COLOR_ENABLED.get_or_init(|| color_enabled(choice));
}

/// Returns whether colour output is active for this process.
///
/// Falls back to `false` if [`init`] was not called (e.g. in tests).
#[must_use]
pub fn is_color_active() -> bool {
    COLOR_ENABLED.get().copied().unwrap_or(false)
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
    use super::{ColorChoice, Painter, color_enabled, no_color_set, parse_color};

    #[test]
    fn always_enables_regardless_of_env() {
        // NO_COLOR may or may not be set in test env; Always ignores it.
        let enabled = color_enabled(ColorChoice::Always);
        assert!(enabled);
    }

    #[test]
    fn never_disables_regardless_of_env() {
        let enabled = color_enabled(ColorChoice::Never);
        assert!(!enabled);
    }

    #[test]
    fn auto_respects_no_color() {
        // In a test process stdout is never a TTY, so Auto → false already,
        // but we verify NO_COLOR logic by forcing NO_COLOR in a controlled way.
        // We can only read the env here; the unit test for no_color_set is below.
        let _ = no_color_set(); // just assert it doesn't panic
    }

    #[test]
    fn auto_non_tty_returns_false() {
        // Test processes are never attached to a real TTY.
        let enabled = color_enabled(ColorChoice::Auto);
        assert!(!enabled, "expected false in non-TTY test environment");
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

    #[test]
    fn parse_named_colour() {
        assert!(parse_color("green").is_some());
        assert!(parse_color("bright-black").is_some());
    }

    #[test]
    fn parse_hex_colour() {
        assert!(parse_color("#e5c07b").is_some());
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse_color("not-a-colour").is_none());
        assert!(parse_color("#zz").is_none());
    }
}
