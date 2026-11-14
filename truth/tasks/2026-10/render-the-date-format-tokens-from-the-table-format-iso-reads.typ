#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Render the date format tokens from the table format_iso reads",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 4),
  tags: ("found", "www"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a new or renamed date token reaches config.typ without a hand edit],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
)

== AS-IS

+ Integrating the docs-claims branch from the pre-release docs review
+ 🔴 #raw("www/content/config.typ"): the {date:FMT} token table on the config page repeats a doc comment in date/mod.rs by hand, so no gate fails when a token changes

```
www/content/config.typ: #table(... [`YYYY`, `YY`], [Four-digit year ...], [`MMMM`, `MMM`] ...)
crates/mindtape/src/date/mod.rs:92  /// | `YYYY` | 4-digit year |  (doc comment above format_iso)
```


This fell outside the task that surfaced it. The fix lives here rather than
in a closing report no one rereads. Its home is
#raw("crates/mindtape/src/date/mod.rs").

== TO-BE

- 🟢 One table in `date/mod.rs` lists each token and its description, and `format_iso` matches tokens by reading that table, longest first, so code and docs cannot disagree.
- 🟢 `mt _cli` exports the table, the facts step writes it to `www/generated/`, and `config.typ` renders it. The page then takes every token from the fact.

  ```json
  "date_tokens": [{"token": "YYYY", "expands_to": "four-digit year"}, ...]
  ```


== Changes

- `crates/mindtape/src/date/mod.rs`: the token table. `format_iso` follows it, and the doc comment points at it instead of repeating it.
- `crates/mindtape-cli/src/cli/tree_json.rs`: export it beside `filter_functions`.
- `www/generate-facts.sh` (or the step that reads `mt _cli`): write the fact. Register it with `check-doc-facts`.
- `www/content/config.typ`: render the table from the fact.


== Rollback

- Revert the commit.


== Out of scope

- Adding tokens.


== Dogfood

Not applicable: this repo's stems use `{slug}` only.

