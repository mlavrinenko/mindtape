# Filter language

Reference for the mindtape skill. Read this when you write a filter for
`mt ls`, `mt rm`, a view or a config rule. Try a filter with `mt ls` before you
save it in config, because `mt ls` shows what it selects.

## Fields

- `status`, `kind`, `title`, `id` and `assignee` are strings.
- `priority` and `difficulty` are numbers. A framework priority such as
  `ice(...)` compares as its score.
- `due`, `start`, `reviewed` and `status.date` are ISO dates and compare as
  strings. `now()` on the right side is today: `due < now()`.
- `status.name` is the same value as `status`. `status.note` is the status
  note, and `status.index` is the status's 0-based position in the legend.
- `path` is the task file's path from the project root, with `/` separators.
  `file` is a column only, and a filter on it is an error.
- `depth` is the folder depth under the root, 0 for a task in the root itself.
- `topo` is 0 for a task nothing blocks, else one more than its deepest
  dependency. Ascending `topo` is a valid working order.
- `blocks` counts the unfinished tasks that wait on this one, directly or
  through other tasks.
- `children` counts the task's `parent`-edge children. `progress` is the
  finished share of them, from 0 to 1, and a task with no children has none.
- `role == "reviewer"` matches a task whose assignee holds that role in
  `[[actors]]`, and `role != "reviewer"` matches every other task.
- `estimate.<unit>` is one unit of the `estimate` dict, as a number.
- `estimate` and `tags` take `has()` and `miss()` only.
- An `x-<ns>` slot reads whether or not `[tasks.fields]` declares it. Text, a
  number or a boolean compares like any field: `x-machine == "alpha"`. A list
  equals nothing, so `has(x-machine, "alpha")` tests membership. A dict such
  as `x-goals` takes `has()` and `miss()` only.

A name keeps its hyphens, as in the task file: `x-goals`,
`estimate.src-files`.

A field the task does not carry is unequal to every literal, so
`kind != "bug"` matches a task with no `kind`. Use `miss(kind)` to select
absence.

`--sort` takes every field listed above `role`, an `x-<ns>` slot, and
`ready`, which sorts ready tasks before blocked ones and finished ones last.
Bare `status` sorts by legend position.

## Operators

`==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!` and parentheses.

## Predicates

- `ready` matches an open task whose `depends-on` targets are all finished.
  `blocked` matches an open task with an unfinished target. A link kind the
  config marks `blocking` counts as `depends-on` does.
- `terminal` matches a task whose status the legend marks terminal.
- `has(f)` and `miss(f)` test whether the task carries field `f`.
  `has(f, "v")` matches when `f`'s list holds `v`, its scalar equals `v`, or
  its dict has the key `v`.
- `has_tag("t")` matches an authored or folder-derived tag.
- `links("kind")` matches a task with at least one edge of that kind.
- `contains("s")` is a case-insensitive substring search over `title`, `id`
  and the body, or over the fields in `[ls] contains-fields` when the project
  sets it. `contains("s", f, …)` searches the named fields: any field above,
  `body`, `note` (the link notes) or `tags`.
- `stale(N)` matches a task whose `reviewed` date, or else status date, is
  more than N days old. A task with neither date is stale too.
- `in_view("v")`, written `@v` for short, matches when view `v`'s filter does.
- `matches(f, "re")` matches when the field's text matches the unanchored
  regular expression.
- `in_dict("d", s)` matches when string `s` is in dictionary `d`, a
  `[dicts.d]` section or a built-in. `word(f, n)` gives the n-th word of a
  string field, counting from 1, and works only as an argument here or in
  `matches`: `in_dict("verbs", word(title, 1))`. The `verbs` dictionary comes
  from the `mindtape-tasks` pack. Without the pack, use the built-in
  `mindtape-verbs` or declare `[dicts.verbs]`.
- `touches("path")` matches the tasks named in the `Refs:` trailers of commits
  that changed that file. `[commits] trailer` sets the trailer key. Run
  `mt ls 'touches("<path>")'` before you edit a file.

## Counts

A count is a number, so it goes in a comparison.

- `count(expr)` counts the loaded tasks that match `expr`, whatever the rest
  of the filter selects: `count(status == "wip") <= 1`.
- `changed("states", "glob", "!glob", …)` counts the paths git reports
  changed. The states are `staged`, `worktree` or `staged+worktree`. Globs are
  relative to the repository root, and a `!` glob excludes:
  `changed("staged+worktree", "src/**") <= estimate.src-files`.

## In config only

`into("s")` matches when a flip stamps status `s`. It works in an
`[[on.flip]]` `when` expression and in a `scope = "task"` commit rule.
