#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Match a text prefix with a starts_with predicate",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("mt", "tests", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[a prefix rule reads as a prefix, not a regex anchor],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 10),
)

== AS-IS

+ Write `when = 'matches(title, "^Fix")'` to match a title that starts with a verb.
+ 🔴 The pattern needs a regex anchor and is case-sensitive, so `fix the login` misses.
+ `word(title, 1) == "Fix"` only checks the first word and trims punctuation, not a prefix.


== TO-BE

- 🟢 `starts_with("fix", title)` matches when `title` starts with `fix`, case-insensitively, the way `contains` matches a substring.
- 🟢 The needle comes first and the field second, and the field is required, so the call never guesses among title, id and body.
- 🟢 A missing field or a wrong arity is a filter error naming the call.

```
$ mt ls 'starts_with("fix", title)'
fix-the-login-redirect
```


== Changes

- `crates/mindtape/src/filter/predicates.rs`: add `starts_with` beside `contains`, sharing its case fold and field reader.
- `crates/mindtape/src/filter/eval.rs`: register `starts_with` with a two-argument arity.
- `crates/mindtape/src/filter/predicates_tests.rs`: a prefix, a case mismatch, a missing field and a wrong arity.
- `www/content/filters.typ`: document the predicate and its argument order.


== Alternatives

- `matches(title, "^Fix")`: a regex anchor, case-sensitive, and a second language for a prefix.
- `word(title, 1) == "Fix"`: only the first word, and punctuation is trimmed.
- `contains` with a start flag: a flag for one predicate, and two meanings in one call.


== Out of scope

- `ends_with`; add it when a rule asks.
- Unicode case folding beyond `to_lowercase`.
- A default haystack; a prefix needs one field.


== Dogfood

This repo's add-hook example matches a title starting with "Fix" through `starts_with("fix", title)`.

