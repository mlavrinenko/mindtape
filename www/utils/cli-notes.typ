// Prose and captures for the CLI page, which `mt _cli --json` does not carry.
// cli-ref.typ merges each entry into the command tree by dotted path
// (`command`, `command.arg-id`, `nested.command.arg-id`), and cli-gate()
// fails the build on a key that names no command or argument.
//
// Fields per key: `prose`; `examples`, the slug of a capture whose commands
// render as a block to copy; `outputs`, the slugs of captures that render as
// transcripts. www/generate-cli.sh makes every capture named here.

#import "site.typ": u

#let cli-notes = (
  // ── Files ───────────────────────────────────────────────────────────────────
  "add": (
    prose: [`mt add` fills a template, names the file and checks the task,
      so you don't copy and edit a task file by hand. The title's words come
      first. Fill the template's other slots with `key=value`, `key+=value`
      and `key@path` tokens, after `--set` or bare after `--`. A YAML or JSON
      file given to `-f` fills them too, and so does the editor.
      #link(u("/templates/"))[Templates & slots] describes all three.
      `--dedup` decides what happens when a task with the same
      #link(u("/concepts/#identity"))[stem] exists. `fail` stops before
      writing, `skip` writes nothing and exits 0, and `add-suffix` adds `-2`,
      `-3` and so on. Your project sets the default in
      #link(u("/config/"))[`[tasks.naming] dedup`]. If it doesn't, the default
      is `add-suffix` when a `stem` template is set and `fail` otherwise.
      `--format` prints the result as `text`, `path`, `path0` or `json`, and
      the JSON says whether `mt` wrote, skipped or suffixed the task. `mt`
      checks the task before it writes, so a refused add leaves no file.
      `--no-verify` skips the check.],
    outputs: ("add",),
  ),
  "import": (
    prose: [To bring tasks in from another tracker, see
      #link(u("/import-export/"))[Import & export]. It describes the JSON
      format, `jq` adapters for Taskwarrior and GitHub Issues, and what a round
      trip keeps.],
    examples: "import-examples",
  ),
  "mv": (
    prose: [`NEW_STEM` is a bare stem. `mt` drops a trailing `.typ` and
      refuses a path separator. `mt mv` renames a task in place, because
      moving it to another folder would change its
      #link(u("/derived-tags/"))[derived tags]. Before anything moves, it
      refuses a stem another task already has, or a file that already exists.
      Inside a git repository it renames with `git mv`, so the rename is
      staged and git keeps the file's history. Outside git, or for an
      untracked file, it renames the file directly. `mt mv` rewrites every
      edge that points at the task in the same pass, and each edge keeps its
      relative prefix: `../sub/old.typ` becomes `../sub/new.typ`. `mt` checks
      the whole set before it writes.],
    outputs: ("mv",),
  ),
  // ── Fields ──────────────────────────────────────────────────────────────────
  "set": (
    prose: [`mt set` uses the same syntax as `mt add --set`. A field that has
      its own command is refused, and the error names that command: `status`
      (`flip`), `links` (`link`), `reviewed` (`review`), `recur` (`recur`) and
      the id (`mv`). A
      #link(u("/priority/"))[framework priority] such as `ice(...)` and an
      #link(u("/estimates/"))[`estimate`] aren't single values, so `mt set`
      doesn't write them. Edit those in the file. Adding a value a list
      already holds writes nothing, so you can safely run it again.],
    outputs: ("set",),
  ),
  "eval": (outputs: ("eval",)),
  "go": (
    prose: [`mt go` finds the task's file and opens it with `[go] editor`, or
      `$VISUAL`, or `$EDITOR`. If none is set, it fails and asks you to set
      `[go] editor`. `mt` replaces a `{task}` token in the command with the
      path, or appends the path when there's no token.],
  ),

  // ── Listing ─────────────────────────────────────────────────────────────────
  "ls": (
    outputs: ("ls",),
    examples: "ls-examples",
  ),
  "find": (
    prose: [`mt find login bug` runs
      `mt ls 'contains("login") && contains("bug")'`. So every `ls` flag
      works, and the `applied:` line shows the filter. `[ls] contains-fields`
      sets the fields a bare `contains()` searches, as
      #link(u("/filters/"))[Filters] describes. To search one field, use a
      filter: `mt ls 'contains("bug", title)'`.],
    examples: "find-examples",
  ),
  "ls.sort": (
    examples: "ls-sort-examples",
  ),
  "ls.columns": (
    examples: "ls-columns-examples",
  ),
  "board": (
    prose: [Each column is a `[[board.columns]]` entry in `at.toml`. A
      `filter` column counts the tasks it matches, and a `top` column shows
      the first match in the project's default order. Every filter runs under
      that project's own config. A cell that fails prints `?` with a note
      under the table, and a project whose config won't load is a `⚠` row,
      after which `mt` exits 1. `hide` drops a row, `sort` orders them, and
      `[at.<name>.board]` overrides `[board]` for one set.],
    examples: "board-examples",
  ),

  // ── Graph ───────────────────────────────────────────────────────────────────
  "graph": (
    examples: "graph-examples",
  ),

  // ── Links ───────────────────────────────────────────────────────────────────
  "link": (
    prose: [`mt link <task> <kind> <target>...` adds one edge per target.
      `mt link --apply <file>`, or `-` for stdin, reads a YAML or JSON map of
      #link(u("/concepts/#identity"))[stem] to kind to targets, described on
      #link(u("/dsl/"))[Task DSL]. Both forms merge. An edge with the same
      kind and target isn't added twice, so running either again writes
      nothing new. With `--apply`, `--replace` makes the map the whole edge
      set of each stem it names. `mt` checks the kind and the target before it
      writes. A kind that requires a note, such as the built-in `related`, is
      refused without one. When one entry fails, `mt` reports it and writes
      the rest. `--dry-run` prints the plan instead.],
    outputs: ("link",),
  ),

  // ── Views ───────────────────────────────────────────────────────────────────
  "views": (outputs: ("views",)),
  "dicts": (outputs: ("dicts",)),

  // ── Status ──────────────────────────────────────────────────────────────────
  "flip": (
    outputs: ("done", "flip-list"),
    examples: "flip-examples",
  ),
  "review": (
    examples: "review-examples",
  ),

  // ── Recurring ───────────────────────────────────────────────────────────────
  "recur": (
    outputs: ("recur",),
  ),

  // ── Validate & inspect config ───────────────────────────────────────────────
  "check": (outputs: ("check",)),
  "rules": (
    outputs: ("rules-commit",),
    examples: "rules-examples",
  ),
  "config": (
    outputs: ("config-origin",),
    examples: "config-examples",
  ),

  // ── Retire ──────────────────────────────────────────────────────────────────
  "rm": (
    prose: [`TARGET` takes any number of entries, each a task reference or a
      filter. `mt rm` retires every task any entry matches, in one pass, and
      asks you to confirm once.],
    outputs: ("rm",),
  ),

  // ── Setup ───────────────────────────────────────────────────────────────────
  "init": (
    examples: "init-examples",
  ),
  "install": (
    prose: [`--no-lib`, `--no-defaults`, `--no-schema` and `--no-completions`
      each skip one part. `--force` overwrites an existing defaults file,
      schema or hook. `mt` writes one completion hook per shell under
      `~/.config/mindtape/completions/`. Source the one for your shell from
      its startup file. The hook asks `mt` for completions on each tab press,
      so it offers this project's #link(u("/concepts/#identity"))[stems],
      statuses, #link(u("/named-sets/#views"))[views], templates and link
      kinds, and the presets for `mt init --preset`. `mt install` copies the
      prelude into the data directory, so run it again after you upgrade `mt`.
      That rewrites the prelude to match the binary. The completion hooks ask
      `mt` on each tab press, so they stay current without it.],
  ),
  "allow": (
    prose: [Until you allow them, each command that would run one warns once
      and lists them. A task that waits on a checker stays blocked, its `why`
      reading `unknown: not allowed`, and `mt wip --force` still starts it. A
      flip gate refuses the flip, and `--force` takes it past. A flip `run`
      without `gate` is skipped. `mt allow` prints each command and records a hash of them all in
      `allowed/` under `mt`'s data directory, outside the repo, so no clone
      can approve its own commands. The record belongs to the repo's git
      directory, so every worktree of it shares one approval. Change a command
      and `mt` asks again.],
    outputs: ("allow",),
  ),
  "at": (
    examples: "at-examples",
  ),
  "preset": (
    examples: "preset-examples",
  ),
)
