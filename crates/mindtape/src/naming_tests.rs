#![allow(clippy::unwrap_used)]

use super::{
    DEFAULT_STEM_TEMPLATE, DedupMode, MAX_DISAMBIGUATE_ATTEMPTS, MintContext, NamingConfig,
    disambiguate, mint_stem,
};

fn ctx<'a>(slug: &'a str, date: (i32, u8, u8), occurrence: u32) -> MintContext<'a> {
    MintContext {
        slug,
        date,
        occurrence,
    }
}

#[test]
fn default_template_is_bare_slug() {
    assert_eq!(DEFAULT_STEM_TEMPLATE, "{slug}");
    let out = mint_stem(
        DEFAULT_STEM_TEMPLATE,
        &ctx("cleanup-kitchen", (2026, 7, 10), 0),
    )
    .unwrap();
    assert_eq!(out, "cleanup-kitchen");
}

#[test]
fn template_default_returns_default_stem_template() {
    let cfg = NamingConfig::default();
    assert_eq!(cfg.template(), DEFAULT_STEM_TEMPLATE);
    assert_eq!(cfg.template(), "{slug}");
}

#[test]
fn template_configured_returns_stem() {
    let cfg = NamingConfig {
        stem: Some("{slug}-{date}".into()),
        ..NamingConfig::default()
    };
    assert_eq!(cfg.template(), "{slug}-{date}");
}

// -- DedupMode: `[tasks.naming] dedup` deserialize + default -----------------

#[test]
fn dedup_mode_default_is_fail() {
    assert_eq!(NamingConfig::default().dedup, DedupMode::Fail);
}

#[test]
fn dedup_mode_deserializes_each_value() {
    let cases = [
        ("\"fail\"", DedupMode::Fail),
        ("\"skip\"", DedupMode::Skip),
        ("\"add-suffix\"", DedupMode::AddSuffix),
    ];
    for (json, expected) in cases {
        let mode: DedupMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, expected, "{json}");
    }
}

#[test]
fn dedup_mode_rejects_unknown_value() {
    let err = serde_json::from_str::<DedupMode>("\"bogus\"").unwrap_err();
    assert!(
        err.to_string()
            .contains("invalid dedup mode `bogus`; expected `fail`, `skip`, or `add-suffix`"),
        "{err}"
    );
}

#[test]
fn speconaut_style_template() {
    let template = "{slug}.task-{yy}{mm}{dd}";
    let out = mint_stem(template, &ctx("ship-it", (2026, 7, 9), 0)).unwrap();
    assert_eq!(out, "ship-it.task-260709");
}

#[test]
fn date_macro_is_iso() {
    let out = mint_stem("{slug}-{date}", &ctx("foo", (2026, 1, 5), 0)).unwrap();
    assert_eq!(out, "foo-2026-01-05");
}

#[test]
fn date_macro_with_format() {
    let out = mint_stem("{date:DD.MM.YYYY}", &ctx("foo", (2026, 7, 9), 0)).unwrap();
    assert_eq!(out, "09.07.2026");
}

#[test]
fn yy_mm_dd_macros() {
    let out = mint_stem("{yy}/{mm}/{dd}", &ctx("foo", (2026, 3, 4), 0)).unwrap();
    assert_eq!(out, "26/03/04");
}

#[test]
fn bare_occurrence_macro() {
    let out = mint_stem("{slug}-{n}", &ctx("foo", (2026, 7, 10), 3)).unwrap();
    assert_eq!(out, "foo-3");
}

#[test]
fn zero_padded_occurrence_macro() {
    let out = mint_stem("{slug}-{n:03}", &ctx("foo", (2026, 7, 10), 5)).unwrap();
    assert_eq!(out, "foo-005");
}

#[test]
fn zero_padded_occurrence_wider_than_value() {
    let out = mint_stem("{n:02}", &ctx("foo", (2026, 7, 10), 123)).unwrap();
    // Width is a floor, never a truncation: the value always survives whole.
    assert_eq!(out, "123");
}

#[test]
fn kind_macro_is_a_reserved_passthrough() {
    let out = mint_stem("{slug}-{kind}", &ctx("foo", (2026, 7, 10), 0)).unwrap();
    assert_eq!(out, "foo-{kind}");
}

