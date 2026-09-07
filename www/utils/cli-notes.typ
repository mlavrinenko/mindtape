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
    prose: [Positional words join into the title; `key=value` / `key+=value` /
      `key@path` tokens (after `--set`, or bare after `--`), a `-f` values file
      (YAML or JSON), or editor mode fill the rest. Full detail on
      #link(u("/templates/"))[Templates & slots]. `--dedup` decides what
      happens when the destination stem already exists (`fail` — bail before
      writing; `skip` — exit 0, write nothing; `add-suffix` — mint `-2`,
      `-3`, …). One policy decides a collision however the stem was produced.
      #link(u("/config/"))[`[tasks.naming]`] carries the project-wide `dedup`
      default the flag overrides; unset, it is `add-suffix` when a `stem`
      template is configured and `fail` otherwise.
      `--format` reports what happened — `text` (default prose), `path`,
      `path0`, or `json` — and the JSON object states the outcome
      (`wrote`/`skipped`/`suffixed`), not only the path. The rendered task is
      validated #emph[before] it is written, so a rejected add leaves no file
      behind: fix the value the error names and re-run. `--no-verify` skips
      that check and writes whatever it was handed.],
    examples: ```sh
    mt add cleanup kitchen              # title from positional words
    mt add fix login +backend +auth     # +word peels into the tags slot
    mt add note -- summary@notes.txt    # slot value read from a file
    mt add --describe                   # dump the template's slot schema
    mt add ship-it --dedup skip         # leave an existing task alone, exit 0
    mt add ship-it --dedup add-suffix   # write ship-it-2.typ instead of failing
    mt add ship-it --format json        # {"outcome": "wrote", "path": "..."}
    ```,
  ),
  "import": (
    prose: [Full cookbook — the canonical schema, `jq` adapters for Taskwarrior
      and GitHub Issues, the round-trip contract — on
      #link(u("/import-export/"))[Import & export].],
    examples: ```sh
    mt import                       # canonical JSON from stdin
    mt import export.json --dry-run # from a file, preview only
    task export | jq -f contrib/jq/taskwarrior.jq | mt import
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
    mt ls ready -n 10               # predicate + limit; a table says what it cut
    mt ls @stuck                    # named view — filter + sort + columns
    mt ls --json                    # shorthand for --format json
    mt ls --format csv              # table (default), json, csv, path, or path0
    mt ls --columns '#,title,file' --hyperlinks always   # clickable # and file columns
    mt ls ready --format path       # bare paths, one per line — for another program
    mt ls 'stale(30)' --format path0 | xargs -0 $EDITOR  # NUL-delimited
    ```,
  ),
  "find": (
    prose: [Sugar over the filter language, nothing more: `mt find login bug`
      desugars to `contains("login") && contains("bug")` and runs the same code
      path as `ls`, so every `ls` flag forwards and the desugared expression
      shows up in the applied filter. Which fields a bare `contains()` sweeps is
      `[ls] contains-fields` — see #link(u("/filters/"))[Filters]. To target a
      field, use the filter language directly: `mt ls 'contains("bug", title)'`.],
    examples: ```sh
    mt find tombstone               # one term
    mt find login bug               # several terms conjoin (AND)
    mt find bug --sort priority:desc -n 5   # every `ls` flag forwards
    mt find bug --format path | xargs $EDITOR
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

  // ── Graph ───────────────────────────────────────────────────────────────────
  "graph": (
    examples: ```sh
    mt graph                              # every task, Graphviz DOT
    mt graph 'blocked' --format mermaid   # subgraph as a Mermaid flowchart
    mt graph --no-terminal | dot -Tsvg -o graph.svg
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
  "init": (
    examples: ```sh
    mt init                           # questionnaire, writes .mindtape/config.toml
    mt init --preset flow --annotate  # named preset, keep the reasoning comments
    mt init --list-presets            # show built-in preset names
    ```,
  ),
  "install": (
    prose: [`--no-lib` / `--no-defaults` / `--no-schema` skip any one install;
      `--force` overwrites an existing XDG `defaults.toml` / schema. The hidden
      `mt install-lib` is a deprecated alias for `mt install --no-defaults
      --no-schema` — lib only, kept for existing docs and muscle memory.],
  ),
  "worktrees": (
    examples: ```sh
    mt worktrees      # list configured @name aliases
    mt @sn ls         # @name sugar rewrites to -w <path> before parsing
    ```,
  ),
  "preset": (
    examples: ```sh
    mt preset save my-shape   # capture this project's config + templates
    mt preset ls              # built-in and user presets together
    ```,
  ),
)
