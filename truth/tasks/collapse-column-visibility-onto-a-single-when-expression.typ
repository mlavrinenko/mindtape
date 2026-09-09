#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "collapse column visibility onto a single when expression",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("cli",),
  estimate: (files: 10),
  assignee: "agent",
  status: done(
    2026,
    9,
    10,
  )[ColumnConfig collapsed to one when expression; Visibility, ColumnRule and auto\_significant deleted, the per-column defaults now one when per column in defaults.toml. Absent when = false, blank when refused at load. mt eval renders against the floor rather than Config::default(), which after this change describes no column. New eval\_e2e.rs pins both mt eval surfaces end to end (the two mutants the refactor exposed). just check green, just crap green (0 regressed, ColumnId::visible -4.0, auto\_significant/rule\_visibility removed), just mutants-diff 10/10 caught via \_\_DIRENV\_ARMED=1.],
  links: depends-on("evaluate-column-conditions-against-the-rendered-rows.typ")
    + related(
      "the-file-filter-field-and-the-file-column-name-different-paths.typ",
    )[both land on what the file column means to a reader; independent, either order],
)

== Summary

A column carries two keys that both answer "does this render": `visibility`
and a `rules` list. That needs a precedence rule (a matching rule wins), and
the precedence needs stating in three places plus an opt-out convention
(`rules = []`) that exists only because the two keys can disagree. One
boolean expression per column removes the disagreement rather than
documenting it.


== Scope

Replace both keys with one:

```toml
[ls.columns.file]
when = 'named() || (!tty() && format() == "table")'
```

No list, so no ordering and no first-match-wins; `||` does what rule order
did. A scalar replaces wholesale across config layers, so a project override
is `when = 'false'` on one line and the `rules = []` opt-out disappears with
the thing it was working around.

Delete `Visibility`, `ColumnRule` and `auto_significant`; the per-column
defaults they encoded move into `crates/mindtape/src/defaults.toml` as one
`when` per column, so the three different meanings of `auto` become legible
where a reader looks for them rather than staying in a Rust match. Roughly:
`#` and `title` are `true`, `status` / `priority` / `due` are `nonempty()`,
`assignee` is `contested()`, `file` is the expression above, `ready` is
whatever the previous task settled.

Decide what an absent `when` means. The recommendation is that no column
falls back to Rust at all — every column in the floor carries one, so the
floor is the complete statement — with an absent `when` reading as `false`,
since a column nobody described should not appear.

No migration shim and no deprecation window: this is pre-1.0 and the old
keys go away in the same commit as the new one.

Then `crates/mindtape/src/config_ls_columns.rs` shrinks to the `when`
string and its validation, `schema/config.schema.json` re-blesses
(`MINDTAPE_BLESS=1`), and three doc sites lose the precedence paragraph they
currently carry: `www/content/cli.typ`, `www/content/config.typ`, and the
embedded floor's own comment.

= Watch for

The floor rule for `file` is what stops an agent quoting a moving `#`
handle, and its `format() == "table"` half is what keeps CSV — a data
contract that resolves the same column set — from silently gaining a
column. Do the end-to-end tests in `crates/mindtape-cli/tests/tty_e2e.rs`
still pin both halves after the rewrite, or do they need porting to the new
key?

