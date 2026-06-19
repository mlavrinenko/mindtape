//! Glyph mode: choose between emoji, ASCII, or no decorative glyphs.

use std::io::IsTerminal;

/// The set of glyph styles the renderer can emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum GlyphMode {
    /// Use emoji (e.g. ✅, 🔨).
    Emoji,
    /// Use ASCII fallback characters (e.g. `[x]`, `WIP`).
    #[default]
    Ascii,
    /// Emit no decorative glyphs.
    None,
}

/// Resolve the glyph mode appropriate for the current output channel.
///
/// When `preferred` is `Emoji` but stdout is not a TTY, the resolver
/// degrades to `Ascii` to keep pipe/file output clean.
#[must_use]
pub fn resolve_glyph_mode(preferred: GlyphMode) -> GlyphMode {
    match preferred {
        GlyphMode::Emoji => {
            if std::io::stdout().is_terminal() {
                GlyphMode::Emoji
            } else {
                GlyphMode::Ascii
            }
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::{GlyphMode, resolve_glyph_mode};

    #[test]
    fn ascii_and_none_pass_through() {
        assert_eq!(resolve_glyph_mode(GlyphMode::Ascii), GlyphMode::Ascii);
        assert_eq!(resolve_glyph_mode(GlyphMode::None), GlyphMode::None);
    }

    #[test]
    fn emoji_degrades_to_ascii_in_non_tty() {
        // Test processes run without a TTY.
        let resolved = resolve_glyph_mode(GlyphMode::Emoji);
        assert_eq!(
            resolved,
            GlyphMode::Ascii,
            "emoji should degrade to ASCII when stdout is not a TTY"
        );
    }
}
