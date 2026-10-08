use super::*;

fn found(stem: &str) -> Resolution {
    Resolution::Found(stem.to_owned())
}

fn ambiguous(stems: &[&str]) -> Resolution {
    Resolution::Ambiguous(stems.iter().map(|s| (*s).to_owned()).collect())
}

/// A hash from the stem's first byte (top bits) and last byte (low bits), so
/// stems with one first byte share all but the last digit of their codes.
fn by_first_byte(stem: &str) -> u64 {
    let bytes = stem.as_bytes();
    let first = bytes.first().copied().unwrap_or(0);
    let last = bytes.last().copied().unwrap_or(0);
    (u64::from(first) << 56) | u64::from(last)
}

#[test]
fn fnv1a_matches_the_published_vectors() {
    assert_eq!(fnv1a_64(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
}

#[test]
fn encoding_is_thirteen_lowercase_crockford_digits() {
    assert_eq!(encode(0), "0000000000000");
    assert_eq!(encode(u64::MAX), "zzzzzzzzzzzzy");
    assert_eq!(encode(0xcbf2_9ce4_8422_2325), "sfs9ss4448hja");
}

#[test]
fn the_first_digit_uses_all_five_bits() {
    assert!(encode(u64::MAX).starts_with('z'));
    assert!(encode(1 << 63).starts_with('g'));
}

#[test]
fn a_known_stem_keeps_its_code() {
    // Computed apart from this module: FNV-1a 64 of the UTF-8 bytes, base32.
    assert_eq!(encode(mix(fnv1a_64(b"fix-login-bug"))), "r7ewe231jcmsw");
}

#[test]
fn a_lone_stem_gets_the_three_character_floor() {
    let hashes = StemHashes::new(["fix-login-bug"]);
    assert_eq!(hashes.hash_of("fix-login-bug"), Some("r7e"));
}

#[test]
fn an_unknown_stem_has_no_hash() {
    assert_eq!(StemHashes::new(["a"]).hash_of("b"), None);
}

#[test]
fn a_shared_prefix_lengthens_only_the_stems_that_share_it() {
    // `a-1` and `a-2` hash to codes that differ only in the last digit.
    let hashes = StemHashes::with_hasher(["a-1", "a-2", "b-1"], by_first_byte);
    let a1 = hashes.hash_of("a-1").unwrap_or_default();
    let a2 = hashes.hash_of("a-2").unwrap_or_default();
    assert_eq!(a1.len(), 13);
    assert_eq!(a2.len(), 13);
    assert_eq!(hashes.hash_of("b-1").map(str::len), Some(3));
}

#[test]
fn a_new_stem_makes_an_old_short_hash_ambiguous_never_another_task() {
    let before = StemHashes::with_hasher(["a-1"], by_first_byte);
    let short = before.hash_of("a-1").unwrap_or_default().to_owned();
    assert_eq!(before.resolve(&short), found("a-1"));
    let after = StemHashes::with_hasher(["a-1", "a-2"], by_first_byte);
    assert_eq!(after.resolve(&short), ambiguous(&["a-1", "a-2"]));
}

#[test]
fn every_hash_resolves_to_its_own_stem() {
    let stems = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta"];
    let hashes = StemHashes::new(stems);
    for stem in stems {
        let hash = hashes.hash_of(stem).unwrap_or_default();
        assert_eq!(hashes.resolve(hash), found(stem), "{stem} via {hash}");
    }
}

#[test]
fn input_below_three_characters_matches_nothing() {
    let hashes = StemHashes::new(["fix-login-bug"]);
    assert_eq!(hashes.resolve("r7"), Resolution::NoMatch);
    assert_eq!(hashes.resolve(""), Resolution::NoMatch);
}

#[test]
fn input_ignores_case_and_a_longer_prefix_still_matches() {
    let hashes = StemHashes::new(["fix-login-bug"]);
    assert_eq!(hashes.resolve("R7E"), found("fix-login-bug"));
    assert_eq!(hashes.resolve("r7ewe231jcmsw"), found("fix-login-bug"));
    assert_eq!(hashes.resolve("r7ewe231jcmsw0"), Resolution::NoMatch);
}

#[test]
fn input_reads_i_l_as_one_and_o_as_zero() {
    let hashes = StemHashes::with_hasher(["x"], |_| 1 << 54);
    let code = encode(1 << 54);
    assert!(code.starts_with("010"), "{code}");
    for input in ["010", "OIO", "oio", "Ol0", "0lO"] {
        assert_eq!(hashes.resolve(input), found("x"), "{input}");
    }
}

#[test]
fn input_outside_the_alphabet_matches_nothing() {
    let hashes = StemHashes::new(["fix-login-bug"]);
    for input in ["r7u", "r7-", "r7 ", "r7e!", "r7\u{e9}"] {
        assert_eq!(hashes.resolve(input), Resolution::NoMatch, "{input}");
    }
}

#[test]
fn an_unmatched_prefix_is_no_match() {
    let hashes = StemHashes::new(["fix-login-bug"]);
    assert_eq!(hashes.resolve("zzz"), Resolution::NoMatch);
}

#[test]
fn equal_full_codes_stay_ambiguous_at_every_length() {
    let hashes = StemHashes::with_hasher(["one", "two", "three"], |s| {
        if s == "three" {
            7
        } else {
            0xffff_0000_0000_0000
        }
    });
    let one = hashes.hash_of("one").unwrap_or_default().to_owned();
    assert_eq!(one.len(), 13);
    for len in 3..=13 {
        assert_eq!(
            hashes.resolve(one.get(..len).unwrap_or_default()),
            ambiguous(&["one", "two"]),
            "{len}"
        );
    }
    let three = hashes.hash_of("three").unwrap_or_default();
    assert_eq!(three.len(), 3);
    assert_eq!(hashes.resolve(three), found("three"));
}

#[test]
fn dated_stems_get_three_character_hashes() {
    let stems: Vec<String> = (1..=30)
        .map(|day| format!("standup-2026-10-{day:02}"))
        .collect();
    let hashes = StemHashes::new(stems.iter().map(String::as_str));
    for stem in &stems {
        assert_eq!(hashes.hash_of(stem).map(str::len), Some(3), "{stem}");
    }
}

#[test]
fn the_mixer_is_the_splitmix64_finaliser() {
    assert_eq!(mix(0), 0);
    assert_eq!(mix(1), 0x5692_161d_100b_05e5);
}

#[test]
fn a_hash_lengthens_past_a_stem_and_a_reserved_word() {
    let plain = StemHashes::new(["alpha"]);
    let short = plain.hash_of("alpha").unwrap_or_default().to_owned();

    let by_stem = StemHashes::new(["alpha", short.as_str()]);
    assert_eq!(by_stem.hash_of("alpha").map(str::len), Some(4));

    let by_word = StemHashes::reserving(["alpha"], |word| word == short);
    assert_eq!(by_word.hash_of("alpha").map(str::len), Some(4));
    assert_eq!(by_word.resolve(&short), found("alpha"));
}

// --- resolve_ref: one resolver for a path, a stem or a stem hash -------------

#[allow(clippy::unwrap_used)]
mod reference {
    use std::path::Path;

    use super::*;
    use crate::test_support::bare_task;

    fn tasks(stems: &[&str]) -> Vec<Task> {
        stems.iter().map(|stem| bare_task(stem)).collect()
    }

    fn resolve(
        tasks: &[Task],
        root: &Path,
        reference: &str,
    ) -> Result<(String, std::path::PathBuf), RefError> {
        let hashes = StemHashes::new(tasks.iter().map(|task| task.id.as_str()));
        resolve_ref(tasks, root, root, &hashes, reference)
            .map(|got| (got.task.id.clone(), got.path))
    }

    fn root_with(stems: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("tasks")).unwrap();
        for stem in stems {
            std::fs::write(dir.path().join(format!("tasks/{stem}.typ")), "").unwrap();
        }
        dir
    }

    #[test]
    fn a_stem_resolves_to_its_task_and_file() {
        let tasks = tasks(&["alpha", "beta"]);
        let got = resolve(&tasks, Path::new("/p"), "beta").unwrap();
        assert_eq!(
            got,
            (
                "beta".to_owned(),
                Path::new("/p/tasks/beta.typ").to_path_buf()
            )
        );
    }

    #[test]
    fn a_stem_hash_resolves_when_no_stem_matches() {
        let tasks = tasks(&["alpha", "beta"]);
        let hash = StemHashes::new(["alpha", "beta"])
            .hash_of("beta")
            .unwrap()
            .to_uppercase();
        assert_eq!(resolve(&tasks, Path::new("/p"), &hash).unwrap().0, "beta");
    }

    #[test]
    fn an_exact_stem_wins_over_a_hash_it_spells() {
        let spelled = StemHashes::new(["alpha"])
            .hash_of("alpha")
            .unwrap()
            .to_owned();
        let tasks = tasks(&["alpha", &spelled]);
        assert_eq!(
            resolve(&tasks, Path::new("/p"), &spelled).unwrap().0,
            spelled
        );
    }

    #[test]
    fn an_unknown_reference_is_no_match() {
        let tasks = tasks(&["alpha"]);
        let got = resolve(&tasks, Path::new("/p"), "zzzz");
        assert!(
            matches!(got, Err(RefError::NoMatch(ref r)) if r == "zzzz"),
            "{got:?}"
        );
    }

    #[test]
    fn an_empty_reference_is_its_own_error() {
        let tasks = tasks(&["alpha"]);
        let got = resolve(&tasks, Path::new("/p"), "");
        assert!(matches!(got, Err(RefError::Empty)), "{got:?}");
    }

    #[test]
    fn a_shared_hash_is_ambiguous_naming_each_stem() {
        let tasks = tasks(&["a-1", "a-2"]);
        let hashes = StemHashes::with_hasher(["a-1", "a-2"], by_first_byte);
        let prefix = hashes.hash_of("a-1").unwrap().get(..3).unwrap().to_owned();
        let got = resolve_ref(&tasks, Path::new("/p"), Path::new("/p"), &hashes, &prefix);
        assert!(
            matches!(got, Err(RefError::Ambiguous { ref stems, .. }) if stems == &["a-1", "a-2"]),
            "{got:?}"
        );
    }

    #[test]
    fn a_path_resolves_from_the_working_directory() {
        let dir = root_with(&["alpha"]);
        let tasks = tasks(&["alpha"]);
        let abs = dir.path().join("tasks/alpha.typ").canonicalize().unwrap();
        for reference in ["tasks/alpha.typ", abs.to_str().unwrap()] {
            let got = resolve(&tasks, dir.path(), reference).unwrap();
            assert_eq!(got, ("alpha".to_owned(), abs.clone()), "{reference}");
        }
    }

    #[test]
    fn a_backslash_reads_as_a_path_not_a_stem_hash() {
        let dir = root_with(&["alpha"]);
        let got = resolve(&tasks(&["alpha"]), dir.path(), "tasks\\alpha");
        assert!(matches!(got, Err(RefError::Path { .. })), "{got:?}");
    }

    #[test]
    fn a_path_naming_no_task_says_where() {
        let dir = root_with(&["alpha"]);
        std::fs::write(dir.path().join("tasks/stray.typ"), "").unwrap();
        let got = resolve(&tasks(&["alpha"]), dir.path(), "tasks/stray.typ");
        assert!(matches!(got, Err(RefError::NoTaskAt(_))), "{got:?}");
    }

    #[test]
    fn a_missing_path_is_a_path_error() {
        let dir = root_with(&[]);
        let got = resolve(&tasks(&["alpha"]), dir.path(), "a/b");
        assert!(matches!(got, Err(RefError::Path { .. })), "{got:?}");
        assert!(got.unwrap_err().to_string().starts_with("resolve "));
    }

    #[test]
    fn only_a_path_shaped_reference_is_a_path() {
        for path in ["a/b", "a\\b", "x.typ", "/abs"] {
            assert!(is_path_ref(path), "{path}");
        }
        assert!(!is_path_ref("alpha"));
    }

    #[test]
    fn typed_digits_match_a_stems_whole_code_not_its_printed_hash() {
        let hashes = StemHashes::new(["fix-login-bug"]);
        assert!(hashes.matches("fix-login-bug", "r7"));
        assert!(hashes.matches("fix-login-bug", "R7EWE2"));
        assert!(!hashes.matches("fix-login-bug", "r8"));
        assert!(!hashes.matches("fix-login-bug", "r7u"));
        assert!(!hashes.matches("other", "r7"));
    }
}
