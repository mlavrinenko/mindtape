#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Add named dictionaries to the configuration",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt",),
  estimate: (files: 12),
  assignee: "agent",
  status: done(
    2026,
    9,
    11,
  )[\[dicts.\<name>\] lands: values/path/include/case union into one set, two embedded built-ins under the reserved mindtape- prefix, all checked once at config load.],
)

== Summary

mindtape has no way to name a set of strings and reuse it. Every vocabulary
sits inline at the point of use (`[tasks.tags] allowed`, `[tasks.fields.<name>]
values`). A set that two rules share is two lists that drift. A set too
large to paste (an English verb list behind a title rule) has nowhere to live
at all.

`[dicts.<name>]` names one. It is a top-level section, sibling of
`[views.<name>]`, because a dictionary is not a property of tasks. The filter
language, an allow-list and a future consumer all read the same named set.


== Scope

+ Add `[dicts.<name>]` with four keys. `values` is an inline array. `path`
  points to a file relative to the project root, with one entry per line and
  `#` comments and blank lines skipped. `include` lists other dicts, built-in
  or project, and unions them in. `case` is `fold` (the default) or `exact`.
  All four may combine. The result is a set.
+ Validate at config load, once, not per task. The check covers an unknown
  `include` name (with a nearest-match suggestion), an `include` cycle, a
  `path` that does not resolve or read, and a dict that ends up empty.
+ Bundle two built-in dicts under the reserved `mindtape-` prefix, embedded
  in the binary. `mindtape-verbs` holds the 500 most frequent English verbs in
  base form. `mindtape-dev-verbs` holds 86 software-work verbs the frequency
  list ranks too low to carry. A project may `include` either. Naming a project
  dict `mindtape-*` is a config error. That rule keeps the namespace the tool's.
+ Surface them in `mt config show` and in the generated JSON schema, and
  document the section on the config page.
+ Cover it: each source shape alone and combined, `fold` against a capitalised
  lookup, the reserved prefix refused, and every load-time error that points
  at the offending dict.

