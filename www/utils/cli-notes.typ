// Prose and captures for the CLI page, which `mt _cli --json` does not include.
// cli-ref.typ merges each entry into the command tree by dotted path
// (`command`, `command.arg-id`, `nested.command.arg-id`), and cli-gate()
// fails the build on a key that names no command or argument.
//
// Fields per key: `prose`; `examples`, the slug of a capture whose commands
// render as a block to copy; `outputs`, the slugs of captures that render as
// transcripts. www/generate-cli.sh makes every capture named here.

#import "config-ref.typ": config-keys-table
#import "shipped-board.typ": shipped-board
#import "site.typ": u

#let at-schema = json("../generated/at-schema.json")

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
      `--json` is the shorthand for `--format json` and wins when both are
      given. The JSON says whether `mt` wrote, skipped or suffixed the task,
      and opens with the URL of its schema. A `text`
      report ends on `next: mt eval <slug>`, the command that reads the task it
      named. The other formats contain no prose. `mt`
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
      #link(u("/derived-tags/"))[derived tags]. Before it renames anything, it
      refuses a stem another task already has, or a file that already exists.
      Inside a git repository it renames with `git mv`, so the rename is
      staged and git keeps the file's history. Outside git, or for an
      untracked file, it renames the file directly. `mt mv` rewrites every
      edge that points at the task in the same pass, and each edge keeps its
      relative prefix: `../sub/old.typ` becomes `../sub/new.typ`. `mt` checks
      every change before it writes.],
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
      already holds writes nothing, so you can safely run it again. A write
      ends on `next: mt eval <task>`, the command that reads it back.],
    outputs: ("set",),
  ),
  "eval": (
    outputs: ("eval",),
    prose: [When an estimate uses `count(...)`, `mt eval` prints the manual
      count, then each query with the files it matched, then the total.
      `--json` puts the same data under `estimate_counts`, beside the
      resolved number in `estimate`. See #link(u("/estimates/"))[Estimates].],
  ),
  "go": (
    prose: [`mt go` finds the task's file and opens it with `[go] editor`, or
      `$VISUAL`, or `$EDITOR`. If none is set, it fails and asks you to set
      `[go] editor`. `mt` replaces a `{task}` token in the command with the
      path, or appends the path when there's no token. A project's
      `[go] editor` waits for #link(u("/cli/allow/"))[`mt allow`]; until then
      `mt go` tries `$VISUAL`, then `$EDITOR`, and fails with neither.],
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
    prose: [A column is a `[[board.columns]]` entry in `at.toml`. A `filter`
      column counts the tasks it matches, and a `top` column shows the first
      match in the project's default order. Every filter runs under that
      project's own config. A cell that fails prints `?` with a note under the
      table, and a project whose config won't load is a `⚠` row, after which
      `mt` exits 1. This is the board `mt` ships:

      #shipped-board

      `[board]` in `at.toml` changes the board for every entry, and
      `[at.<name>.board]` changes it for one, key by key over the shipped
      board:

      #config-keys-table(at-schema, "board")

      Each `[[board.columns]]` entry takes these keys:

      #config-keys-table(at-schema, "board.columns")],
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
      #link(u("/concepts/#identity"))[stem] to kind to targets, shown on
      #link(u("/links/#the-apply-map"))[Task links]. Both forms merge. An edge with the same
      kind and target isn't added twice, so running either again writes
      nothing new. With `--apply`, `--replace` uses the map as the edge set
      of each stem in the map. `mt` checks the kind and the target before it
      writes. A kind that requires a note, such as the built-in `related`, is
      refused without one. When one entry fails, `mt` reports it and writes
      the rest. `--dry-run` prints the plan instead.],
    outputs: ("link",),
  ),

  // ── Explain ─────────────────────────────────────────────────────────────────
  "explain": (
    prose: [Bare, it prints the project's index: the rules grouped by config
      section, the derived fields, the tags, views and dictionaries, and each
      dictionary's entry count. `default` leads the view and tag lists when the
      project declares it. `--json` leads with `$schema`: the rules, tags and
      index keep their keys, and views and dicts sit under `views` and
      `dicts`. A blank line separates the index from a `next:` line naming the
      optional category and JSON flag:
      `mt explain [<rules|tags|views|dicts>] [--json]`.],
    outputs: ("explain",),
  ),
  "explain.rules": (
    outputs: ("rules-commit",),
    examples: "rules-examples",
  ),
  "explain.tags": (:),
  "explain.views": (outputs: ("views",)),
  "explain.dicts": (outputs: ("dicts",)),

  // ── Status ──────────────────────────────────────────────────────────────────
  "flip": (
    outputs: ("done", "flip-list"),
    examples: "flip-examples",
  ),
  "review": (
    prose: [A stamp ends on `next: mt eval <task>`, the command that reads the
      task back.],
    examples: "review-examples",
  ),

  // ── Recurring ───────────────────────────────────────────────────────────────
  "recur": (
    outputs: ("recur",),
  ),

  // ── Validate & inspect config ───────────────────────────────────────────────
  "check": (outputs: ("check",)),
  "schema": (
    prose: [`mt schema` prints the JSON Schema of a machine output. Bare, it
      lists each output, its version and the URL its `$schema` names.
      `mt schema <name>` prints one, and `--write` records every schema under
      `schema/output/` in the current directory. `--check` fails, names every
      missing or changed copy, and writes nothing. It works outside a
      project.],
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
  "fix": (:),
  "install": (
    prose: [`--no-lib`, `--no-defaults`, `--no-schema` and `--no-completions`
      each skip one part. The generated JSON Schemas are refreshed on every
      install, so a newer binary's schema reaches the installed copy without a
      flag. `--force` overwrites an existing defaults file or hook. It also
      replaces a package directory that is a symlink,
      which `mt install` otherwise leaves alone, so it never writes into the
      checkout the link points at. `mt` writes one completion hook per shell under
      `~/.config/mindtape/completions/`. Source the one for your shell from
      its startup file. The hook asks `mt` for completions on each tab press,
      so it offers this project's #link(u("/concepts/#identity"))[stems],
      statuses, #link(u("/named-sets/#views"))[views], templates and link
      kinds, and the presets for `mt init --preset`. `mt install` copies the
      prelude into #link(u("/config/#local-typst-packages"))[Typst's package
        directory], so run it again after you upgrade `mt`.
      That rewrites the prelude to match the binary. The completion hooks ask
      `mt` on each tab press, so they stay current without it.],
  ),
  "allow": (
    prose: [Until you allow them, each command that would run one warns once
      and lists them. A task that waits on a checker stays blocked, its `why`
      reading `unknown: not allowed`, and `mt wip --force` still starts it. A
      flip gate refuses the flip, and `--force` takes it past. A flip `run`
      without `gate` is skipped. An editor that isn't allowed is skipped too.
      `mt go`, the `mt ls` picker and `mt add --edit` try `$VISUAL`, then
      `$EDITOR`. With neither, `mt go` fails and `mt add` opens `vi`. `mt allow` prints each command and records a hash of them all in
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
  "packs": (
    prose: [`mt packs install` fetches each
      #link(u("/packs/#packs-from-a-git-source"))[pack source] that
      `.mindtape/packs.lock` pins and your cache lacks. It refuses a lock
      that disagrees with `[pack-sources]`, so a clone or a CI job gets the
      packs you reviewed. It writes only the cache, never your project.
      `mt packs update` sets each source's lock entry to the commit its `ref`
      points to
      and rewrites the lock, the one file it writes.],
  ),
)
