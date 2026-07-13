//! Glyph mode: choose between emoji, ASCII, or no decorative glyphs.

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
/// When `preferred` is `Emoji` but `stdout_is_tty` is false, the resolver
/// degrades to `Ascii` to keep pipe/file output clean.
#[must_use]
pub fn resolve_glyph_mode(preferred: GlyphMode, stdout_is_tty: bool) -> GlyphMode {
    match preferred {
        GlyphMode::Emoji => {
            if stdout_is_tty {
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
        assert_eq!(
            resolve_glyph_mode(GlyphMode::Ascii, false),
            GlyphMode::Ascii
        );
        assert_eq!(resolve_glyph_mode(GlyphMode::None, false), GlyphMode::None);
    }

    #[test]
    fn emoji_degrades_to_ascii_in_non_tty() {
        assert_eq!(
            resolve_glyph_mode(GlyphMode::Emoji, false),
            GlyphMode::Ascii
        );
    }

    #[test]
    fn emoji_kept_in_tty() {
        assert_eq!(resolve_glyph_mode(GlyphMode::Emoji, true), GlyphMode::Emoji);
    }
}
