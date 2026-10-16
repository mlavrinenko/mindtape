//! A counted noun in English: `1 task`, `12 tasks`. Every message that counts
//! something says it through here, so none prints `task(s)`. `just
//! plural-check` refuses a `(s)` in a string literal outside tests.

/// `noun` as `n` of it reads: `task` for one, `tasks` for any other count. A
/// noun ending in a consonant and `y` takes `ies`, as `entry` does; every other
/// noun our messages count takes a plain `s`.
#[must_use]
pub fn plural<N: Copy + PartialEq + From<u8>>(n: N, noun: &str) -> String {
    if n == N::from(1) {
        return noun.to_owned();
    }
    match noun.strip_suffix('y') {
        Some(stem) if stem.ends_with(|c: char| !"aeiou".contains(c)) => format!("{stem}ies"),
        _ => format!("{noun}s"),
    }
}

/// `n` and its noun: `1 task`, `0 tasks`, `12 tasks`.
#[must_use]
pub fn count<N: Copy + std::fmt::Display + PartialEq + From<u8>>(n: N, noun: &str) -> String {
    format!("{n} {}", plural(n, noun))
}

#[cfg(test)]
mod tests {
    use super::{count, plural};

    #[test]
    fn one_takes_the_bare_noun_and_any_other_count_a_plural() {
        assert_eq!(count(1, "task"), "1 task");
        assert_eq!(count(0, "task"), "0 tasks");
        assert_eq!(count(12, "link"), "12 links");
        assert_eq!(plural(1, "slot"), "slot");
        assert_eq!(plural(2, "slot"), "slots");
        assert_eq!(count(1_u32, "edge"), "1 edge");
        // A consonant and `y` takes `ies`; a vowel and `y` a plain `s`.
        assert_eq!(count(2, "entry"), "2 entries");
        assert_eq!(count(1, "entry"), "1 entry");
        assert_eq!(count(2, "key"), "2 keys");
        assert_eq!(plural(2, "y"), "ys");
    }
}
