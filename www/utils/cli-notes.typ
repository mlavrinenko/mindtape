// Reference-only enrichment for the CLI page — prose, worked examples, and the
// generated sample-output captures that belong with each command. NONE of this
// is in `mt _cli --json`; it lives only in ./www and is merged into the
// reflected command tree by cli-ref.typ, keyed by dotted path
// (`command`, `command.arg-id`, `nested.command.arg-id`).
//
// Fields per key: prose (content), examples (a ```sh block```),
// outputs (generated `<slug>.txt` captures), outputs-ansi (`<slug>.ansi`).
// cli-gate() fails the build if a key here does not resolve to a live node.
//
// Refs: truth/tasks/structured-cli-reference-from-command-dump.typ

#import "site.typ": u

#let cli-notes = (
  // ── Files ───────────────────────────────────────────────────────────────────
  "add": (
    prose: [Positional words join into the title; `--set`, `-f values.json`, or
      editor mode fill the rest. Full detail on #link(u("/templates/"))[Templates & slots].],
    examples: ```sh
    mt add cleanup kitchen              # title from positional words
    mt add fix login +backend +auth     # +word peels into the tags slot
    mt add --describe                   # dump the template's slot schema
    ```,
  ),
  "eval": (outputs: ("eval",)),
  "go": (
    prose: [Resolves `id` to a path, then spawns `[go] editor` (falling back to
      `$VISUAL`, then `$EDITOR`; no `vi` floor — a GUI user is told to configure
      `[go] editor` rather than dropped into one). A `{task}` token in the
      configured command is replaced with the resolved path; without one the
      path is appended.],
  ),

  // ── Listing ─────────────────────────────────────────────────────────────────
  "ls": (
    outputs-ansi: ("ls",),
    examples: ```sh
    mt ls ''                        # empty filter — lists everything
    mt ls 'has_tag("classical")'    # filter expression
    mt ls ready -n 10               # predicate + limit
    mt ls @stuck                    # named view — filter + sort + columns
    mt ls --json                    # shorthand for --format json
    mt ls --format csv              # table (default), json, or csv
    mt ls --columns '#,title,file' --hyperlinks always   # clickable # and file columns
    ```,
  ),
  "ls.sort": (
    examples: ```sh
    mt ls --sort due:asc --sort priority:desc   # composed, repeatable
    mt ls --sort @this-week                      # splice a view's sort chain
    ```,
  ),
  "ls.columns": (
    examples: ```sh
    mt ls --columns status,title    # one-off override of [ls] columns-order
    ```,
  ),

  // ── Views ───────────────────────────────────────────────────────────────────
  "views": (outputs: ("views",)),

  // ── Status ──────────────────────────────────────────────────────────────────
  "flip": (
    outputs: ("done", "flip-list"),
    examples: ```sh
    mt flip blocked 1b 2c -m "waiting on infra"   # batch + note
    mt flip --list                                # show the shortcut verbs
    mt flip done 1b --force                       # bypass the graph, waive a gate
    mt flip wip 1b --quiet                        # no [[on.flip]] show/list output
    ```,
  ),
  "review": (
    examples: ```sh
    mt review fur-elise-polish   # stamp reviewed: with today's date
    ```,
  ),

  // ── Recurring ───────────────────────────────────────────────────────────────
  "recur": (
    examples: ```sh
    mt recur hanon-warmup-daily              # spawn a new file, or refresh in place
    mt recur hanon-warmup-daily --dry-run    # preview without writing
    mt recur hanon-warmup-daily --advance 4  # pre-materialize 4 future spawns (spawn only)
    mt recur hanon-warmup-daily --skip       # advance the anchor/count, materialize nothing
    mt recur hanon-warmup-daily --force      # spawn again even if a successor exists
    ```,
  ),

  // ── Validate & inspect config ───────────────────────────────────────────────
  "check": (outputs: ("check",)),
  "config": (
    outputs: ("config-origin",),
    examples: ```sh
    mt config show              # effective config as re-parseable TOML
    mt config show --origin     # which config layers contributed, and from where
    mt config show --defaults   # just the embedded floor
    mt config show --json       # effective config as JSON
    ```,
  ),

  // ── Retire ──────────────────────────────────────────────────────────────────
  "rm": (
    examples: ```sh
    mt rm abandoned-rondo-alla-turca  # by ref
    mt rm 'status == "cancelled"'     # by filter — every match
    mt rm terminal --dry-run          # preview without writing
    ```,
  ),

  // ── Setup ───────────────────────────────────────────────────────────────────
  "install": (
    prose: [`--no-lib` / `--no-defaults` / `--no-schema` skip any one install;
      `--force` overwrites an existing XDG `defaults.toml` / schema. The hidden
      `mt install-lib` is a deprecated alias for `mt install --no-defaults
      --no-schema` — lib only, kept for existing docs and muscle memory.],
  ),
)
