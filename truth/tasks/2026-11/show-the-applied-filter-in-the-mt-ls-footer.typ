#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Show the applied filter in the mt ls footer",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("cli", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[the listing says what it selected, not only how many.],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the listing explains its own filter.],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 19),
  links: parent("make-mt-explain-the-session-brief.typ"),
)

== AS-IS

+ Run `mt ls` in this repo.
+ 🔴 The footer says `default view · 2 of 1013 tasks`, but not the filter the view applied.
+ Run `mt ls --json`.
+ 🔴 The filter is there under `applied.facets.filter.value`. The text reader needs a second command.
+ Run `mt explain views default`.
+ 🔴 The filter is there too, so the listing and the index each hold half the answer.

== TO-BE

- 🟢 The `mt ls` footer names the filter it applied when a view or an `[[ls.rules]]` entry supplied it: `default view (!terminal && (miss(start) || start <= now())) · 2 of 1013 tasks`.
- 🟢 A view with a `description` shows it. A narrowing `[[ls.rules]]` entry shows its filter.
- 🟢 The line wraps at the terminal width, so a long filter does not run off the edge.
- 🟢 A filter the user typed stays `your filter`. A match-all listing keeps the counts alone.
- 🟢 `--json` keeps `applied.facets.filter.value`, unchanged.

```text
$ mt ls
...
default view (!terminal && (miss(start) || start <= now())) · 2 of 1013 tasks
agent view (Open tasks an agent can take) · 3 of 1013 tasks
```

== Changes

- `crates/mindtape-cli/src/cmd/ls/rules_applied.rs`: the footer prints the filter value or the view's description.
- `crates/mindtape-cli/src/cmd/ls/rules_applied_ls_rules_tests_applied.rs`: the footer tests.
- `www/content/cli/ls.typ`: the footer shape.

== Alternatives

- Put the filter in `mt explain`: the index does not resolve the view, and `mt ls` is the command that applies it.
- Leave it to `--json`: the text reader is the one asking what the listing selected.
- Print every facet's value. The table shows the sort and columns. The filter is the one that hides rows.
- Always print the filter: the `agent` view's description is shorter and says why. The filter is in `mt explain views`.
- No wrapping: the default filter is long, and the line would run off the edge.

== Out of scope

- The sort and columns values. The table shows them.
- A new `--explain` flag. The footer is the place.
- The `[[ls.rules]]` cascade. `mt ls --json` keeps the full attribution.
- The view's description in `mt explain views`. It already prints it.

== Dogfood

This repo's `mt ls` says `default view (!terminal && ...)`. A session reads what the default view selects without `mt explain views`.
