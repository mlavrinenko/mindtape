// docs/src/skill.typ — source for skills/mindtape/SKILL.md, rendered by
// docs/generate-md.sh via typlite (render-path constraints: docs/src/lib.typ).
// Two traps shape this source: `frontmatter` must stay a plain Typst string
// (prepended as YAML by `yq`, which typlite cannot emit), and the title's
// literal backtick is interpolated (`#"`"`), never a raw span, or the emitted
// h1 stops being plain text and the generator's demotion skips it. SKILL.md
// loads whole into an agent's context, so only what no command can teach
// belongs here; the deep dives live in skills/mindtape/references/ (linked).
#import "lib.typ": doc-title

#metadata((
  output: "skills/mindtape/SKILL.md",
  frontmatter: (
    name: "mindtape",
    description: "Drive MindTape, a daemon-free file-based task tracker, via its `mt` CLI and one-task-per-file Typst artifacts. Load in any repo holding `.mindtape/config.toml` for anything about tasks — listing, finding, filing, blocking, closing, validating, retiring, or \"what should I work on\" — even when the tool is not named. There it IS the tracker, not GitHub issues or ad-hoc TODOs; a repo with no tracker at all gets `mt init` offered before anything else.",
  ),
)) <mindtape-md>

#doc-title[MindTape (#"`"mt#"`")]

MindTape tasks are one Typst file each; the filename stem is the identity and
what `links:` names. Drive the `mt` CLI: a hand edit skips what `mt` stamps,
slugs, injects, resolves and validates, and writes validate before landing, so
a rejected write leaves nothing to repair. Hand-edit a `.typ` only for what no
command covers — the body, a framework priority, an `estimate` dict — then run
`mt check`.

= Is this a MindTape project?

Yes if `.mindtape/config.toml` exists at the repo root: it declares the task
roots and rules (status legend, priority scale, tags). Run `mt` from the
directory holding `.mindtape/`, or any descendant; from outside the tree use
`-w <path>`, `$MINDTAPE_WORKTREE`, or a `@name` that `mt worktrees` lists. No
tracker at all: offer `mt init`.

= The loop

#table(
  columns: 2,
  table.header([Goal], [Command]),
  [List active tasks], [`mt ls`],
  [List everything], [`mt ls ''` (the empty filter; no `--all` flag)],
  [Filter and sort], [`mt ls '<filter>' --sort <field>[:asc|desc]`],
  [Search by text], [`mt find <term…>`],
  [Read one task], [`mt eval <task>`],
  [File a task], [`mt add <title…>`],
  [Write one field], [`mt set <task> <field>=<value>`],
  [Rename a task], [`mt mv <task> <new-stem>`],
  [Change status], [`mt flip <status> <task…>` or the status as a verb],
  [Link tasks], [`mt link <task> <kind> <target…>`],
  [Validate], [`mt check`],
  [Retire], [`mt rm <task…>`],
)

`<task>` is a stem, a path, or a `#` handle (`1b`). Handles are recomputed on
every command — a flip that re-sorts or filters renumbers them — so use one
only for your very next command; in anything a human reads, cite the file path
instead (`[title](truth/tasks/<stem>.typ)`). A stem changes only when `mt mv`
renames it, which retargets every inbound edge in the same pass.

`mt set` writes `title priority difficulty kind assignee tags due start` —
`+=` appends to a list, `@path` reads the value out of a file, and a rejected
value leaves the file byte-identical. A field with its own verb (`status`,
`links`, `reviewed`, `recur`, the stem) is refused naming that verb.

Flags live in `mt <cmd> --help`. Per-project values — the legend and its
verbs, link kinds, the priority scale, tags — live in `.mindtape/config.toml`
(or `mt flip --list`); read them, never guess.

= Finding tasks

Bare `mt ls` applies an auto-activated `[[ls.rules]]` rule, else the default
view (`[ls] default-view`, else `[views.default]`; usually active, startable
tasks), naming the winner. A filter argument replaces the filter facet only —
the rule's or view's sort and columns still apply, so `--columns @default --sort @default`
restores the plain default view. `-n N` cuts after the sort and says how much
it cut; `--skip N` drops N rows first, and both the table footer and
`--json`'s `truncated.next_skip` name the `--skip` that pages forward from
here.