#[test]
fn unknown_macro_errors() {
    let err = mint_stem("{nope}", &ctx("foo", (2026, 7, 10), 0)).unwrap_err();
    assert_eq!(err.to_string(), "unknown stem template macro {nope}");
}

#[test]
fn unterminated_macro_errors() {
    let err = mint_stem("{slug", &ctx("foo", (2026, 7, 10), 0)).unwrap_err();
    assert_eq!(
        err.to_string(),
        "unterminated macro (missing '}') in stem template"
    );
}

#[test]
fn invalid_occurrence_width_errors() {
    let err = mint_stem("{n:abc}", &ctx("foo", (2026, 7, 10), 0)).unwrap_err();
    assert_eq!(err.to_string(), "invalid zero-pad width \"abc\" in {n:...}");
}

#[test]
fn literal_braces_pass_through_untouched_text() {
    let out = mint_stem("shopping/{slug}", &ctx("milk", (2026, 7, 10), 0)).unwrap();
    assert_eq!(out, "shopping/milk");
}

// -- disambiguate: fs-free collision guard -----------------------------------

#[test]
fn disambiguate_keeps_free_stem() {
    let out = disambiguate("foo", |_| false).unwrap();
    assert_eq!(out, "foo");
}

#[test]
fn disambiguate_appends_dash_two_on_first_collision() {
    let out = disambiguate("foo", |candidate| candidate == "foo").unwrap();
    assert_eq!(out, "foo-2");
}

/// The suffix search must skip candidates that are themselves taken, not stop
/// at the first one it builds: "foo" and "foo-2" taken means "foo-3".
#[test]
fn disambiguate_skips_a_taken_suffix() {
    let out = disambiguate("foo", |candidate| {
        candidate == "foo" || candidate == "foo-2"
    })
    .unwrap();
    assert_eq!(out, "foo-3");
}

#[test]
fn disambiguate_walks_past_multiple_collisions() {
    let taken = ["foo", "foo-2", "foo-3"];
    let out = disambiguate("foo", |candidate| taken.contains(&candidate)).unwrap();
    assert_eq!(out, "foo-4");
}

#[test]
fn disambiguate_never_returns_a_taken_stem() {
    let out = disambiguate("foo", |candidate| candidate != "foo-5").unwrap();
    assert_eq!(out, "foo-5");
}

#[test]
fn disambiguate_exhausts_ceiling_when_every_candidate_is_taken() {
    // A predicate that reports every candidate as taken must not loop
    // forever (the `n += 1` -> `n *= 1` mutant froze `n` at 2 and spun until
    // the whole suite timed out) and must not return a stem `exists` reports
    // as taken — it must error once the numeric-suffix ceiling is exhausted.
    let err = disambiguate("foo", |_| true).unwrap_err();
    assert_eq!(
        err.to_string(),
        format!("exhausted {MAX_DISAMBIGUATE_ATTEMPTS} disambiguation attempts for stem \"foo\"")
    );
}

// -- reuse shape: a simulated `mt recur` spawn -------------------------------
//
// `mt recur` (a later wave) mints occurrence stems by building its own
// `MintContext` with the occurrence's anchor date and index — never today's
// clock. This proves the expander is deterministic and clock-free: the same
// template plus the same (caller-supplied) date/occurrence always renders the
// same stem, regardless of when the test runs.

#[test]
fn recur_style_spawn_is_deterministic_given_explicit_date_and_index() {
    let template = "{slug}-{date}-{n:02}";
    let anchor = (2026, 7, 6); // the occurrence's anchor date, not "today"
    let first = mint_stem(template, &ctx("standup", anchor, 1)).unwrap();
    let second = mint_stem(template, &ctx("standup", anchor, 1)).unwrap();
    assert_eq!(first, second);
    assert_eq!(first, "standup-2026-07-06-01");
}

#[test]
fn recur_fallback_shape_slug_dash_date() {
    // The built-in fallback recur uses when naming is unset: `{slug}-{date}`.
    let out = mint_stem("{slug}-{date}", &ctx("standup", (2026, 7, 13), 4)).unwrap();
    assert_eq!(out, "standup-2026-07-13");
}
