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
    prose: [The words of the title come first. `key=value`, `key+=value` and
      `key@path` tokens, after `--set` or bare after `--`, fill the template's
      other slots, as do a YAML or JSON file given to `-f` and the editor;
      #link(u("/templates/"))[Templates & slots] describes them. `--dedup`
      decides what happens when a task with the same
      #link(u("/concepts/#identity"))[stem] exists: `fail`
      stops before writing, `skip` writes nothing and exits 0, and
      `add-suffix` adds `-2`, `-3` and so on. The project's default is
      #link(u("/config/"))[`[tasks.naming] dedup`]. Without it, the default is
      `add-suffix` when a `stem` template is set and `fail` otherwise.
      `--format` prints the result as `text`, `path`, `path0` or `json`, and
      the JSON says whether `mt` wrote, skipped or suffixed the task. `mt`
      checks the task before it writes, so a refused add leaves no file.
      `--no-verify` skips the check.],
    outputs: ("add",),
  ),
  "import": (
    prose: [#link(u("/import-export/"))[Import & export] describes the JSON
      format, `jq` adapters for Taskwarrior and GitHub Issues, and what a round
      trip keeps.],
    examples: "import-examples",
  ),
  "mv": (
    prose: [`NEW_STEM` is a bare stem: `mt` drops a trailing `.typ` and refuses a
      path separator. `mt mv` renames a task in place, because moving it to
      another folder changes its #link(u("/derived-tags/"))[derived tags]. It
      refuses a stem another task already has, or an existing file, before
      anything moves. Inside a git repository it renames with `git mv`, so the
      rename is staged and git keeps the file's history. Outside git, or for an
      untracked file, it renames the file directly. Every edge that points at
      the task is rewritten in the same pass, each keeping its relative prefix
      (`../sub/old.typ` becomes `../sub/new.typ`), and `mt` checks the whole
      set before it writes.],
    outputs: ("mv",),
  ),
  // ── Fields ──────────────────────────────────────────────────────────────────
  "set": (
    prose: [One field per call, written as `mt add --set` writes it:
      `field=value`, `field+=value` to add to a list, `field@path` to read the
      value from a file. `mt set` writes `title`, `priority` as a number,
      `difficulty`, `kind`, `assignee`, `tags`, `due` and `start`. A field with
      its own command is refused with that command's name: `status` (`flip`),
      `links` (`link`), `reviewed` (`review`), `recur` (`recur`) and the id
      (`mv`). A #link(u("/priority/"))[framework priority] such as `ice(...)`
      and an #link(u("/estimates/"))[`estimate`] are not single values, so
      `mt set` does not write them. `mt` checks the task before it writes, so
      a refused value leaves the file unchanged. Adding a value a list already
      holds writes nothing, so a re-run is safe.],
    outputs: ("set",),
  ),
  "eval": (outputs: ("eval",)),
  "go": (
    prose: [Resolves the task to a path and runs `[go] editor`, or `$VISUAL`, or
      `$EDITOR`. With none set it fails and asks for `[go] editor`. `mt`
      replaces a `{task}` token in the command with the path, or appends the
      path when there is no token.],
  ),

  // ── Listing ─────────────────────────────────────────────────────────────────
  "ls": (
    outputs: ("ls",),
    examples: "ls-examples",
  ),
  "find": (
    prose: [`mt find login bug` runs
      `mt ls 'contains("login") && contains("bug")'`, so every `ls` flag works and the `applied:` line
      shows the filter. `[ls] contains-fields` sets the fields a bare
      `contains()` searches; see #link(u("/filters/"))[Filters]. To search one
      field, use a filter: `mt ls 'contains("bug", title)'`.],
    examples: "find-examples",
  ),
  "ls.sort": (
    examples: "ls-sort-examples",
  ),
  "ls.columns": (
    examples: "ls-columns-examples",
  ),

  // ── Graph ───────────────────────────────────────────────────────────────────
  "graph": (
    examples: "graph-examples",
  ),

  // ── Links ───────────────────────────────────────────────────────────────────
  "link": (
    prose: [`mt link <task> <kind> <target>...` adds one edge per target.
      `mt link --apply <file>`, or `-` for stdin, reads a YAML or JSON map of
      #link(u("/concepts/#identity"))[stem] to kind to targets, described on #link(u("/dsl/"))[Task DSL]. Both
      merge: an edge with the same kind and target is not added twice, so
      re-running either writes nothing new. `--replace`, with `--apply`, makes
      the map the whole edge set of each stem it names. `mt` checks the kind
      and the target before it writes. A kind whose note is required, such as
      the built-in `related`, is refused without one. When one entry fails,
      `mt` reports it and writes the rest. `--dry-run` prints the plan
      instead.],
    outputs: ("link",),
  ),

  // ── Views ───────────────────────────────────────────────────────────────────
  "views": (outputs: ("views",)),

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
  "config": (
    outputs: ("config-origin",),
    examples: "config-examples",
  ),

  // ── Retire ──────────────────────────────────────────────────────────────────
  "rm": (
    prose: [`TARGET` takes any number of entries, each a task reference or a
      filter. `mt rm` retires every task any entry matches, in one pass with
      one confirmation prompt.],
    outputs: ("rm",),
  ),

  // ── Setup ───────────────────────────────────────────────────────────────────
  "init": (
    examples: "init-examples",
  ),
  "install": (
    prose: [`--no-lib`, `--no-defaults`, `--no-schema` and `--no-completions`
      each skip one part, and `--force` overwrites an existing defaults file,
      schema or hook. `mt` writes one completion hook per shell under
      `~/.config/mindtape/completions/`. Source the one for your shell from
      its startup file. A hook asks `mt` for completions on each tab press, so
      it offers this project's #link(u("/concepts/#identity"))[stems],
      statuses, #link(u("/named-sets/#views"))[views], templates and link
      kinds, and the presets for `mt init --preset`. A new `mt` needs no
      re-install.],
  ),
  "worktrees": (
    examples: "worktrees-examples",
  ),
  "preset": (
    examples: "preset-examples",
  ),
)
