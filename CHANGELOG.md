# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Upgrading from 0.2

0.3 is a rewrite, so there is no in-place upgrade. Start over like this.

- The binary is `mt` now. `cargo install mindtape-cli` installs it, and the old
  `mindtape` binary can go.
- A task is one Typst file of its own (`#show: task.with(...)`), not a checkbox
  line with an `#id()`. Nothing converts 0.2 files. Refile each open task with
  `mt add`, or as a batch with `mt import`.
- Config lives in each project's `.mindtape/config.toml`, which `mt init` writes.
  The global `~/.config/mindtape/config.toml` is no longer read. Settings you
  share across projects go in `~/.config/mindtape/defaults.toml`.
- The `watch` daemon, its SQLite index and the NixOS module are gone. Every `mt`
  command reads the files directly, so stop the service and delete the index.
- The Typst package is `@local/mindtape:0.2.0`. `mt install` puts it in place.

### Added

#### Task files

- **Task files are checked field by field.** A misspelled field fails with
  Typst's own `unexpected argument: titel`, placed on that argument so an editor
  marks it in the task file. A wrong-typed value reports the position you can
  count to (`tags[1]`, `estimate.days`, `priority.inputs.ease`, `status.date`,
  `recur.on[1]`), the entries that do read survive, and a wrong-typed required
  field is reported once, by its type.
- **The DSL helpers refuse what they would drop.** `depends-on(link("a.typ"))`,
  a second note block or a named argument to an edge helper (`depends-on`,
  `parent`, `recurred-from`, `supersedes`, `duplicates`, `related`, `edge`), a
  non-content note in a status constructor (`wip(2026, 6, 14, "note")`) and a
  positional input to a framework are eval errors naming the helper, not a
  silently empty value.
- **A load failure on a literal `#` in a body says to write `\#`.** Prose like
  `Channel #N` opens Typst code, so every command that warns about load errors
  adds one note pointing at `\#` or a code block.
- **Typst errors carry their place and hints.** `mt check` prints
  `error: tasks/broken.typ:2:3: unexpected argument: titel` and each Typst hint
  as a `hint:` line, and `mt ls` prints the same line as a `warning:`. An error
  inside the prelude names the task file's line that led into it. A Typst
  warning on a file that loads prints as its own `warning:`, cached or not. A
  missing prelude is one error naming `mt install` and
  `[[typst.local-packages]]`, not one per task.
- **`recur` describes a recurring task.** `recur: every(weeks: 1, on: (mon,
  wed, fri), anchor: "due", until: on(2027, 1, 1), count: 10, materialize:
  "spawn")` is the common shape, and `start: on(2026, 6, 1)` picks which weeks
  a multi-week series runs on. `anchor: "completion"` (the default) advances
  from today, and `anchor: "due"` catches up from a stale due date.
  `rrule("FREQ=MONTHLY;BYDAY=1MO")` covers the rest of RFC 5545, anchored on
  `due`, once `[tasks.recur] rrule = true`. Every build expands it, and
  `mt check` quotes a malformed rule.
- **`mt recur X` files the next occurrence.** `spawn` writes a new file, named
  by `[tasks.naming]` or `{slug}-{date}`, with a `recurred-from` edge to the
  one before it, and `mutate` refreshes the file in place. `--dry-run`
  previews, `--force` spawns a second, `--advance N` spawns ahead up to
  `[tasks.recur] max-advance`, and `--skip` moves the series on without filing
  anything. `mt done` keeps `recur` and points at `mt recur`.
- **A parent tracks its children.** Computed fields `children` and `progress`
  (the finished share, shown as `3/5`) filter, sort and render as columns. A
  status that is both `terminal` and `unblocked = true` needs every child
  finished, so `mt flip` names the open ones and `mt check` warns about a parent
  already closed over them.
- **`waits-on` and `[[tasks.waits]]` let a task wait on something outside the
  repo.** A `[[tasks.waits]]` entry names a command, and a task lists
  `checker:id` entries in `waits-on`. `mt` runs the command with `{id}` filled
  in, once per distinct line per call and only for open tasks. Exit 0 frees the
  task, 1 keeps it `blocked`, and anything else blocks it as unknown. `why`
  shows the command's first line, `mt check` refuses an undeclared checker or an
  id a shell could misread, and `mt check --commits` asks each checker once per
  run.
