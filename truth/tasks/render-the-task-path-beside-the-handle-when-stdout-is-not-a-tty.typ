#import "@local/mindtape:0.2.0": *

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
  )[two halves of one fix — this makes the path available, that makes it the thing an agent quotes]
    + related(
      "the-hyperlinks-flag-help-names-only-the-file-column.typ",
    )[both settle what the]
    + related(
      "ls-rules-lose-their-sort-and-columns-facets-under-an-explicit-filter.typ",
    )[column rules are chosen here partly because the ls.rules bundle cascade cannot carry a presentation facet under a filter],
)

== Summary

The \`mt ls\` table renders \`\#\` handles whose ordinal is a position in a view
recomputed on every command, so an agent that quotes one back to a human
hands over an id that may already name a different task by the time it is
read. Every other renderer already avoids this: \`--json\` carries \`id\` and
\`file\` and no handle at all, and \`--format csv\` substitutes \`id\` for \`\#\`.
The table is the only surface emitting a positional id with no stable
identity beside it. Show the \`file\` column automatically when stdout is not
a TTY, so a single \`mt ls\` yields both the short handle and a path.


== Scope

- \`crates/mindtape/src/condition.rs\` — add a \`tty()\` predicate beside
  \`env\`, \`on\_branch\`, \`now\` and \`weekday\`. It answers "is stdout a TTY",
  resolved once per process like the rest of the condition environment.
- \`crates/mindtape/src/config\_ls.rs\` — give \`ColumnConfig\` a \`rules\` list,
  each entry a \`condition\` plus a \`visibility\`. Validate the condition at
  config load, as \`\[\[ls.rules\]\]\` already does.
- \`crates/mindtape-cli/src/render/columns.rs\` — evaluate the rules when
  deciding whether a column renders.
- The embedded floor config ships
  \`\[\[ls.columns.file.rules\]\] condition = '!tty()'\` / \`visibility = "visible"\`,
  so a project with no config still never hands an agent a bare handle.
- Table only. \`--format csv\` honours the column set and is a data contract;
  silently gaining a column would break someone's parser, and CSV already
  carries \`id\`.
- \`www/content/config.typ\` and the config JSON schema document the new key.

Settled precedence, one regression test per step: \`--columns\` wins over
everything; a matching rule wins over the column's own \`visibility\`, since
rules ARE the conditional-override mechanism; the bare \`visibility\` wins
over the built-in default. A project opts out of an inherited rule with
\`rules = \[\]\`, which is explicit and greppable, rather than by setting
\`visibility\` and wondering why it lost.

= Watch for

Shipping the rule in the floor changes \`mt ls\` output for every existing
project the moment it is piped. Is a table gaining a column acceptable
where a CSV gaining one is not? The position taken here is yes — the
aligned table is a presentation surface and \`--format csv\` / \`--json\` are
the contracts — but a project that disagrees needs \`rules = \[\]\` to be a
documented opt-out, not folklore.