Fields: `status priority difficulty kind title id due start reviewed path assignee depth topo blocks children progress role`, plus `status.name`/`.date`/`.note`/`.index` and `estimate.<unit>` (a number; `has(estimate.<unit>)` tests it). Names are kebab-case as in the task file: `has(x-goals)`.
`--sort` takes all but `role`, plus a computed `ready` rank; `role` matches a
role the task's assignee holds.
`topo`/`blocks` read the `links:` graph, `blocks` counting unfinished tasks
transitively waiting on this one; `children` counts `parent`-edge children and
`progress` is their finished share, `0..1` (a `3/5` column; absent on a leaf). Operators: `== != < > <= >= && || !` and
parentheses. Helpers: `has(f)`/`miss(f)`, `has_tag("t")`, `links("kind")`,
`contains("s"[, field…])` (needle first; with no field it searches title/id/body,
or `[ls] contains-fields`), `ready`/`blocked`, `terminal`, `stale(N)`, `now()`,
`in_view("v")` (also `@v`), `matches(f, "re")`, `in_dict("d", word(f, n))`,
`changed("staged+worktree", "glob", "!glob")` (a count of git-changed paths,
compared to a literal or a field), `count(expr)` (a number over the whole
loaded set: `count(status == "wip") <= 1`), `touches("path")` (tasks the
trailers of the commits that changed a file name — run it before editing
one), and `into("s")` inside an
`[[on.flip]]` `when`.

`mt find a b` is `contains("a") && contains("b")`. A bare `@name` applies a
`[views.<name>]` bundle (filter, sort, columns). `--json` and `--format csv|path`
hand the listing to another program; `path` is the filter/sort field, `file`
only a display column.

= Filing tasks

`mt add <title…>` seeds the title and filename slug, writes under the first
root, and validates before writing. `--describe` shows the template's slots,
`--list` the templates, `--print` previews; `-f -` with YAML is the channel for
values you generate (one payload, no shell quoting), `--set k=v`, `k+=v` (list
append) and `k@path` (read a file) for values you type. An unknown or unfilled
slot is a hard error naming the slot.

After the fill, `mt add` prints a `note:` to stderr for each config-required
core field the template did not declare and you did not set: the field, its
value, where it came from (`[tasks.assignee]` env `MINDTAPE_PRINCIPAL`,
`[tasks.fields.kind]` default, or a config-spliced slot), and the
`--set <field>=<value>` that overrides it. Stdout stays clean for `--print`
and `--format json`.

`--describe` cannot show three slot semantics, each of which silently yields a
wrong task: token keys are slot names (a framework priority's slots are its
leaves, never `priority`), a `list` slot needs `name+=value`, and `text` slots
are markup-escaped while `raw` slots splice live. Read
`references/add-slots.md` before filling an unfamiliar template.

`mt import` files a whole backlog from canonical task-JSON (`mt ls --json`'s
schema). Its two silent sharp edges — the stem comes from `title`, not `id`,
and intra-batch `links` resolve only backwards — are in
`references/import.md`; read it before the first batch.

= Changing status

`mt flip <status> <task…>` stamps a legend value, dated today; every legend
value is also a verb, so `mt done 1b` works. `-m` attaches the status note. A
transition the legend's `to` edges disallow is rejected with the valid next
states listed, and so is entering an `unblocked = true` status while a
dependency is open, naming it, and closing a parent over an unfinished child;
`--force` overrides all three, and waives a failing
`[[on.flip]]` gate — read the gate before you waive it. Gates run as `sh -c` with `mt`'s own
environment, so a binary from the project's dev shell is found only when `mt`
ran inside that shell; exit 127 is "binary not found", not a refusal.

`ready`/`blocked` derive from the `links:` graph; `mt ls blocked --columns title,why`
names what each blocked task waits on. Most legends carry no `blocked` status
— check `mt flip --list` before reaching for one. An external blocker with no
task to depend on: file a task for it and depend on that, or park the task in
a pre-start status with the reason in the note
(`mt flip proposed 1b -m "blocked on upstream X"`), never `wip`, which burns
the WIP cap (a `[[tasks.rules]]` entry over `count(...)`).

A project running `flip-with-work` (mindtape-tasks) refuses a `wip`/`done`
flip staged with nothing else: `mt wip <stem>` rides uncommitted into the
first commit of the change, `mt done <stem>` into the last, named by that
commit's `Refs:` trailer. Filing a task is the one commit that flips
nothing and carries no `Refs:` — never fold a flip into it.

= Linking and validating

`mt link <task> <kind> <target…>` upserts one edge per target. `mt link --apply <file|->`
reads a YAML/JSON map of stem to kind to targets; `--replace` makes it the
complete edge set, `--dry-run` prints the plan. Both forms merge and dedupe on
(kind, target), and kind, target and note rule are checked before the write
(`related` needs `-m "why"`).

`mt check` validates every task against the config — enums, required fields,
bounds, links and cycles, tags, body rules, limits — and an `error` breach of
the body/rules also refuses the write commands. Read the error: it names the
field or file. `mt config show --origin` names the pack a rule came from.

= Authoring or editing a task file by hand

What no command writes: the prose body, a framework priority (`ice(...)`) and
the `estimate` dict. The rules and a worked example (stem identity, `x-<ns>` namespacing, edge
helpers, body pitfalls) are in `references/hand-editing.md`. Run `mt check`
after any hand edit.
