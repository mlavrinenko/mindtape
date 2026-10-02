// Reference-only enrichment for the CLI page: prose, and the captures that
// belong with each command. None of this is in `mt _cli --json`; cli-ref.typ
// merges it into the reflected command tree by dotted path (`command`,
// `command.arg-id`, `nested.command.arg-id`). cli-gate() fails the build if a
// key here does not resolve to a live node.
//
// Fields per key: `prose` (content); `examples`, the slug of a capture whose
// commands render as a block to copy; `outputs`, slugs of captures that render
// as transcripts. www/generate-cli.sh runs every capture named here.

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
    outputs: ("add",),
  ),
  "import": (
    prose: [Full cookbook — the canonical schema, `jq` adapters for Taskwarrior
      and GitHub Issues, the round-trip contract — on
      #link(u("/import-export/"))[Import & export].],
    examples: "import-examples",
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
    outputs: ("mv",),
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
    outputs: ("set",),
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
    outputs: ("ls",),
    examples: "ls-examples",
  ),
  "find": (
    prose: [Sugar over the filter language, nothing more: `mt find login bug`
      desugars to `contains("login") && contains("bug")` and runs the same code
      path as `ls`, so every `ls` flag forwards and the desugared expression
      shows up in the applied filter. Which fields a bare `contains()` sweeps is
      `[ls] contains-fields` — see #link(u("/filters/"))[Filters]. To target a
      field, use the filter language directly: `mt ls 'contains("bug", title)'`.],
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
    prose: [`TARGET` is variadic: zero or more entries, each a reference or a
      filter, resolved the same way a lone target resolves. The union of every
      entry's matches, deduplicated by id, retires in one pass — one
      confirmation prompt, one tombstone/rewrite pass covering the whole
      set.],
    outputs: ("rm",),
  ),

  // ── Setup ───────────────────────────────────────────────────────────────────
  "init": (
    examples: "init-examples",
  ),
  "install": (
    prose: [`--no-lib` / `--no-defaults` / `--no-schema` / `--no-completions`
      skip any one install; `--force` overwrites an existing XDG
      `defaults.toml`, schema or hook. The completion hooks land one per shell
      under `~/.config/mindtape/completions/`; source the one your shell uses
      from its startup file. Each holds a couple of lines that ask `mt` for the
      completion on every tab press, never a generated completion script — so
      the candidates are this project's own stems, statuses, views, templates
      and link kinds, plus the presets `mt preset ls` lists for
      `mt init --preset`, and an `mt` upgrade needs no re-install.],
  ),
  "worktrees": (
    examples: "worktrees-examples",
  ),
  "preset": (
    examples: "preset-examples",
  ),
)
