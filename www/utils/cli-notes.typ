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
  "mv": (
    prose: [`NEW_STEM` is a bare stem — a trailing `.typ` is dropped, a path
      separator refused: `mv` renames a task where it stands, and moving one
      between roots or folders changes its
      #link(u("/derived-tags/"))[derived tags], which is a decision of its
      own. A stem another task already answers to, or an existing file, is
      refused before anything moves. The move goes through `git mv` inside a
      repository — so the rename lands staged and git carries the file's
      history across it — and through a plain rename outside one, or for a
      file git does not track yet. Every inbound edge is retargeted in the
      same pass, each keeping its own relative prefix (`../sub/old.typ`
      becomes `../sub/new.typ`), and the whole set is validated before a byte
      is written.],
    examples: ```sh
    mt mv abandonned-rondo abandoned-rondo   # rename, retargeting every edge
    mt mv abandoned-rondo minuet --dry-run   # the plan: move plus changed files
    ```,
  ),
  // ── Fields ──────────────────────────────────────────────────────────────────
  "set": (
    prose: [One field per call, spelled the way `mt add --set` already spells
      one: `field=value`, `field+=value` to append to a list field,
      `field@path` to read the value out of a file. Writable are the fields
      whose Typst literal is one unambiguous value — `title`, `priority` (the
      raw number), `difficulty`, `kind`, `assignee`, `tags`, `due`, `start`.
      A field with a command of its own is refused naming that command
      (`status` → `flip`, `links` → `link`, `reviewed` → `review`, `recur` →
      `recur`, the stem → `mv`), an unknown field naming the writable ones. A
      #link(u("/priority/"))[framework priority] (`ice(...)`) and an
      #link(u("/estimates/"))[`estimate`] unit dict stay out: neither is one
      value. The rewritten task is validated #emph[before] it is written, so a
      rejected value leaves the file byte-identical — and appending a value
      the list already carries writes nothing at all, so a re-run is safe.],
    examples: ```sh
    mt set fur-elise-polish priority=8        # a raw number
    mt set fur-elise-polish due=2026-07-01    # a date, written as on(2026, 7, 1)
    mt set fur-elise-polish tags+=classical   # append to the list
    mt set fur-elise-polish title@title.txt   # the value read from a file
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

  // ── Links ───────────────────────────────────────────────────────────────────
  "link": (
    prose: [The write path for `links:`. `mt link \<task\> \<kind\>
      \<target\>...` upserts one edge per target; `mt link --apply \<file\>`
      (`-` for stdin) reads an adjacency map — a YAML or JSON mapping of stem
      to kind to targets, documented with its full shape on
      #link(u("/dsl/"))[Task DSL]. Both merge: edges dedupe on (kind, target),
      so re-running either form writes nothing the second time and a map stays
      safe to re-apply as it grows. `--replace` (`--apply` only) makes the map
      the complete edge set for each stem it names. The kind must be
      configured and the target must resolve #emph[before] anything is
      written, so a refused edge leaves no file to repair — a kind whose
      `note` rule is `required` (`related`, out of the box) is refused without
      one, naming the edge. A per-entry failure is reported and the rest still
      land, `mt rm`-style; `--dry-run` prints the plan instead.],
    examples: ```sh
    mt link wire-oauth depends-on pick-a-provider   # one edge
    mt link wire-oauth depends-on a b c             # one call, three targets
    mt link wire-oauth related write-docs -m "same release"
    mt link --apply links.yaml                      # a whole adjacency map
    mt ls --format json | jq -f edges.jq | mt link --apply -
    mt link --apply links.yaml --replace --dry-run  # preview the full rewrite
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
    prose: [`TARGET` is variadic: zero or more entries, each a reference or a
      filter, resolved the same way a lone target resolves. The union of every
      entry's matches, deduplicated by id, retires in one pass — one
      confirmation prompt, one tombstone/rewrite pass covering the whole
      set.],
    examples: ```sh
    mt rm abandoned-rondo-alla-turca              # by ref
    mt rm abandoned-rondo-alla-turca minuet-in-g  # two refs — one pass
    mt rm 'status == "cancelled"'                 # by filter — every match
    mt rm terminal --dry-run                      # preview without writing
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
      `--force` overwrites an existing XDG `defaults.toml` / schema.],
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
