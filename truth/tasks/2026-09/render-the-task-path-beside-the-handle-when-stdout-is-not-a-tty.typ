#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "render the task path beside the handle when stdout is not a tty",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 5,
  tags: ("cli",),
  estimate: (files: 8),
  assignee: "agent",
  status: done(2026, 9, 8),
  links: related(
    "cite-task-paths-as-links-never-handles-in-prose-to-a-human.typ",
  )[two halves of one fix. This makes the path available, and that makes it the thing an agent quotes.]
    + related(
      "the-hyperlinks-flag-help-names-only-the-file-column.typ",
    )[both settle what the file column shows.]
    + related(
      "ls-rules-lose-their-sort-and-columns-facets-under-an-explicit-filter.typ",
    )[the fix adds column rules here partly because the ls.rules bundle cascade cannot carry a presentation facet under a filter],
)

== Summary

The `mt ls` table renders `#` handles whose ordinal is a position in a view
recomputed on every command. An agent that quotes one back to a human
hands over an id that may already name a different task by the time a human
reads it. Every other renderer already avoids this: `--json` carries `id` and
`file` and no handle at all, and `--format csv` substitutes `id` for `#`.
The table is the only surface emitting a positional id with no stable
identity beside it. Show the \`file\` column automatically when stdout is not
a TTY, so a single \`mt ls\` yields both the short handle and a path.


== Scope

- `crates/mindtape/src/condition.rs`: add a `tty()` predicate beside
  `env`, `on_branch`, `now` and `weekday`. It answers "is stdout a TTY",
  resolved once per process like the rest of the condition environment.
- `crates/mindtape/src/config_ls.rs`: give `ColumnConfig` a `rules` list,
  each entry a \`condition\` plus a \`visibility\`. Validate the condition at
  config load, as \`\[\[ls.rules\]\]\` already does.
- `crates/mindtape-cli/src/render/columns.rs`: evaluate the rules before a
  column renders.
- The embedded floor config carries
  \`\[\[ls.columns.file.rules\]\] condition = '!tty()'\` / \`visibility = "visible"\`,
  so even a project without a config never hands an agent a bare handle.
- Table only. `--format csv` honors the column set and is a data contract.
  Silently gaining a column would break someone's parser, and CSV already
  carries `id`.
- \`www/content/config.typ\` and the config JSON schema document the new key.

The precedence settles as follows, with one regression test per step. `--columns` wins over
everything. A matching rule wins over the column's own `visibility`, since
rules are the conditional-override mechanism. The bare `visibility` wins
over the built-in default. A project opts out of an inherited rule with
`rules = []`, which is explicit and easy to search, rather than by setting
`visibility` and wondering why it lost.

= Watch for

Putting the rule in the floor changes `mt ls` output for every existing
project the moment its output goes into a pipe. Is a table gaining a column acceptable
where a CSV gaining one is not? The answer here is yes: the
aligned table is a presentation surface, and `--format csv` / `--json` are
the contracts. A project that disagrees needs `rules = []` to be a
documented opt-out, not folklore.