- **A link kind can point outside the backlog.** `[[tasks.links.kinds]] validate`
  takes `local-task-path` (the default), `url`, `git-sha`, `uuid`, `regex` (with
  a sibling `regex = "..."`) or `any`. An external kind skips stem resolution,
  cardinality, cycles and readiness (`blocking` and `acyclic` don't apply) but
  keeps its note rules, and `mt graph` draws its targets as dashed leaves. A
  `swh:` target is a history pointer, not an edge.
- **Status transitions turn the legend into a state machine.** A
  `[[tasks.status]]` entry's `to = [...]` lists the allowed next statuses, and
  with none declared any move goes. `mt flip` refuses an unlisted move and
  names the allowed ones, and `mt check` flags a status the initial one cannot
  reach. `unblocked = true` makes `mt flip` refuse entering a status while a
  blocking target is unfinished, and `mt check` warns about a task already
  there. The `flow` and `gated` presets set it on `wip` and `done`. `--force`
  overrides both.
- **An estimate unit can count the files its queries match.**
  `estimate: (src-files: count(2, grep(`parse_task`, "src/**")))`
  resolves to 2 plus the files that `grep(...)` or `glob(...)` match, each file
  once, among the files git lists. A unit takes `count(...)` once
  `[tasks.estimate.queries]` names the path set its files must fall in, such
  as `src-files = "@source"`, and rules read `estimate.src-files` as the
  resolved number. `mt check` reports a regex that doesn't compile, and a
  count on a unit with no entry.
- **`mt eval` shows where a `count(...)` estimate comes from.** Under the
  table it prints each such unit's total, its manual part and each query with
  the files it matched, so a query that matches nothing shows as `0 files`.
  `--json` carries the same under `estimate_counts`, beside the resolved
  number in `estimate`.
- **A `count(...)` estimate survives `mt ls --json | mt import`.** The
  canonical JSON carries each such unit under `estimate_counts` in its source
  form, and `mt import` writes it back as the `count(...)` it came from, not
  the number it resolved to on the day of the export.

#### Naming tasks

- **A task is named by its stem, its path or its stem hash.** The `slug#`
  column prints the shortest prefix, three characters or more, of a hash of the
  filename stem that no other task shares. It is the same in every listing and
  changes only when `mt mv` renames the stem. An exact stem wins over a hash, an
  ambiguous hash is refused naming each candidate, and a stem two folders share
  names the one beside the base or is refused. A link target spelled `name.typ`
  is a path from the linking file.

#### Commands

- **`mt flip` sets a status without opening the file.** `mt flip <status>
  <task>...` stamps the status dated today, and every legend status is a verb of
  its own, such as `mt wip a` or `mt done a b`, listed by `mt flip --list`. A
  built-in command wins a name clash, with a warning. `-m/--note` sets the note.
  A refused flip prints the way past as a `hint:` and what it waits on as
  `note:` lines, and a batch ends with `error: flipped 1 of 3 tasks; 2 failed`.
- **`mt set <task> <field>=<value>` writes one core field.** It covers `title`,
  `priority` (a number), `difficulty`, `kind`, `assignee`, `tags`, `due`,
  `start` and `waits-on`, with `mt add`'s `=`, `+=` and `@path` tokens. A field
  with a command of its own (`status`, `links`, `reviewed`, `recur`, the stem)
  is refused naming that command, and a framework priority and `estimate` stay
  hand edits.
- **`mt link` writes `links:`.** `mt link <task> <kind> <target>...` upserts one
  edge per target, and `mt link --apply <file>` (`-` for stdin) reads a YAML or
  JSON map of stem to kind to targets, where a target is a task reference or
  `{target, note}`. Re-applying writes nothing, `--replace` makes the map the
  whole edge set (refused over a SWHID history pointer), and `--dry-run` prints
  the plan. A target is written relative to the linking file, one failure
  doesn't stop the rest, and a write that closes a cycle is refused.
- **`mt mv <task> <new-stem>` renames a task and retargets every inbound link.**
  The file moves through `git mv` when git tracks it, each edge keeps its own
  relative prefix, a taken stem is refused, and `--dry-run` prints the plan.
  The new stem is bare, since moving folders changes derived tags.
- **`mt rm` retires finished tasks.** It deletes each target and rewrites every
  surviving link to it into a SWHID tombstone,
  `link("swh:1:rev:<commit>;path=<path>")`, planned in memory before a byte is
  written. It takes several targets, a folder (`mt rm tasks/sw1`) or a filter,
  and with none it applies `[rm] default-filter`. It refuses a dirty tree,
  `--dry-run` previews, `--yes` skips the prompt (needed without a terminal),
  and it commits nothing. A gitless project uses `--strategy retire` or
  `unlink`. A mistyped name reads `error: no task: 'nope'` with the nearest stem.
- **`mt go` opens a task in your editor.** It reads `[go] editor`, then
  `$VISUAL`, then `$EDITOR`, and a `{task}` token places the path.
- **`mt review` stamps `reviewed:` with today's date** on an open task.
- **`mt eval <task>` takes a stem, path or hash.** It loads the project config,
  so `[[typst.local-packages]]` applies, and a path still evaluates a lone file.
- **`mt find TERM…` searches text.** `mt find login bug` runs
  `mt ls 'contains("login") && contains("bug")'`, takes every `mt ls` flag and
  shows the expression in the `applied:` footer.
- **`mt graph [filter]` draws the dependency graph.** Graphviz DOT by default,
  `--format mermaid` for Mermaid, or `--json` with `nodes`, typed `edges` and
  `external` leaves. Nodes are coloured by readiness, a filter scopes the
  subgraph and `--no-terminal` drops closed tasks.
- **`mt import [FILE|-]` files tasks from JSON.** It reads the array or envelope
  `mt ls --json` prints, maps each record onto the project's legend, scales,
  `kind` values, tags and `x-<ns>` fields, names the file from the title, and
  validates the batch as one set. A bad record is skipped and the rest land.
  `--dry-run` previews, `--into`/`--to` pick the destination, `--status` fills
  a missing status, `--map OLD=NEW` renames a foreign one, and
  `--drop-unmapped` drops a tag outside the allow-list. A dry run that skips a
  record ends `error: would file 1 of 2 tasks; 1 failed`, not `filed`.
  A skipped record's report keeps a content rule's detail on an indented
  `note:` line under the breach, as `mt check` prints it.
  `contrib/jq/taskwarrior.jq` and `contrib/jq/github.jq` turn `task export` and
  `gh issue list --json` output into that JSON, with neutral statuses (`todo`,
  `done`, `cancelled`) and the source's own data kept in `x-tw` or `x-gh`.
- **`mt preset save <name>` and `mt preset ls`.** `save` copies the project's
  `.mindtape/config.toml` and `templates/` into a user preset (`--force`
  overwrites), and `ls` lists built-in and user presets, the shadowed ones and
  the default (`--json` too). A preset's `[overlay]` table holds raw config the
  `mt init` wizard can't ask for, merged last.
- **`mt init` composes a config from presets and axis flags.** The built-in
  presets are `todo-done`, `flow` (the default), `personal` and `gated`. Two
  fragments declaring the same array section are an error naming both.
  `--list-presets` names the real preset directory, `--recurrence rrule`
  composes, `--priority ice-strict` adopts the `mindtape:ice` pack, and an
  answer's packs fold into one `extends` array. The scaffolded config opens
  with a `#:schema` line (`--schema URL|PATH` retargets it, `--no-schema`
  drops it).
- **`mt install` installs the prelude, defaults, schema and shell completions.**
  `--no-lib`, `--no-defaults`, `--no-schema` and `--no-completions` skip a part,
  and `--force` overwrites. A package directory that is a symlink, such as one
  pointing at a checkout's `lib/`, stays as it is unless `--force` replaces it.
- **Tab completion asks the live project.** The hooks under
  `~/.config/mindtape/completions/` call `mt` on each press, so `mt flip `
  offers the legend, `mt ls @` the views, `mt add -t` the templates,
  `mt link <task> ` the edge kinds, `mt init --preset ` the presets, every task
  argument its stems, and `mt set ... tags=` the tags. A project that fails to
  load offers nothing, quietly.
- **`mt allow` approves the config's commands before any run.** No
  `[[tasks.waits]]` checker, no `[[on.flip]]` `run` and no `[go] editor` or
  `[add] editor` runs until `mt allow` records a hash of every such line for
  the repository, shared by its worktrees. Until then a wait reads
  `unknown: not allowed`, a gate refuses with `gate not allowed` (`--force`
  waives it), a `run` without `gate` is skipped with a note, `mt go`, the
  `mt ls` picker and `mt add --edit` try `$VISUAL`, then `$EDITOR`, in place of
  a held editor (`mt go` fails with neither, `mt add` opens `vi`), and `mt`
  warns once listing the commands. A changed line asks again. An editor in your
  own `<config home>/mindtape/defaults.toml` needs no approval.
- **`mt views` lists and inspects views**, with `--json`.
- **`mt dicts` lists the dictionaries**, built-ins included, with each one's
  entry count and case, and `mt dicts <name>` prints its entries one per line,
  after `include`, `values` and `path` are merged. Both take `--json`.
- **`-w/--worktree <path>` and `$MINDTAPE_WORKTREE` point `mt` at another
  project.**
- **`mt @home` reads every project you track in one run.** A bare `mt @home`
  prints one board row per project, `mt @home ls ready` lists their tasks in
  one table grouped by project, and `mt @home check` checks them all. Each
  project loads in parallel under its own config, so 28 projects take about
  as long as the slowest one, and a project whose config won't load is a ⚠
  row instead of a dead run. Writes still go to one project at a time.
- **`~/.config/mindtape/at.toml` names projects and sets of projects.** An
  `[at.<name>]` entry holds `include` (literal paths, one-level `*` globs or
  other `@name`s), `exclude`, and `forks` (`false`, `true` or `"only"`, for
  linked git worktrees). Put `@name` right after `mt`. An entry with one
  literal path is a project, so `mt @sn ls` means `mt -w <path> ls`, and so is
  a directory an `include` matches by name. Two `blog` directories go by the
  shortest path suffix that tells them apart, as `mt @oss/blog ls`, and a bare
  `mt @blog` lists both with the suffix that picks each. Any other entry is a set. Every
  command but `ls`, `find`, `check` and `board` refuses a set before it runs,
  `mt @set at` and `mt @set config schema` included, naming a member to pick. `mt at`
  lists the entries, and `mt at <name>` lists a set's projects in aligned columns, a home path as `~/…`, marking
  forks. Both take `--json`. `mt --help` ends with a line on `@name`.
- **`mt @set check` checks every project of a set.** It prints each project's
  problems together in set order, each naming its project, and ends with
  `ok: 27 projects, 1904 tasks` or `3 of 30 projects failed`, which exits 1.
  A project whose config won't parse fails that way too. `--json` prints one
  object per project, the projects check in parallel, and `--commit-msg` and `--commits` refuse a set.
- **`mt board` shows one row per project**, and a bare `mt @home` runs
  `mt @home board`. Each row counts the project's open, ready and blocked
  tasks and names its next ready one, under that project's own config. The
  columns are `[[board.columns]]` in `at.toml`, each a `filter` count or a
  `top` task, with `sort` and `hide` beside them and `[at.<name>.board]` per
  set. A failed cell prints `?` with a note, a project whose config won't load
  is a `⚠` row and exits 1, `--all` shows the hidden rows and `--json` prints
  them all.
- **`mt _cli --json` describes the CLI**, its filter functions, the filter
  fields and which of them sort, the template slot kinds, the built-in link
  kinds with their rules, the `{date:FMT}` tokens and which commands run on a
  set, for tools and docs.
- **`--json` on every read command**, `mt graph` and `mt preset ls` included.
  `mt check --json` prints a `problems` array (`severity`, `file`, `line`,
  `place`, `rule`, `message`, `hints`) in the `schema_version` envelope.

#### Filing tasks with `mt add`

- **`mt add <title…>` files a task from a template and checks it before writing.**
  It names the file from the title, writes it under the first root and prints
  its slug hash (`wrote tasks/x.typ (ksr)`, `slug` in `--format json`). A task
  the project would refuse is never written. `--print` shows the task after the
  same check, `--no-verify` skips the check for both, and a refused `--edit`
  buffer (opened with `[add] editor`, `$VISUAL` or `$EDITOR`) is parked in a
  temp file named on stderr. A refusal names the line that failed to load and
  hints only at templates that pin the field at fault. `mt add --list` lists
  the templates.
- **One token grammar carries every value.** `key=value` sets, `key+=value`
  appends to a list and `key@path` reads a file, after `--set` or bare after
  `--`, as in `mt add "Fix it" -- estimate.files=2 tags+=docs`. `+word` adds a
  tag. `-f` reads YAML or JSON values (`-f -` for stdin), and `MINDTAPE_ADD_*`
  env vars and the `--edit` buffer fill slots too.
- **Template slots.** `slot(..., example: ...)` shows a sample that
  `mt add --describe` prints, in text and `--describe=json`, and nothing files.
  A `text` or `raw` default or example can be a raw block, and a slot name may
  hold dots (`estimate.files`). A `text` slot escapes Typst markup, a `raw` slot
  splices verbatim, and an unknown `kind:` is an error. A slot that is a whole
  field with config `values` is a choice over them, `multichoice` for a
  `multiple` field. `--describe` refuses a title and suggests `-t`.
- **A code span in a `text` slot files escaped, with a warning** that names the
  slot and suggests `kind: "raw"`.
- **A YAML error in `mt add -f` carries a `hint:`.** A value that starts with a
  YAML indicator such as a backtick needs quotes, or a `|-` block.
- **A template's own `status` is the status `mt add` files it at**, dated today,
  and `--status` overrides it. The `tweak` template files done. Filed done, a
  task asks only for the fields a done task requires.
- **A strict framework priority gets one slot per input.** With
  `allow-manual = false`, where `ice(...)` is the only spelling, `mt add`
  splices `framework("ice", impact: ..., ...)` with a typed slot for each
  declared input, so a strict, required `[tasks.priority]` can still be filed.

#### Listing and filtering

- **`mt @set ls` and `mt @set find` list every project of a set in one
  table.** A `PROJECT` column comes first and rows group by project in set
  order. Each project applies its own default listing, `@view` and sort, and
  `-n` and `--skip` count per project. The footer reads `your filter · 9 of 61
  tasks in 27 projects`. `--json` prints one object per project with its
  `project` and `error`, and CSV gains a `project` column. A project that
  fails to list prints its error naming it after the rest, and `mt` exits 1.
- **`[views.<name>]` names a filter, sort and columns.** `mt ls @broken` applies
  a view whole, `@name` inside a filter (`'@summer && !@broken'`) or
  `in_view("name")` uses its filter, and `--sort @view` and `--columns @view`
  borrow its sort or columns. Views are checked at load, cycles included.
- **Bare `mt ls` picks its view from context.** The first `[[ls.rules]]` entry
  whose `condition` holds wins, using `on_branch(glob)`, `env(name)`,
  `weekday()` and `now()`, with `import-views` and its own overrides, else
  `[ls] default-view`, else the floor's `[views.default]`. `[ls] default-sort`
  sets the fallback sort. A filter argument replaces only the filter, so the
  rule's sort and columns stay, and the `applied:` footer names each facet's
  source, marking a rule that lost every facet as superseded. `mt ls ''` lists
  everything.
- **`-n`, `--skip` and a footer that says what was cut.** A cut table ends with
  `truncated: 20 of 143, next: mt ls -n 20 --skip 20`, and `--json` carries
  `truncated` (`shown`, `total`, `next_skip`, `null` when complete). CSV and the
  path formats stay clean.
  The `next page:` command keeps how you chose the project, so pasting it
  from any directory pages the same listing: `mt @sn ls -n 20 --skip 20`, or
  `mt -w ../x ls -n 20 --skip 20` with a quoted path when it needs one.
  `$MINDTAPE_WORKTREE` stays out, because the environment carries over.
- **The footer counts the task files that failed to load.** `mt ls`, `mt find`
  and their set forms end it with `· 1 broken`, as in `default view · 1 of 2
  tasks · 1 broken`. They still list the rest and exit 0, and `mt check` fails
  on the file.
- **Output formats.** `--format json` prints `{"tasks": [...], "applied": ...}`
  with a `retired` key when `touches()` ran, `--format csv` gives the
  root-relative `file` and stable `id`, and `--format path` or `path0` prints
  bare file paths for `xargs` and `fzf` (`contrib/fzf-mt-go.sh`).
- **Columns.** `slug#`, `assignee`, `why` (what a blocked task waits on, in
  `--json` too), `progress` and `children` join the set. Each
  `[ls.columns.<name>]` shows when its `when` expression holds, using
  `nonempty()`, `contested()`, `named()`, `blocked()`, `tty()` and `format()`,
  and the floor sets every default. `mt eval` and `[[on.flip]]`
  lists render columns the same way.
- **`mt ls -i` (`--interactive`) picks a task.** Type a hash to highlight it,
  `Enter` opens it in the editor, `Ctrl+Enter` opens every listed task, `Space`
  reloads, `Del` clears and `Esc` or `Ctrl+C` leaves. It needs a terminal.
- **`contains()` searches text.** `contains("bug")` sweeps `title`, `id` and
  `body`, `contains("bug", title, body)` names its fields, and
  `[ls] contains-fields` changes the default, with `note` (link notes) and
  `tags` on offer. `has_tag()` still tests a tag.
- **Filters read more of a task.** `status.name`, `status.date`, `status.note`
  and `status.index` filter and sort. `estimate.<unit>` reads one unit, names
  may be kebab-case (`has(x-goals)`), and an `x-<ns>` value compares like any
  field (a dict value takes only `has()` and `miss()`). `has(field, value)`
  tests a list, scalar or dict key, as in `has(tags, "frontend")`.
  `links("<kind>")` tests an edge kind, `ready` and `blocked` read the graph,
  and the root-relative path is `path`.
- **Functions compose.** `word(field, n)` takes the nth word, `matches(field,
  "regex")` tests a pattern, and `in_dict("name", <string>)` looks a word up, so
  `in_dict("verbs", word(title, 1))` checks a title's first word.
- **`count(<expr>)` counts matching tasks** across the loaded set, as in
  `count(status == "wip") <= 1`, evaluated once per command.
- **`changed("<states>", "<glob>"…)` counts changed files.** It reads `staged`,
  `worktree` or `staged+worktree` with include and `!` exclude globs, so
  `changed("staged+worktree", "crates/*/src/**") <= estimate.src-files` is one
  rule. A glob may be `@<dict>`, a path set. A comparison's right side may be a
  field or a count. Git runs read-only, once per command.
- **`touches("<path>")` finds the tasks that shaped a file**, through the `Refs:`
  trailers of the commits that changed it, following renames.
- **`topo` and `blocks` rank the dependency graph.** `topo` is a task's depth
  (0 when nothing blocks it), so ascending is a working order, and `blocks`
  counts the unfinished tasks waiting on it. Both read `depends-on` and any
  kind with `blocking = true`, and both filter and sort, as in
  `--sort topo --sort priority:desc`.
- **`[dicts.<name>]` names a set of strings once.** Sources are `values`, a
  `path` file (one entry per line) and `include` of other dicts, compared under
  `case = "fold"` (default) or `"exact"`. `mindtape-verbs` and
  `mindtape-dev-verbs` ship built in, and the `mindtape-` prefix is reserved.
  `[tasks.tags] allowed` and `[tasks.fields.<name>] values` take `@<dict>`
  entries, and an entry starting with `@` always names a dict.
- **Sorting is consistent.** Every field sorts a missing value last ascending
  and first descending. `--sort` takes every filter field plus the `ready`
  rank, `assignee` and `reviewed` included, and a direction other than `asc` or
  `desc` is an error.

#### Rules and checks

- **`[[tasks.rules]]` holds any convention as a filter expression.** An entry
  has `assert`, plus optional `name`, `when`, `message` and `severity` (`warn` by
  default, or `error`). `check = { typst = "<path>", function = "<name>",
  field = "<field>" }` replaces `assert` for a shape the filter language can't
  name, returning one string per breach. A breach reports as
  `rules.<name>`, or `rules[0]` unnamed, and every expression is dry-run at load.
  A work-in-progress cap is `assert = 'count(<when>) <= 2'`.
- **A rule refused by `in_dict()` says what missed.** Under the rule's own
  `message`, the breach names the word, the dictionary and up to three of its
  nearest entries, as in `hint: 'Login' is not in dict verbs; did you mean
  begin, join or log?`, then `hint: run mt dicts verbs to see its entries`.
  `mt check --json` carries both in `hints`. A call under `!` explains nothing.
- **Every write enforces the rules, not just `mt check`.** `mt add`, `set`,
  `flip`, `link`, `mv`, `import`, `recur` and `review` check the candidate, body
  rules included, and write nothing on an `error` breach. A warning prints and
  the write lands. A batch is checked as one set, a breach it opens on another
  task (a cap, a cycle) is one `note:` naming that task, and a refused rewrite
  leaves the file as it was.
- **Body rules.** `[[tasks.body.headers.rules]]` hold a task's heading tree to an
  `action` of `require`, `forbid`, `unique`, `order` or `allowlist`, and
  `[[tasks.body.content.rules]]` hold a section to `non-empty`, `min-words`,
  `max-words`, `must-match`, `must-not-match` or `density` (`pattern`, `max`,
  `per = "section" | "paragraph" | "sentence"`, code skipped whether fenced or
  inline). Both take a `name`, reported as `body.headers.rules.<name>`.
- **A content rule's `message` keeps the check's own words.** They print on a
  `note:` under the message, such as `section 'TO-BE' sentence 2 ("Should it
  also cover the board?") must carry at most 0 matches`, in `mt check`, in a
  refused write, and in a `notes` array that every `mt check --json` problem
  now carries.
- **Packs.** A config layer's `extends = [...]` pulls in a pack, named
  `<source>:<pack>` or given as a TOML file by path, and `disable = [...]`
  drops rules by name. `mindtape` is the one source known without a
  declaration, and `mindtape:tasks` is the pack built in. An old
  `mindtape-tasks` is refused as unknown, with `mindtape:tasks` as the
  suggestion. A later rule with the same `name` replaces an earlier one, and
  the rule arrays append across layers. `mt config show --origin` names each
  rule's pack or layer.
- **A pack carries any config key.** Its `[tasks.priority]`, views, statuses
  and the rest deep-merge just below the layer that extends it, so the layer's
  `[tasks.priority] allow-manual` keeps the pack's `framework` and the floor
  fills in the rest. Two packs in one `extends` that set one value differently
  fail, naming the dotted key and both packs, unless the extending layer sets
  that key itself. A pack may not declare `extends`, `disable`,
  `pack-sources`, `project.code`, `tasks.roots`, `templates.dirs` or
  `[typst]`. The library's `ConfigError::Invalid` carries a `hint`, which `mt`
  prints as `hint:`.
- **A project declares pack sources.** `[pack-sources.<name>]` names a git
  repository by `git` and `ref`, a tag, branch or commit, and
  `.mindtape/packs.lock` pins each one to a commit and a hash. Every config
  load refuses a source the lock lacks, or pins from another `git` or `ref`,
  as `pack source lock is stale: 'acme'`. Only the project's own
  `.mindtape/config.toml` may declare a source or extend a pack from one, and
  `mindtape` is taken. The library's `config::pack_sources::read_raw` reads
  the sources and the lock without loading the config.
- **A pack can be a directory.** An `extends` path naming a folder reads the
  `pack.toml` in it, and the folder's `templates/` joins `[templates] dirs`
  behind the project's own, so `mt add --list` lists both and a project
  template with the same name wins. The folder may sit outside the project. A
  `team.toml` beside a `team/` folder makes `extends = ["./team"]` ambiguous,
  and `mt` refuses it naming both.
- **A pack's check and dictionary files resolve from its directory.** A pack
  rule's `check` Typst module and a pack dictionary's `path` are read from
  the folder of a directory pack, or the one holding a pack file, not the
  project root. The check runs in a Typst world rooted there, so it imports
  the pack's own files and `@local` packages, never a file of the project. A
  rule or dictionary the project declares itself resolves from the project
  root as before.
- **`[[on.flip]]` hooks merge by name.** A hook takes an optional `name`. Hooks
  from every config layer and pack now add up instead of the nearest layer's
  replacing the rest, a later hook with the same `name` replaces the earlier
  one in place, and `disable = ["<name>"]` drops it. A hook with no `name`
  always appends. `mt config show --origin` labels a hook `on.flip.<name>` or
  `on.flip[<index>]`, and `mt rules` lists its actions under that label, such
  as `on.flip.stale-wip.actions[0]`.
- **A pack takes parameters.** It declares each with a default in `[params]`,
  and its expressions read it as `param("name")`. An `extends` entry
  `{ pack = "./team.toml", with = { max-files = 8 } }` sets the pack's own
  value, and `mt` writes it into every expression of the pack at load, skipping
  string literals and a rule's `message`. `mt` refuses a `with` key the pack
  doesn't declare, suggesting the nearest, a value of another type than the
  default, a `param()` naming no parameter, a `param()` in a layer's own
  expression, and one pack listed twice in one `extends`, even under two
  paths to the same file. A path pack's label is its canonical path. The
  config schema tags each expression key with `x-expression` and its
  language.
- **`mt config show --origin` names the pack behind each key and parameter.**
  After the rule lines it prints `key <path>: <origin>` for each value a pack
  set, naming the layer instead when the layer replaced it, and `param <name> =
  <value>: <set by>, for pack <pack>` for each parameter, where `<set by>` is
  the layer whose `with` set it or `default`. The library's `Config` keeps the
  record: `key_origins()` and `param_origins()`.
- **A config warning points into the pack that holds its key.** A filter in a
  pack's rule that names an undeclared `x-` field now warns at that pack's
  file and line, as `team.toml:6: tasks.rules[0].assert: ...`, not at the
  project config. `Config::warning_place` gives the place.
- **The library fetches a locked pack source into a cache.**
  `config::pack_sources::fetch` runs `git fetch` for the source's `ref`, with
  the caller's `GIT_*` repository variables dropped, fetched objects checked
  and only https, ssh, git and local paths allowed. It writes the locked
  commit's files from git's own objects, so no `core.autocrlf`,
  `.gitattributes` or filter changes a byte, and renames the tree to
  `<cache root>/<sha256>/`. `config::pack_sources::hash` is that sha256, over
  every file framed by its path and length. A symlink, a submodule, a path
  that is not ASCII, has an empty, `.` or `..` part, or names something
  Windows can't hold (`C:x`, `a\b`, `CON`, a trailing dot), two spellings of
  one path in different case, or a hash other than the lock's fails, on every
  platform. A SHA-256 repository fetches by its 64-digit commit, two fetches
  of one tree at once both pass, and a fetch sweeps scratch directories over
  an hour old.
- **A config error names the layer or pack that holds its key.** An unknown
  or wrong-typed key in a pack or an ancestor `defaults.toml` fails as
  `team.toml:3:2: can't parse config: unknown field 'projet'`, and a pack from
  the binary reads `pack mindtape:tasks`. A bad value a pack sets, such as a
  view's filter, is placed at the pack's line, and a pack rule's bad expression
  at its own `[[tasks.rules]]` table, where `mt` used to point at a line of
  `.mindtape/config.toml` that held neither. A key no file declares in full has
  no place, and the error leads with the key alone.
- **A pack from a declared source resolves from the cache.** `acme:commits`
  reads `commits.toml`, or `commits/pack.toml` for a folder pack, in
  `<cache>/mindtape/packs/<sha256>/`, the tree the lock's hash names, and
  `mt config show --origin` labels its rules `pack acme:commits`. Every load
  hashes each source's tree once, before reading it, and refuses one that
  doesn't match the lock, naming its directory. A source the cache lacks fails as
  `pack source not installed: 'acme'`, with a hint to run `mt packs install`,
  and an unknown pack lists the ones the tree holds. A pack name takes letters,
  digits, `-` and `_`, a cached pack's dictionary `path` must stay inside its
  tree, and any pack's `check` module must stay inside the pack's directory.
  The cache is `$MINDTAPE_CACHE_HOME`, else the platform cache directory, such
  as `~/.cache` on Linux. The library adds `config::pack_sources::cache_root`
  and `load_config_layered_in`, which takes the cache, while
  `load_config_layered_from` reads none.
- **`mt packs install` fetches the pack sources the lock pins.** For each
  declared source whose tree the cache lacks it fetches the locked commit,
  checks its hash against the lock and prints `fetched acme 3f2c1a9 into
  <dir>`, then counts the rest as `1 source up to date`. Like `npm ci`, it
  refuses a lock that disagrees with `[pack-sources]`, with a hint to run
  `mt packs update`. It reads the sources without loading the config, writes
  only the cache, and takes `--json`. A relative `git` path is read from the
  project root, which the library's `config::pack_sources::locate` resolves,
  and a drive-relative one such as `C:packs` is refused on every platform.
  Every git command runs in the cache root and searches no further, so no
  repository around the project, nor its config, takes part.
- **`mt packs update` moves pack sources along their `ref`.** It resolves
  each source's `ref` with `git ls-remote`, the peeled commit of an annotated
  tag and a tag before a branch of the same name, or takes a 40- or 64-digit
  `ref` as the commit. It fetches each moved source, rewrites its lock entry,
  and prints `acme v1: 3f2c1a9 → 8d01e44`, `acme v1: locked 3f2c1a9` for a
  new one, or `acme v1: up to date`. It drops the entries no source declares,
  takes source names to update only those, and writes `.mindtape/packs.lock`
  once, atomically, after every fetch passed. It prints its report only after
  that write, so a failed update prints its error alone. The library adds
  `config::pack_sources::resolve` and `write_lock`.
- **`mt packs update` prints what a moved source changes.** Under each moved
  source it lists the packs added (`+`), changed (`~`) or removed (`-`), and
  in a changed pack each key, rule and array entry, such as
  `~ rule tasks.rules.changelog` or `+ on.flip.gate`, compared as parsed TOML
  so a reformatted pack shows nothing. Every new `run` or `editor` line
  prints in full under its entry, and every other file of the source as
  `~ file team/check.typ`. When the old tree isn't cached it fetches it by
  its commit from the old `git`, then from the source's `git`, since a
  source may move hosts. When both fail it warns and lists the new tree as
  all added, so no new command goes unlisted. The library adds
  `config::pack_sources::diff_trees`.
- **`mt rules` lists the rules that apply, one line each.** A line is the
  rule's severity, its name as `mt config show --origin` prints it, and the
  message a breach prints, after packs, overrides and `disable`. A rule with no
  `severity` shows the level `mt` applies, and an `[[on.flip]]` action with a
  `message` is listed too. `--for add|commit|flip` keeps what that step checks,
  and `--json` adds the section, `when`, `assert` and a `check` rule's
  function. The library's `active_rules` returns the same list as
  `ActiveRule` records, and the config schema gives `warn` as the default
  `severity` of `[[tasks.rules]]` and `[[commits.rules]]`.
- **The `mindtape:tasks` pack ships the flagship practice.** It holds the
  section shape (Problem, TO-BE, Changes, Out of scope, plus Environment,
  Alternatives, Source of truth or Rollback by kind), heading order, 🔴/🟢
  markers, word caps, an em-dash density cap, `no-open-questions` (a `?` that
  ends a sentence of an open task's TO-BE, Changes, Out of scope or Dogfood,
  outside code) and `imperative-title`, and each message says the next step.
  Its commit rules are `work-names-a-task`,
  `flip-with-work` (a `wip` or `done` flip rides with its work; a `decision`
  task, a `milestone` or an `mt rm` of other tasks counts) and `named-tasks-finish-at-the-tip`.
- **The `mindtape:wip-caps` and `mindtape:budget` packs ship the WIP caps and
  the size budget.** `wip-caps` holds `milestone-wip-cap` and `leaf-wip-cap`,
  with `milestones = 1` and `leaves = 1` as parameters. `budget` holds
  `judgment-stays-small`, which skips a milestone, and `src-budget`, with
  `min-difficulty = 3` and `max-files = 5` as parameters. It also sets the
  `src-files` estimate unit, its `@source` query and a `source` path set of
  `**`, which a project's own `[dicts.source]` replaces.
- **The `mindtape:tweaks` and `mindtape:commits` packs ship the tweak and commit
  rules.** `tweaks` holds `tweak-lands-done` and `tweak-stays-one-file`, with a
  `source` path set of `**` that equals `mindtape:budget`'s, so a project
  adopts both. `commits` holds `refs-task-is-started` and
  `changelog-with-product-change`, which skips a `fixup!`, `squash!` or `amend!`
  subject and a `Changelog: none` trailer, with a `product` path set of `**`
  that a project replaces.
- **The `mindtape:dogfood` pack ships the Dogfood section, and `mindtape:tasks`
  declares `kind`.** `dogfood` holds `one-dogfood-section`,
  `dogfood-after-out-of-scope`, `dogfood-non-empty`, `dogfood-word-cap` and a
  `section-allowlist` that admits Dogfood, which replaces the tasks pack's
  allowlist when listed after it. `mindtape:tasks` now declares
  `[tasks.fields.kind]` (`feat`, `fix`, `refactor`, `docs`, `test`, `dev`,
  `milestone`, required of an open task) and `[tasks.comments]` with
  `policy = "forbid"`, so a project that extends it alone needs neither.
- **The `mindtape:flow` and `mindtape:ice` packs ship the status legend and
  the ICE priority.** `flow` restates the floor's six statuses with their `to`
  edges and `unblocked` flags, and holds the `stale-wip` hook, with `days = 30`
  as a parameter. A project's own `[[tasks.status]]` replaces the legend whole.
  `ice` sets `framework = "ice"`, `allow-manual = false`, `required` and
  `example` under `[tasks.priority]`, so the floor's scale and bands show
  through.
- **Field rules live in one descriptor.** `[tasks.fields.<name>]` (a project
  field such as `[tasks.fields.x-<ns>]` included), `[tasks.assignee]`,
  `[tasks.tags]`, `[tasks.estimate]` and the numeric scales take `values`,
  `default`, `required`, `example` and `values-when`. `required` is `true`,
  `false` or a filter such as `"!terminal"`. `multiple = true` makes an
  `x-<ns>` field a list, with `values` checked per element. Any other key in
  `[tasks.fields]` fails the load. Tags and estimate units are checked on open
  tasks only, and a filter naming an undeclared `x-` field warns with the
  nearest one.
- **`mt check` checks templates too.** It fills every template with each slot's
  `example`, else its `default`, and reports breaches at the template's file
  and line, the built-in one at `[templates] fallback-severity`. A template
  that fails to load reports like a task file, and a `text` slot whose example
  shows a code span is refused.
- **`mt check` reports on stderr and ends with a count**, such as
  `error: 3 problems in 2 tasks`, so `mt check 2>/dev/null` prints only `ok:`.
  A rule skipped for lack of git points at its own config entry.

#### Commits

- **`mt check --commit-msg <file>` checks a commit from a hook.** The
  `[commits] trailer` (default `Refs`) names tasks by stem, and a path or a
  stem naming no task is refused with the stem to write. `git commit --amend`
  is judged against the replaced commit's parent.
- **`[[commits.rules]]` write commit policy in the filter language.** A rule
  reads `subject`, `refs`, `trailers`, the staged `files` and `tasks_only`, and
  `count(files, "glob"…)` counts files. `scope = "task"` runs per task the
  commit changes or names, through `head(...)`, `staged(...)`, `worktree(...)`,
  `exists`, `named`, `into("s")` and `retires_other`. `scope = "tip"` runs once
  per named task, over the range's newest commit.
- **`mt check --commits <range>` holds past commits to the rules**, each against
  its first parent. A rule reading `worktree(...)` or `changed(...)` is skipped
  and reported, and each problem names its commit.
- **The `mindtape:tasks` pack's `work-names-a-task` skips a `fixup!`, `squash!`
  or `amend!` commit**, since autosquash drops its message.
- **A commit rule reads `in_range`**, true under `mt check --commits` and false
  under `mt check --commit-msg`. The pack's `no-unsquashed-fixup` uses it to
  refuse a range that still holds a `fixup!`, `squash!` or `amend!` commit.

#### Flip hooks

- **`[[on.flip]]` runs actions when a task changes status.** `when` is a filter
  over the task before the write plus `into("status")`. `{ show = "text" }`
  prints, `{ list = "filter or @view", title = "..." }` lists tasks, and
  `{ run = "cmd", gate = true, message = "..." }` refuses the flip when the
  command fails. A gate without `run` refuses outright. `--force` waives a gate
  and records it in the note. Commands run through `sh -c` (`cmd /C` on
  Windows) from the project root with `MINDTAPE_TASK_FILE`, `MINDTAPE_TASK_STEM`
  and `MINDTAPE_TASK_STATUS` set, and `show` and `list` go quiet under
  `--quiet` or a pipe. Read commands run nothing. The `gated` preset lists the
  open work on the way into `wip` and gates the way into `done`.

#### Config and environment

- **A JSON Schema for `.mindtape/config.toml`.** `mt install` writes it,
  `mt config schema` prints it, and a schema-aware editor reads the `#:schema`
  line `mt init` writes. Its descriptions read as user docs.
- **A JSON Schema for `at.toml`.** `mt install` writes `at.schema.json` beside
  `config.schema.json`, so a `#:schema ./at.schema.json` line gets your editor
  completing `include`, `exclude`, `forks` and the `[board]` keys.
- **Config layers.** `~/.config/mindtape/defaults.toml`, ancestor
  `.mindtape/defaults.toml` files and the project config merge in order.
- **`$MINDTAPE_CONFIG_HOME`, `$MINDTAPE_DATA_HOME` and `$MINDTAPE_CACHE_HOME`
  move the per-user directories** on every platform, for reading, `mt install` and package
  loading alike. Set but empty means none.
- **`$MINDTAPE_NOW` pins today**, as a `YYYY-MM-DD` date or an RFC 3339
  timestamp, for every stamp, `now()`, `stale(N)` and `mt recur`. A malformed
  value is an error, and `$SOURCE_DATE_EPOCH` is ignored.
- **A content-hash cache under `.mindtape/cache/`** skips evaluating unchanged
  task files. A change to the file, a module it imports, the config, the prelude
  or `mt` re-reads it, and `MINDTAPE_NO_CACHE=1` bypasses it. Read commands
  fill the cache but never change the authored tree.
- **Writes come out typstyle-formatted.** `[format] enabled = false` keeps raw
  splices.
- **Windows binaries.** Releases attach `mindtape-x86_64-pc-windows-msvc.exe`
  beside the Linux and macOS builds, and CI tests on Windows. `mt add --edit`
  falls back to `notepad` there.

#### Docs and skill

- **Every docs page has a Markdown copy**, and `/llms.txt` lists them all. The
  skill points agents at it.
- **The docs open on the problems Mindtape solves**, using the
  `examples/api-service` backlog, with a page per problem. The sidebar groups
  pages by job, and terms are defined at first use.
- **New pages.** One per `mt` command under `/cli/`, `/task-files/` for the
  Typst in a task file and its common mistakes, `/packs/`, rendered
  graphs of the links and the legend, and the language server's completions and
  diagnostics on a task file.
- **The docs build their lists from the code.** The filter fields, link kinds,
  slot kinds, `at.toml` keys, the shipped board and the commands that run on a
  set all render from what `mt` itself knows, and the docs build fails when a
  page names a key or value `mt` doesn't. The schema page shows the real JSON
  of a set and of the board, and `mt --help`, the quickstart and the sidebar
  lead to reading many projects with `mt @name`.
- **The agents page names the skill installers**, `gh skill install` and
  `npx skills add`.
- **The skill files verb-first titles** and leads `-f` with YAML.

#### Library

- **The `mindtape` crate is the domain library behind `mt`**, over seven
  crates.io crates (`typst-world`, `typst-harvest`, `typst-cst`, `typst-edit`,
  `typst-cleanup`, `swhid-mint`, `artifact-gate`). Areas live in folders
  (`mindtape::filter`, `config`, `commit`, `graph`, `model`, `loader`, `body`,
  `validate`, `template`, `recur`, `emit`, `link`, `git`), with root re-exports.
  Public entry points include `check_candidate`, `check_candidates`,
  `check_body_rules`, `BodyRules`, `Checked::elsewhere`, `resolve_ref`
  (`Base::Cwd` or `Base::Linker`, `Resolved`, `RefError::Ambiguous`,
  `RefError::NoMatch`), `relative_target`, `Severity`, `Violation` with its
  optional severity and `ViolationKind`, `write_atomic`, `shell`,
  `config_schema_json`, `map_canonical`, `stem_for_title`, `emit_task`,
  `package_overrides`, `package_root` and `sort::sort_fields`. `LoadError`
  carries `line`, `column`, `hints`, `place()` and `region`, the `CallRegion`
  that `TaskCallSpans::region_at` gives its point. `LoadOutcome` carries
  `warnings`, `ConfigError::Invalid` carries a `ConfigPlace`,
  `naming::disambiguate` returns `NamingError::Exhausted` past 10,000 tries, and
  recurrence catch-up returns `RecurError::CatchUpExhausted`.
- **`mindtape::load_one(root, config, path)` loads one task file.** It reads
  through the same content-hash cache as `load_project`, annotates waits the same
  way, and returns `Ok(Some(task))`, `Ok(None)` for a file with no `task.with`,
  or the file's `LoadError`. A path outside every task root, or not a `.typ`
  file, is an error. `topo`, `blocks`, `children` and `progress` stay empty,
  since they need the whole set.

### Changed

- **One error format.** Every stderr line starts with `error:`, `hint:`, `note:`
  or `warning:`, coloured on the prefix in a terminal, with typed values in
  single quotes and near misses as `hint: did you mean x?`. A wrong command
  line, an unknown field or a filter that doesn't parse exits 2. Rust's `Error:`
  and `Caused by:` lines are gone, and each failure is reported once, with no
  summary of its own. Outside a project, `mt` names `mt init` and `-w <path>`.
- **Config errors name their file and line**, as in
  `error: .mindtape/config.toml:5: views.broken.sort: unknown sort field: 'nofield'`,
  for every layer and pack. An unknown status in a filter or view fails and
  suggests the nearest one.
- **Filter errors say what to do**, as in `a '(' is never closed` or
  `'~' isn't an operator` with a `matches()` hint, and show no Rust type.
- **Today is the local calendar date** (`$TZ` on Unix), not UTC, unless
  `$MINDTAPE_NOW` pins it.
- **`--help` explains each flag in a sentence or two**, and the name is spelled
  Mindtape throughout.
- **`cargo install mindtape-cli` builds faster**, with thin LTO.
- **`--hyperlinks` belongs to `ls`, `find` and `flip`**, the commands that
  print a task table, so write it after the command, as in
  `mt ls --hyperlinks always`. `mt board` and `mt at` no longer list it, and its
  help and the `[ls] hyperlinks` schema text name the `slug#` and `file` columns.
- **The config schema descriptions read as plain sentences**, with no em dashes
  and no `[section]` lead-in. `mt config schema`, the hover text in an editor
  and the Configuration page of the docs site now share one voice.

### Removed

- **`search()`**, replaced by `contains()`.
- **The implicit `lib/` package.** A project whose `lib/typst.toml` named the
  package `mindtape` resolved `@local/mindtape` from there with no entry. Pin
  it with `[[typst.local-packages]]` instead, as this repository does.
- **Spellings from 0.3's development.** They never shipped and have no alias.
  The gone ones are `[handle] check-len` (handles are stem hashes now),
  `[[tasks.limits]]` (use `count()` in a rule), `[tasks.estimate.drift]` and
  `[[tasks.estimate.max]]` or `min` (use a rule with `estimate.<unit>` or
  `changed()`), `[ls] default-filter` (use `default-view`), `mt ls --all` (use
  `mt ls ''`), `mt install-lib`, `key[]=value`, `required = "non-terminal"`,
  the `file` filter field (use `path`), `has(depends_on)`, the `typst` slot
  kind (use `raw`), `[ls.columns.<name>] visibility` and `rules` (use `when`),
  the `rrule` cargo feature, `mindtape::sort::SORT_FIELDS` and
  `LimitSeverity`.

### Fixed

- **Task files in a folder load on Windows.** Each one failed with `bad
  virtual path "tasks\\alpha.typ": path contains backslash`, so a project
  with a `tasks/` root loaded nothing there. `mt` now names every file to
  Typst with `/`, as on Linux and macOS.
- **A tombstone written on Windows resolves everywhere.** `mt rm` wrote its
  `path=` with `\`, as in `swh:1:rev:…;path=tasks\gone.typ`, and git paths
  use `/`. It writes `/` now.
- **A failed read ends without a stray period on Windows.** Windows ends its
  reasons with one, so `can't read: 'a.json': the system cannot find the file
  specified.` had it mid-sentence.
- **A bare `Refs:` value is the stem as written.** `Refs: foo.task-261023`
  read as `foo`, so `mt check --commits` said it named no task and
  `touches()` matched nothing. Only a value holding `/` or `\`, or ending in
  `.typ`, reads as a path and loses its extension.

- **`mt install` puts the prelude where Typst and tinymist look, on Windows
  too.** It wrote under `%LOCALAPPDATA%\typst\packages` while both read
  `%APPDATA%\typst\packages`, so tinymist never found it. `mt install` and task
  files now share the directory typst-world resolves for Typst, and a set
  `$TYPST_PACKAGE_PATH` moves it on every platform, as it does for `typst`. On
  Windows, run `mt install` once more and delete the old copy.
- **`mt find` names a task file that fails to load.** It used to skip the file
  without a word, while `mt ls` warned. Both print the same warning now, and
  `mt find` still lists the rest and exits 0.
- **A misspelled status helper points at the statuses, not at `#`.**
  `status: finished(…)` used to get the advice about a literal `#` in a task
  body. Now an unknown name in `status:` hints `run mt flip --list to see the
  statuses`, any other unknown name in the `task.with(...)` header gets no
  hint, and only one outside the header keeps the `#` advice.
- **A hint inside a set runs in the project it came from.** Under blog's
  error, `mt @work check` used to hint `run mt flip --list to see the
  statuses`, which ran in whatever project you stood in. Now every problem a
  member prints names its `mt` commands with the member's `@name`, as in
  `run mt @blog flip --list to see the statuses`. In `mt @work ls @nope`, a
  project that lacks the view hints `run mt @blog views to see the views`, and
  a member's warning about commands you haven't allowed ends in `mt @blog
  allow` and `note: project: blog`. A command that reads no project, such as
  `mt install`, stays bare.
- **A hint under `mt @blog` or `-w` runs on that same project.** `mt @blog
  check` used to hint `run mt flip --list to see the statuses`, which fails
  typed where you stand. Now a hint's `mt` command carries the `@blog` or
  `-w <dir>` you gave, as in `run mt @blog flip --list to see the statuses`. A
  run inside the project, or one `$MINDTAPE_WORKTREE` points at, keeps its
  hints bare, and so does a command that reads no project, such as `mt preset
  ls`.
- **A second `mt init` fails.** Over an existing config it used to print a note
  and exit 0, so a script never knew nothing was written. Now it exits 1 with
  `error: config already exists` and the hint to pass `--force`.
- **A mistyped slot kind lists every kind `mt add` accepts.** The error left
  out `multichoice`, so a template author who wrote `multi-choice` was told
  to pick from ten kinds when there are eleven.
- **`mt ls --format csv` ends with one newline.** It used to print a blank
  line after the last record, so a script that read the listing saw an empty
  last row. An empty listing is now `id,title` and one newline.
- **`mt ls --format path` prints nothing when no task matches.** It used to
  print a lone newline, so `mt ls blocked --format path | wc -l` said 1.
- **A reader that stops reading no longer panics `mt`.** `mt ls | head -3`
  ends cleanly, and a real write error such as a full disk still fails.
- **A killed write can't leave a task file empty or torn.** Writes go through
  a temp file and a rename, and `mt add` never overwrites a file that appeared
  meanwhile.
- **`mt rm` writes retargeted links with `/` on Windows**, and stops suggesting
  a commit outside a git repository.
- **`mindtape-cli` builds from its published tarball**, carrying its own copy
  of the prelude.
- **An injected `status:` brings its own comma**, so a one-line template files
  a task that parses.

### The rewrite (2026-06-14)

#### Added

- **Per-file task model** — one task per file via `#show: task.with(...)`, with
  a `(name, date, note)` status, configurable enum/number core fields, task
  links, tags, and an `x-<ns>` extension seam for downstream tools
- **`typst-harvest` crate** — a package-agnostic, eval-only Typst World plus
  labelled-metadata-marker harvesting; a reusable substrate for file-based tools
- **`.mindtape/config.toml`** — per-project status lifecycle, enum/number field
  constraints (required-unless-terminal, bounds), tag allow-list, default filter
- **`mt check`** — config-driven validation incl. link resolution
- **`mt install`** — installs the Typst library (once per machine)
- **`mt init`** — scaffolds project config (once per project)
- Dev tooling: parallel `just check`, sccache, cargo-machete, linecop, outdatty

#### Changed

- Split into a three-crate workspace: `typst-harvest`, `mindtape` (library),
  `mindtape-cli` (binary). The filter language now runs over live-evaluated
  tasks instead of SQL
- Typst library bumped to 0.2.0 (new task DSL; breaking)

#### Removed

- The `watch` daemon, the SQLite index, the `mindtape-store` crate,
  eval-hash skipping, the global cross-project agenda, inline checkbox markers
  and `#id()` identity, and the NixOS service module

### The agenda era (2026-03-23)

#### Added

- **Global agenda filter** — `[agenda]` now supports a top-level `filter` field
  that is AND-ed into every task section, e.g. hide tasks with a future
  `#start()` date across all sections
- **Default `#start()` filter** in the built-in agenda: tasks whose start date
  hasn't arrived are automatically hidden

#### Changed

- **Agenda config restructured** — `[[agenda]]` sections are now nested under
  `[agenda]` as `[[agenda.sections]]`, enabling agenda-level settings alongside
  per-section configuration (breaking change to config format)

#### Fixed

- Resolve workspace version for naersk Nix build

## [0.2.0] - 2026-03-16

### Added

- **`mindtape agenda`** subcommand — configurable Typst-rendered agenda with
  pluggable sections (task list, file index)
- **File index section** for the agenda, showing indexed files and their status
- **`mindtape inspect`** improvements — comfy-table rendering, `--format typst`
  output, `~`-shortened paths, and file indexing error tracking

### Fixed

- Exclude imported duplicate tasks from query results
- Exclude nested sub-items from parent task title during evaluation
- Clear stale file errors on unchanged content hash and prune deleted files on scan
- Remove incorrect header from `mindtape list` output, adjust header levels

### Changed

- CI: use stable Rust toolchain, guard unnecessary workflow runs, allow manual
  workflow dispatch

## [0.1.0] - 2026-03-07

Initial release.

### Added

- **Typst-based task format** with `#due()`, `#start()`, `#id()`, `#tag()`,
  `#rank()` metadata functions and `#high`, `#medium`, `#low` aliases
- **Typst evaluation engine** (`mindtape-eval`) — extracts structured task data
  from `.typ` files using `typst-eval`, with AST-based write-back support
- **SQLite store** (`mindtape-store`) — indexes tasks into a queryable database
  with FTS5 full-text search, schema migrations, and a `Store` trait abstraction
- **File watcher** with `.gitignore`-aware filtering, hot-reload config, and
  cross-file dependency tracking
- **CLI commands**:
  - `mindtape <file>` — evaluate a single Typst file
  - `mindtape watch` — watch folders and build the index
  - `mindtape list` — query tasks with `--filter` expressions, `--sort`, and
    `--format` (table/json/csv/typst) output
  - `mindtape check <id>` — toggle task checkboxes (writes back to `.typ` files)
  - `mindtape set <id>` — update task properties (due, start, rank, tags)
  - `mindtape inspect` — show index stats, files, and dependency graphs
  - `mindtape id` — generate and convert base62-encoded UUIDv7 task IDs
  - `mindtape init` — install the Typst prelude library
  - `mindtape eval` — evaluate filter expressions
  - `mindtape deps` — show file dependencies
- **Filter expressions**: `has(due)`, `miss(tag)`, `has_tag("backend")`,
  `search("auth")`, comparisons on `due`, `start`, `rank`, and boolean combinators
- **NixOS module** for declarative service configuration
- **Cross-platform release binaries** (Linux x86_64/aarch64, macOS x86_64/aarch64)

[0.2.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.2.0
[0.1.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.1.0
