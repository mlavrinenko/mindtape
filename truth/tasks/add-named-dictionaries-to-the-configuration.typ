#import "@local/mindtape:0.2.0": *

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
  status: proposed(2026, 9, 11),
)

== Summary

MindTape has no way to name a set of strings and reuse it. Every vocabulary is
written out where it is used — `[tasks.tags] allowed`, `[tasks.fields.<name>]
values` — so a set that two rules share is two lists that drift, and a set too
large to paste (an English verb list behind a title rule) has nowhere to live
at all.

`[dicts.<name>]` names one. It is a top-level section, sibling of
`[views.<name>]`, because a dictionary is not a property of tasks: the filter
language, an allow-list and a future consumer all read the same named set.


== Scope

+ Add `[dicts.<name>]` with `values` (inline array), `path` (a file relative to
  the project root, one entry per line, `#` comments and blank lines skipped),
  `include` (names of other dicts, built-in or project, unioned in) and `case`
  (`fold`, the default, or `exact`). All four may combine; the result is a set.
+ Validate at config load, once, not per task: an unknown `include` name (with
  a nearest-match suggestion), an `include` cycle, a `path` that does not
  resolve or read, and a dict that ends up empty.
+ Ship two built-in dicts under the reserved `mindtape-` prefix, embedded in
  the binary: `mindtape-verbs` (the 500 most frequent English verbs in base
  form) and `mindtape-dev-verbs` (86 software-work verbs the frequency list
  ranks too low to carry). A project may `include` either; naming a project
  dict `mindtape-*` is a config error, so the namespace stays the tool's.
+ Surface them in `mt config show` and in the generated JSON schema, and
  document the section on the config page.
+ Cover it: each source shape alone and combined, `fold` against a capitalised
  lookup, the reserved prefix refused, and every load-time error naming the
  offending dict.

