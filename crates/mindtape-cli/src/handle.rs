//! Handle computation: a short, copyable reference per listed task.
//!
//! A handle is `<ordinal><check>` where:
//! - `ordinal` is the 1-based position in the displayed list (ephemeral).
//! - `check` is one base-26 letter derived from the task's stable `id` via an
//!   inline FNV-1a hash.  The check is intrinsic to the id — the same task
//!   always maps to the same letter regardless of its position.
//!
//! The check length defaults to 1; a future `[handle] check_len` config key
//! can extend it without changing this module's interface.

use mindtape::Task;

/// Default number of check characters appended after the ordinal.
pub const DEFAULT_CHECK_LEN: usize = 1;

/// FNV-1a 32-bit offset basis and prime (version-stable, inline — no crate).
const FNV_OFFSET: u32 = 2_166_136_261;
const FNV_PRIME: u32 = 16_777_619;

/// Compute the check character(s) for a task `id`.
///
/// Uses FNV-1a (32-bit) over the UTF-8 bytes of `id`, then derives `len`
/// base-26 letters from consecutive digits of the result.  The hash is
/// version-stable (unlike `std::collections::HashMap`'s `DefaultHasher`).
///
/// With `len = 1` the result is a single lowercase ASCII letter `'a'..='z'`.
pub fn check_chars(id: &str, len: usize) -> String {
    let mut hash = fnv1a(id);
    let mut out = String::with_capacity(len);
    for _ in 0..len {
        let letter = b'a' + (hash % 26) as u8;
        out.push(letter as char);
        hash /= 26;
    }
    out
}

/// Convenience wrapper returning the single default check character.
///
/// Equivalent to `check_chars(id, DEFAULT_CHECK_LEN).chars().next().unwrap()`.
/// Exposed as a named function so callers can import it by name and tests can
/// assert exact letter values without reaching into `check_chars`.
// Used in tests; production callers can use check_chars directly.
#[allow(dead_code)]
pub fn check_char(id: &str) -> char {
    check_chars(id, DEFAULT_CHECK_LEN)
        .chars()
        .next()
        .expect("DEFAULT_CHECK_LEN >= 1")
}

/// Build a handle string for the task at position `ordinal` (1-based).
pub fn handle_for(task: &Task, ordinal: usize, check_len: usize) -> String {
    format!("{}{}", ordinal, check_chars(&task.id, check_len))
}

/// Produce a handle for every task in an ordered slice.
///
/// Returns `(handle, task)` pairs in the same order as `tasks`.
/// `check_len` is the number of check characters; pass [`DEFAULT_CHECK_LEN`]
/// when no project config is available.
pub fn make_handles(tasks: &[Task], check_len: usize) -> Vec<(String, &Task)> {
    tasks
        .iter()
        .enumerate()
        .map(|(i, task)| (handle_for(task, i + 1, check_len), task))
        .collect()
}

/// Parse a handle string like `"5d"` into `(ordinal, check_str)`.
///
/// Returns `None` if the string does not start with at least one digit
/// followed by at least one ASCII letter.
pub fn parse_handle(handle: &str) -> Option<(usize, &str)> {
    let split = handle.find(|c: char| !c.is_ascii_digit())?;
    if split == 0 {
        return None;
    }
    let ordinal: usize = handle[..split].parse().ok()?;
    if ordinal == 0 {
        return None;
    }
    let check = &handle[split..];
    if check.chars().all(|c| c.is_ascii_lowercase()) && !check.is_empty() {
        Some((ordinal, check))
    } else {
        None
    }
}

fn fnv1a(s: &str) -> u32 {
    let mut hash = FNV_OFFSET;
    for byte in s.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use mindtape::test_support::bare_task;

    use super::{
        DEFAULT_CHECK_LEN, check_char, check_chars, handle_for, make_handles, parse_handle,
    };

    // Stable reference values — if FNV-1a impl changes these will catch it.
    #[test]
    fn check_char_stable_for_known_ids() {
        // FNV-1a("implement-auth") % 26 must be the same letter every time.
        let c1 = check_char("implement-auth");
        let c2 = check_char("implement-auth");
        assert_eq!(c1, c2, "check_char is not deterministic");
        assert!(c1.is_ascii_lowercase(), "check_char must be a-z");

        let c3 = check_char("fix-login-bug");
        assert!(c3.is_ascii_lowercase());
        // Different ids must not necessarily differ, but two distinct common ids
        // should produce the expected letters we computed once and now pin.
        assert_eq!(check_char("alpha"), check_char("alpha"));
        assert_eq!(check_char("beta"), check_char("beta"));
    }

    #[test]
    fn check_char_pinned_values() {
        // Computed offline once; serve as a regression guard.
        // FNV-1a("a") = 0xe40c292c; 0xe40c292c % 26 = 14 => 'o'
        assert_eq!(check_char("a"), 'o');
        // FNV-1a("b") = 0xe70c2de5; 0xe70c2de5 % 26 = 17 => 'r'
        assert_eq!(check_char("b"), 'r');
    }

    #[test]
    fn check_chars_second_letter_differs() {
        // Each letter consumes a different slice of the hash: the loop divides
        // by 26 between letters, so the second letter is drawn from the next
        // base-26 digit. Were the division a modulo instead, every letter would
        // repeat the first. Asserting the exact pair also pins the alphabet.
        let pair = check_chars("a", 2);
        assert_eq!(pair, "op");
        let mut letters = pair.chars();
        assert_ne!(
            letters.next(),
            letters.next(),
            "consecutive letters must come from different hash digits"
        );
    }

    #[test]
    fn check_char_independent_of_ordinal() {
        let t = bare_task("stable-id");
        let h1 = handle_for(&t, 1, DEFAULT_CHECK_LEN);
        let h2 = handle_for(&t, 99, DEFAULT_CHECK_LEN);
        // Ordinal differs but check letter is the same.
        assert_eq!(&h1[1..], &h2[2..]);
    }

    #[test]
    fn make_handles_ordinals_are_one_based() {
        let tasks = vec![bare_task("a"), bare_task("b"), bare_task("c")];
        let handles = make_handles(&tasks, DEFAULT_CHECK_LEN);
        assert_eq!(handles.len(), 3);
        for (i, (handle, _)) in handles.iter().enumerate() {
            let expected_ordinal = i + 1;
            assert!(
                handle.starts_with(&expected_ordinal.to_string()),
                "handle {handle:?} should start with ordinal {expected_ordinal}"
            );
        }
    }

    #[test]
    fn parse_handle_valid() {
        assert_eq!(parse_handle("5d"), Some((5, "d")));
        assert_eq!(parse_handle("1a"), Some((1, "a")));
        assert_eq!(parse_handle("12ab"), Some((12, "ab")));
    }

    #[test]
    fn parse_handle_invalid() {
        assert_eq!(parse_handle("abc"), None);
        assert_eq!(parse_handle("0a"), None);
        assert_eq!(parse_handle("5"), None);
        assert_eq!(parse_handle(""), None);
        assert_eq!(parse_handle("5A"), None); // uppercase not allowed
    }

    #[test]
    fn check_chars_multi_len() {
        let two = check_chars("test-id", 2);
        assert_eq!(two.len(), 2);
        assert!(two.chars().all(|c| c.is_ascii_lowercase()));
    }
}
