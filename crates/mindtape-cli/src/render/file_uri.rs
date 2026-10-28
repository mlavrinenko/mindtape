//! A path as a `file:` URI, as RFC 8089 spells one, for the OSC 8 links of
//! the `slug#` and `file` cells and the `#:schema` line `mt init` writes.
//!
//! A URI separates with `/` and percent-encodes the rest, whatever the
//! platform, and `--path-separator` never reaches it. On Windows the drive
//! gets a leading `/`, as in `file:///C:/proj/a.typ`, a UNC path's server
//! becomes the host, as in `file://server/share/a.typ`, and a `\\?\` prefix
//! goes.

use std::path::Path;

/// `path` as a `file:` URI.
#[must_use]
pub fn file_uri(path: &Path) -> String {
    format!("file://{}", uri_tail(path))
}

/// What follows `file://` for the absolute `path`: `/home/u/a.typ`,
/// `/C:/proj/a.typ`, or `server/share/a.typ` for a UNC path.
#[must_use]
pub fn uri_tail(path: &Path) -> String {
    tail_of(&path.to_string_lossy(), cfg!(windows))
}

/// The relative `path` as a URI path: `/`-separated and percent-encoded.
#[must_use]
pub fn uri_path(path: &Path) -> String {
    encode(&slashed(&path.to_string_lossy(), cfg!(windows)))
}

/// [`uri_tail`] over a path's text, `windows` saying whether `\` separates
/// and a drive or UNC prefix can lead it.
fn tail_of(text: &str, windows: bool) -> String {
    let text = slashed(text, windows);
    if !windows {
        return encode(&text);
    }
    let text = match (text.strip_prefix("//?/UNC/"), text.strip_prefix("//?/")) {
        (Some(unc), _) => format!("//{unc}"),
        (None, Some(rest)) => rest.to_owned(),
        (None, None) => text,
    };
    if let Some(unc) = text.strip_prefix("//") {
        return encode(unc);
    }
    match drive(&text) {
        Some((letter, rest)) => format!("/{letter}:{}", encode(rest)),
        None => encode(&text),
    }
}

/// `text` with each `\` read as `/` where `windows` says it separates.
fn slashed(text: &str, windows: bool) -> String {
    if windows {
        text.replace('\\', "/")
    } else {
        text.to_owned()
    }
}

/// The drive letter `text` opens with, as in `C:/proj`, and what follows it.
fn drive(text: &str) -> Option<(char, &str)> {
    let mut chars = text.chars();
    let letter = chars.next().filter(char::is_ascii_alphabetic)?;
    chars.next().filter(|c| *c == ':')?;
    Some((letter, chars.as_str()))
}

/// `text` with every byte but an unreserved character or `/` percent-encoded.
fn encode(text: &str) -> String {
    text.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "file_uri_tests.rs"]
mod tests;
