#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices
#import "../utils/config-ref.typ": config-keys-table, config-values-table

#let meta = (
  route: "/rules/",
  label: "Rules",
  title: "Rules",
  summary: "The checks a project adds to its tasks: required fields, assertions, Typst checks, body rules and comment policy.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Every block below comes from examples/rules/, the project
// `just check-examples` validates: each TOML block is a slice of its config,
// and each refusal is a real `mt` run against a copy of it
// (www/generate-config-runs.sh).
#let schema = json("../generated/config-schema.json")
#let slices = checked-slices(
  json("../generated/rules-config.json"),
  "rules-config",
  ("required", "assert", "wip-cap", "check", "headers", "content", "comments"),
)

Rules are how you tell `mt` what a good task looks like in your project.
`mt check` and every command that writes a task file run the same checks. They
check each field against the config, such as a status against the
#link(u("/statuses/"))[status legend], plus any checks you add with the keys
on this page. Rules from an ancestor directory's defaults and from rule packs
add to your project's own, as #link(u("/config/"))[Configuration] describes.
To share one set of rules between projects, see
#link(u("/rule-packs/"))[Rule packs]. To list the rules your project ends up
with, one line each, run #link(u("/cli/rules/"))[`mt rules`].

= Required fields

`required` takes `true`, `false`, or a #link(u("/filters/"))[filter
  expression]. With an expression, the field is required on exactly the tasks
it matches. You can set it on `[tasks.priority]`, `[tasks.difficulty]`,
`[tasks.estimate]`, `[tasks.assignee]` and each `[tasks.fields.<name>]`:

#raw(
  render-slices(
    slices,
    (
      (
        "required",
        "required while open; an estimate on bugs only; a known assignee",
      ),
    ),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

`mt` refuses a write that would leave a required field empty:

#terminal("rules-required")

`mt` parses each expression once, when it loads the config, so a broken one
gives you one error that names the field.

= Assertions

A `[[tasks.rules]]` entry with `assert` checks every task it selects against
a filter expression. Unlike `required`, it can test what a field holds:

#raw(
  render-slices(
    slices,
    (("assert", "an error on open bugs, and a warning on every task"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

#config-keys-table(schema, "tasks.rules")

An `error` breach refuses the write, and nothing is written:

#terminal("rules-assert")

A `warn` breach prints a warning, and the write goes ahead:

#terminal("rules-warn")

`mt check` reports the same breach at the line of the task's `task.with(…)`
call. A rule with no `name` reports by its position, `rules[0]`, so you can
always find the entry. `mt` runs each `assert` and `when` once against a
sample task when it loads the config. So a rule that names an unknown field,
#link(u("/named-sets/"))[view or dictionary] gives you one config error, not
one breach per task.

= Work-in-progress caps

`count(...)` counts the loaded tasks a filter matches (see
#link(u("/filters/"))[Filter language]). Compare it with a number to cap how
many tasks can be in a set at once:

#raw(
  render-slices(
    slices,
    (("wip-cap", "at most one task in flight"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

`when` repeats the counted filter on purpose. The comparison gives the same
answer for every task, so the rule reports a breach on every task `when`
selects: the task moving into the set, and each task already in it. The second
line below is the breach on the task already in flight, reported against the
#link(u("/statuses/"))[flip] that caused it:

#terminal("rules-wip-cap")

The capped set can be any filter, so the same rule caps a view
(`when = '@stuck'`), a #link(u("/priority/"))[priority band] or a `kind`.

= Typst checks

The filter language reads an `x-<ns>` field's text, number or list, but
can't look inside a dictionary. `[tasks.fields.x-<ns>]` only covers a list of
strings and a `required` rule. For any other shape, write a rule with `check`
instead of `assert`. It calls a Typst function you ship in the project, with
one field's value or the whole task:

#raw(
  render-slices(
    slices,
    (("check", "every open task lists its acceptance criteria"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

#config-keys-table(schema, "tasks.rules.check")

The function returns an array of strings. An empty array passes. Each string
is one breach, reported like an `assert` breach at the rule's `severity`:

#raw(read("../generated/rules-lib.typ"), lang: "typ", block: true)

The value arrives as a Typst literal, and a content block in it as its plain
text. A date in a `field` value arrives as a `datetime`. Without `field`, the
task arrives in the shape `mt eval --json` prints, where a date is a
`YYYY-MM-DD` string. A Typst error, a `panic()` in the function, or a return
that isn't an array of strings counts as one breach with the message, and the
rest of the project still loads.

When `mt` loads the config, it checks that the module exists and exports the
function, and reports each failure as its own config error. It doesn't call
the function then, so a function that rejects some values can't fail the
config load. #link(u("/self-validating-tasks/"))[Self-validating tasks] walks
through a complete example.

= Body rules

To make every task carry, say, a Summary and acceptance criteria, check its
body. Two rule families do that. Both pick sections by a path through the
heading tree. A bare string matches a heading's text exactly, and only among
the direct children of the previous step. `{ match = "regex" }` matches
heading text with a regex, and `recursive = true` lets it match at any depth.

`[[tasks.body.headers.rules]]` checks which headings exist and in what order:

#raw(
  render-slices(
    slices,
    (("headers", "one Summary, and no heading outside a fixed set"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

#config-keys-table(schema, "tasks.body.headers.rules")

`[[tasks.body.content.rules]]` checks the text under a heading: everything
down to the next heading of the same or a higher level, subsections included.

#raw(
  render-slices(
    slices,
    (
      (
        "content",
        "a non-empty Summary, a detailed repro on bugs, and a dash limit",
      ),
    ),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

`assert` takes one of six forms:

- `"non-empty"`: the section has some text.
- `{ min-words = N }` and `{ max-words = N }`: a word is any run of
  non-whitespace characters containing a letter or a digit. Markup, code and
  subsection headings count.
- `{ must-match = "regex" }` and `{ must-not-match = "regex" }`: the regex runs
  against the section's source text, markup included.
- `{ density = { pattern = "regex", max = N, per = "section" } }`: no single
  unit carries more than `max` matches. `per` is `section` (the default),
  `paragraph` or `sentence`. Paragraphs split at blank lines and at each list
  item. Sentences split after a `.`, `!` or `?` followed by a space or the
  end. Fenced code is removed first, so a match inside a code sample never
  counts.

`mt` refuses a write that breaks a body rule, the same as a field rule.
`mt check` reports a body violation at the heading's line, or at the
`task.with(…)` line if a required section is missing, named
`body.headers.rules.<name>` or `body.content.rules.<name>`.

= Review comments

If you or a reviewer leave comments in task files, you may want them cleared
before work moves on. `[tasks.comments]` sets what `mt check` does with a
comment anywhere in a task file, inside `task.with(…)` or in the body:

#raw(
  render-slices(
    slices,
    (("comments", "fail on any comment except a TODO"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

#config-values-table(schema, "CommentsPolicy")

`allow-pattern` is a regex matched against the comment's text, trimmed and
without its comment markers. Comments it matches are let through. `mt` finds
comments by parsing the file, so `//` inside a string or a raw block is never
reported. Only `mt check` applies this policy. A write command won't refuse a
task for a comment.
