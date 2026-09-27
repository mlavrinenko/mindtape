# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Filters read estimate units and kebab-case names.** `mt ls 'estimate.files
  > 3'` was refused, since `estimate` took presence only, and `has(x-goals)`
  failed with `expected a field, got Sub`, since `-` always lexed as minus.
  Names now follow Typst's identifier rule — a `-` between name characters is
  part of the name, a leading one is still a negative number — and
  `estimate.<unit>` reads one unit as a number for any unit name, absent when
  the task leaves it out, so a bound on an estimate is one rule expression. A
  unit outside a declared `[tasks.estimate] units` is a config error at load;
  `has(x-<ns>)` tests an extension slot's presence.

- **A status can refuse blocked work.** The legend's `to` edges checked the
  status pair, never the graph, so `mt done` closed a task whose dependency was
  still proposed. `unblocked = true` on a `[[tasks.status]]` entry now makes
  `mt flip` refuse entering it while a direct blocking-kind target is
  unfinished, naming each one (`--force` overrides), and `mt check` warns about
  a task already there. The `flow` and `gated` presets and this repo set it on
  `wip` and `done`.

- **A `why` column says what a blocked task waits on.** `mt ls blocked` named
  the stuck tasks but not their blockers, so an agent ran `mt eval` on each
  task and then on each target. `--columns title,why` now lists every
  unfinished direct blocking-kind target with its status (`waits on: a
  (proposed), b (wip)`), and `--json` carries it as `why: [{stem, status,
  kind}]` on every task, empty when ready. The column is `named()` in the floor,
  so nothing changes until it is asked for.

- **Shell completions, answered by the binary rather than generated into a
  file.** Tab after `mt flip ` used to offer nothing, and no completion script
  was ever written because the values worth completing are all
  project-derived: a committed script can state `--format json|csv` correctly
  forever and still be wrong about every stem, status, view and template.
  `mt install` now writes a hook per shell under
  `~/.config/mindtape/completions/` (`--no-completions` skips it, alongside the
  existing `--no-lib`/`--no-defaults`/`--no-schema`), each a couple of lines
  that ask `mt` itself for the answer on every press
  (`clap_complete`'s `CompleteEnv`). So the static half is clap's own parser
  and the dynamic half is the live config: `mt flip ` offers this project's
  legend, `mt ls @` its views, `mt add -t` its templates, `mt link <task> ` its
  edge kinds, `mt init --preset ` the presets `mt preset ls` lists, and
  every task argument its stems (`mt done ` included, as `mt flip done ` does) — with `mt set`'s writable
  fields and this project's tags behind `tags=`. Nothing is generated into a
  repository, so nothing can drift from one, and an `mt` upgrade needs no
  re-install. A project that fails to load yields no candidates and no error,
  because a broken tab press must not look like a broken shell. Which argument
  draws on which source is declared once, in
  `crates/mindtape-cli/src/completion.rs`, and reflected back out through
  `mt _cli --json`: `just check-doc-facts` set-differences the arguments clap
  declares against the ones named there, so a new completable argument fails
  a gate until it says where its values come from — "no project value fits"
  being an answer too, recorded with its reason.

- **`mt set <task> <field>=<value>` writes one core field from the CLI.** `mt`
  wrote `status:` (`flip`), `links:` (`link`), `reviewed:` (`review`) and the
  stem (`mv`), and nothing else — so changing a priority, a tag or a due date
  meant the hand edit `skills/mindtape/SKILL.md` opens by warning against, with
  nothing validating the result until `mt check` read the saved file. `mt set`
  now writes the fields whose Typst literal is one unambiguous value —
  `title`, `priority` (the raw number), `difficulty`, `kind`, `assignee`,
  `tags`, `due`, `start` — through the same `pre_write` seam every other
  writer goes through, so a value the project refuses never reaches the file.
  The token grammar is `mt add --set`'s own, parsed by the same code rather
  than re-spelled: `field=value`, `field+=value` appending to a list field
  (and writing nothing when the list already carries it), `field@path`
  reading the value out of a file. A field with a command of its own
  (`status`, `links`, `reviewed`, `recur`, the stem) is refused naming that
  command; an unknown field is refused naming the writable ones, with a
  nearest-match suggestion. A framework priority (`ice(...)`) and `estimate`'s
  unit dict stay out: neither is one value.

- **`mt mv <task> <new-stem>` renames a task and retargets every inbound
  link.** A task's identity is its filename stem, so a stem written before the
  task settled used to need a `git mv` plus a hand edit of every `links:`
  target naming it, with nothing catching a missed one until `mt check`
  reported a dangling edge. `mt mv` now owns both halves as one validated
  write: the file moves — through `git mv` inside a repository, so the rename
  lands staged, and through a plain rename outside one or for a file git does
  not track yet — and every inbound edge is retargeted onto the new stem,
  each keeping its own relative prefix (`../sub/old.typ` becomes
  `../sub/new.typ`). A stem another task already answers to, or an existing
  file, is refused before anything moves; `--dry-run` prints the plan (the
  move, and every file whose edges change) and writes nothing. `NEW_STEM` is a
  bare stem: a trailing `.typ` is dropped, and a path separator is refused —
  moving a task between roots or folders changes its derived tags and stays a
  decision of its own, as does changing its `title:`.

- **A config can `extends` rule packs and `disable` a rule by name.** Any
  config layer (XDG global defaults, an ancestor `.mindtape/defaults.toml`, or
  the project's own `.mindtape/config.toml`) may declare `extends = [...]` and
  `disable = [...]` at the top level. A bare name in `extends` resolves to a
  pack compiled into `mt` (the reserved `mindtape-` prefix; `mindtape-tasks` is
  the first to ship), and anything carrying a path separator resolves to a
  TOML pack file relative to the layer that names it (the project root, for
  the project's own config). A pack may declare only
  `[[tasks.rules]]`, `[[tasks.body.headers.rules]]`,
  `[[tasks.body.content.rules]]` and `[dicts]`, and may not itself `extends`.
  Per rule array, a layer's `extends` packs merge in first (in listed order),
  then that layer's own entries; a later entry sharing an earlier one's `name`
  replaces it in place rather than appending a second copy. `disable` then
  drops every rule (in any of the three arrays) carrying a listed name,
  erroring on a name matching none. `mt config show --origin` now also names,
  for each rule in those three arrays, the pack (`pack mindtape-tasks` / `pack
  <path>`) or layer it came from. Replacement by name works only ACROSS
  files: two entries sharing a name WITHIN one layer's or one pack's own
  array is a config error naming both, matching `[[tasks.body.headers.rules]]`
  / `[[tasks.body.content.rules]]`'s own duplicate-name check below.

  **Behavior change:** those three rule arrays now append across layers
  instead of the nearer layer replacing the farther one's wholesale — every
  other section keeps replacing wholesale, unchanged. An ancestor
  `.mindtape/defaults.toml`'s `[[tasks.rules]]` entry used to vanish the
  moment a project declared its own; it now survives alongside it (same name,
  still replaces in place).

- **The flagship task practice ships as the `mindtape-tasks` pack.**
  `extends = ["mindtape-tasks"]` gives a project the six-section task shape
  (Problem, TO-BE, Changes, Out of scope, plus a fix/feat/docs/dev kind's own
  Environment/Alternatives/Source of truth/Rollback section), its heading
  order, the 🔴/🟢 markers, the word caps, the em-dash density check, and the
  imperative-title rule with the `verbs` dictionary it reads — the whole
  practice this repo's own backlog already enforced by hand, now one line for
  any repo to adopt instead of a rule-for-rule copy that drifts. This repo's
  own `.mindtape/config.toml` now extends it, keeping local only what is
  actually local: the `goals` rule and an extra `Dogfood` section, added back
  by overriding the pack's `section-allowlist` rule by name — the worked
  example for that mechanism.

- **A body rule can carry a `name`.** `[[tasks.body.headers.rules]]` and
  `[[tasks.body.content.rules]]` entries accept an optional `name`, mirroring
  `[[tasks.rules]]`'s: `mt check` reports a breach as `body.headers.rules.<name>`
  / `body.content.rules.<name>` instead of the flat `body.headers` /
  `body.content` it printed before, so a reader can grep straight back to the
  config entry that fired. An unnamed rule now reports its position instead
  (`body.headers.rules[0]`), where it previously reported the same flat label
  every rule in the array shared. Two entries in one array sharing a name is a
  config error naming both. Groundwork for a future `disable = ["<name>"]`
  rule-pack override; nothing reads the name yet.

- **Every write refuses a task whose body breaks a header or content rule, not
  just `mt check`** (breaking, pre-1.0; library callers only). `mt add`, `mt
  flip`, `mt import`, `mt link`, `mt recur` and `mt review` now run `mt check`'s own
  body-header (`[[tasks.body.headers.rules]]`) and section-content
  (`[[tasks.body.content.rules]]`) gates over a candidate's rendered source
  before writing it, naming the same field and message `mt check` would print
  — a task `mt check` would reject can no longer land and wait for a later
  `mt check` or the pre-commit hook to catch it. `mindtape::check_candidate`/
  `check_candidates` now take a `mindtape::BodyRules` of the compiled header
  and content rules, and both gates run through one shared entry point,
  `mindtape::check_body_rules`, so `mt check` and the candidate seam cannot
  drift apart on what a body rule means.

- **A `[[tasks.rules]]` entry can check a field with a Typst function.**
  `check = { typst = "<path>", function = "<name>", field = "<field>" }` is
  now an alternative to `assert`, for a structure the filter language cannot
  name — an `x-<ns>` extension field, or a nested dictionary inside one. The
  function receives the field's value (`none` when absent), or the whole task
  in the shape `mt eval --json` prints when `field` is omitted, and returns an
  array of strings: empty passes, each string is one breach reported under the
  rule's own label and severity. A non-array return is one breach naming what
  came back, and a panic inside the function (Typst's own `panic()`) is one
  breach carrying the panic message rather than an eval error that aborts the
  project load. `name`, `when`, `message` and `severity` work the same as for
  `assert`; a rule declaring both or neither of `assert`/`check`, a missing
  `typst` file, or a module that does not export `function` is refused at
  config load, each as one config error. `mt add` refuses a write an
  error-severity `check` rejects, exactly as it already does for `assert`.

- **Filters and sorts reach a field's named parts.** A field with fixed named
  parts is now addressed as `field.property`: `status.name`, `status.date`
  (the ISO stamp), `status.note`, and `status.index` (the 0-based position in
  the configured legend, numeric) work in a comparison, `has`/`miss`, and
  `--sort` — one registry entry each, so a property is filterable and sortable
  by construction. A string-valued property takes `contains`/`matches` too;
  `status.index` is numeric, and `Missing` for a statusless task or a name
  outside the legend. Bare `status` keeps its split: a comparison reads the
  lifecycle name (`status == "wip"` unchanged), an ordering reads the legend
  position. `--sort`'s accepted vocabulary is now derived from that registry
  plus the computed `ready` rank, so a new field or property is sortable
  without touching a second list.

- **A flip gate can refuse without a `run` command.** `[[on.flip]]`'s
  `run` is now optional on a gating action: `{ gate = true, message = "..." }`
  with no `run` refuses the flip unconditionally whenever the hook's `when`
  already matched, printing only its `message` — no command to spawn, no
  output to surface. Previously a condition that belonged entirely to `when`
  (e.g. "refuse starting a stale task") had to be spelled as a command whose
  exit code stood for the refusal, `run = "false"` being the only shape that
  worked. `FlipAction::kind()` resolves `gate = true` with `show`/`list`/`run`
  all unset to the same `Run` kind with no command; an action with neither
  `run` nor `gate = true` — nothing to show, list, run, or refuse — is still a
  config error at load, naming the action. This repo's own stale-start gate in
  `.mindtape/config.toml` drops its `run = "false"` stand-in now that the
  direct shape exists.

- **The skill files verb-first titles.** `skills/mindtape/SKILL.md`'s `mt add`
  section now defaults every new title to an imperative verb first
  (`fix login bug`, not `login bug fix`) — the shape a project's own
  `[[tasks.rules]]` entry may hold it to (`in_dict("verbs", word(title,
  1))`), and the one this repo's own `.mindtape/config.toml` now enforces at
  `error`: no mechanism of its own, just the dictionary, string-argument and
  rules work landing before it turned on. `examples/learn-piano` carries the
  same rule at `warn`, the severity a project adopting the convention
  mid-backlog actually wants — its closed history stays untouched, and its
  open titles were brought into line except one, left as filed to prove a
  `warn` breach prints and `mt check` still passes. The worked example lives
  on the config page's `Config-declared rules` section; the filter page's
  `in_dict` entry now points at it instead of restating it.

- **`mt rm` retires more than one task in one call.** `TARGET` is now
  variadic, mirroring `mt flip`'s argument shape: each entry resolves as a
  reference or a filter exactly as a single target does today, and the union
  of every entry's matches — deduplicated by id, so the same task named twice
  or named once directly and again by a filter retires once — is retired in
  one pass: one confirmation prompt, one SWHID tombstone/inbound-link rewrite
  covering the whole set. Zero targets still applies `[rm] default_filter`
  exactly as before.

- **A `density` content assert.** `{ density = { pattern = "regex", max = N,
  per = "section" } }` joins `non-empty`/`min-words`/`max-words`/
  `must-match`/`must-not-match` as a sixth `[[tasks.body.content.rules]]`
  assertion: it fails when any single unit of the matched section carries
  more than `max` matches of `pattern`, a rate `must-not-match` cannot
  express since it fails on the first occurrence. `per` (default `section`)
  picks the unit — the whole section, a blank-line-separated paragraph (a
  `- `/`+ ` list item opens its own paragraph even without a blank line
  before it), or a paragraph split again into sentences on `.`/`!`/`?`. A
  fenced code block is dropped before the split runs, unlike every other
  content assert, so a pattern inside a code sample is never counted. This
  repo's own `.mindtape/config.toml` now carries a `density` rule capping
  every open task's paragraphs at one em-dash.

- **Rule packs get a dedicated reference page.** `/rule-packs/` covers what a
  pack file may contain (the three rule arrays plus `[dicts]`, nothing else,
  no nested `extends`), an embedded pack versus a path pack, adopting one
  with `extends`, overriding a single rule by name or dropping it with
  `disable`, the merge order across layers and packs, what `mt config show
  --origin` prints, and what `mindtape-tasks` enforces — quoted from the pack
  itself so the page cannot drift from it. `/config/`'s "The four layers"
  now carries a short `extends`/`disable` paragraph of its own and links out
  to the new page instead of repeating it.

- **`mt ls --skip` pages a listing, and a cut table names the exact command
  that fetches the next page.** `mt ls -n 20` printed what it cut but not how
  to see the rest, so an agent with a large backlog narrowed the filter
  blindly or raised `-n` until nothing was cut. `--skip N` now drops N rows
  after the sort and before `-n`, and a truncated table's footer names the
  next command verbatim — this run's own filter and flags, `--skip` advanced
  past what it just showed (`truncated: 20 of 143 — next: mt ls -n 20 --skip
  20`). `--json` carries the same value structurally as `truncated.next_skip`,
  additive alongside `.shown`/`.total`, so `SCHEMA_VERSION` does not move.
  Paging past the end of the match set — `--skip` clearing the last row — is
  a complete listing, not a truncation, exactly like a limit that hides
  nothing.

### Removed

- **Every backward-compatibility shim for a pre-release spelling** (breaking,
  pre-1.0). Nothing here ever shipped in a public release, so each one was a
  hand-written hint pointing an old spelling at its replacement rather than a
  compatibility path worth keeping; each now fails the same plain way any
  other unrecognised name does, naming nothing to migrate from.
  - `required = "non-terminal"` (`crates/mindtape/src/config_required.rs`) no
    longer gets a dedicated "no longer a value; write `!terminal`" message —
    it is dry-run as an ordinary filter expression like any other `required`
    string, and whatever that expression fails on is reported under the
    field's path.
  - The filter/`--sort` field `file` (`crates/mindtape/src/fields.rs`,
    `filter.rs`, `sort.rs`) no longer appends "renamed to `path`" to its
    unknown-field/unknown-sort-field error; `file` is simply not a field.
  - `contains(field, "substr")`'s reversed-argument hint and the `search(...)`
    dispatch arm (`crates/mindtape/src/filter_predicates.rs`, `filter.rs`) are
    gone: a leading identifier where the needle belongs is now an ordinary
    "expected a literal" error, and `search(...)` is an ordinary "unknown
    function" one.
  - `has(depends_on)` / `has(supersedes)` / `has(duplicates)`
    (`crates/mindtape/src/filter_links.rs`) are no longer sugar for
    `links("depends-on")` etc.; the underscore spelling is now an unknown
    field like any other. Use `links("<kind>")`.
  - `mt install-lib`, the hidden alias for `mt install --no-defaults
    --no-schema` (`crates/mindtape-cli/src/cli.rs`, `init.rs`, `install.rs`),
    is gone outright — not merely hidden. It is neither a subcommand nor a
    legend shortcut verb, so it falls through to the ordinary "unknown
    command" error.
  - The `key[]=value` `mt add` append spelling
    (`crates/mindtape-cli/src/add_tokens.rs`) no longer fails naming
    `key+=value`; the bracketed text is now an ordinary (if oddly spelled)
    key, rejected downstream as any unknown `--set`/`-f` key is.

- **`mindtape::sort::SORT_FIELDS`** (breaking, pre-1.0; library callers only).
  The sort vocabulary is now derived from the field registry —
  `sort::is_sort_field` accepts every registered name plus the computed `ready`
  rank — so the hand-kept const would be the second list the registry exists to
  remove. An out-of-tree caller that enumerated it can ask `is_sort_field` per
  candidate name; nothing in this workspace imported the const.

### Changed

- **An estimate bound can be gated on the task it bounds** (breaking, pre-1.0;
  library callers only). `[tasks.estimate.max]` and `[tasks.estimate.min]` now
  take an array of entries, each with an optional `when` filter and `message`:
  a unit's bound is the first entry whose `when` matches and that names the
  unit, so `when = "difficulty >= 3"` can hold judgment work to 5 files while a
  mechanical sweep keeps a wider ceiling. A breach names the gate and carries
  the entry's `message` as its advice. The single-table spelling still parses
  as one unconditional entry, so no config has to change; a malformed `when`,
  an entry naming no unit, or a unit outside `units` now fails at load.
  `EstimateConfig::bound_problems` is replaced by `EstimateBounds`, compiled
  once per validation pass.

- **`LimitSeverity` is now `Severity`** (breaking, pre-1.0; library callers
  only). `warn` / `error` is no longer a property of a work-in-progress cap:
  `[[tasks.rules]]` spells the same two words in TOML and means the same two
  things by them, so `mindtape::Severity` is one enum both sections read rather
  than two that would have to be kept in step by hand. No config key changed —
  `severity = "error"` under `[[tasks.limits]]` reads exactly as before.

- **A filter argument narrows the rows and leaves the presentation alone**
  (breaking, pre-1.0). `mt ls '<filter>'` used to skip rule matching and the
  default view wholesale, so typing a filter on a branch where an `[[ls.rules]]`
  entry was active silently took that rule's `sort` and `columns` down with it
  — and the `applied:` trailer never said so. Every layer is per facet now: the
  cascade resolves first, an explicit filter overrides the `filter` facet,
  `--sort` and `--columns` override theirs, and a facet nobody claims keeps
  what the rule or the default view gave it. A bare `@view` argument still
  overrides every facet that view sets — naming a view is choosing a reading,
  not just a set of rows.

  Two attribution changes come with it. A rule's own `filter`/`sort`/`columns`
  override is now labelled with the rule (its `name`, else its `condition`,
  else `rule`) instead of `manual`, which is the command line's label alone —
  otherwise a listing whose filter came from the caller and whose sort came
  from a rule would report both as one source. And a rule that matched but lost
  every facet is named rather than vanishing: `applied: manual (filter) ·
  release superseded` under the table, `"superseded": true` in `mt ls --json`'s
  `applied` block (additive, no `schema_version` bump).

- **A column's visibility is one `when` expression** (breaking, pre-1.0).
  `[ls.columns.<name>]` used to carry two keys that both answered "does this
  render" — a `visibility` scalar (`auto` / `visible` / `hidden`) and a `rules`
  list of conditional overrides. Two keys need a precedence rule, the
  precedence needed stating in three places, and the `rules = []` opt-out
  existed only because the two could disagree. All of it is replaced by a
  single boolean expression in the condition language:

  ```toml
  [ls.columns.file]
  when = 'named() || (!tty() && format() == "table")'
  ```

  No list, so no ordering and no first-match-wins; `||` does what rule order
  did. A scalar replaces wholesale across config layers, so a project override
  is `when = 'false'` on one line and the `rules = []` opt-out goes away with
  the thing it was working around.

  The per-column defaults `auto` used to encode in Rust move into the embedded
  floor, one `when` per column — `#` and `title` are `true`, `status` /
  `priority` / `due` are `nonempty()`, `assignee` is `contested()`, `ready` is
  `blocked()` — so the three different things `auto` meant are now legible
  where a reader looks for them. Nothing falls back to Rust: the floor is the
  complete statement of what `mt ls` shows, and a column with no `when` never
  renders. A blank `when = ''` is a config error naming the column, since the
  condition language reads an empty expression as always-true and that would
  contradict what an absent one means.

  Rendered output is unchanged for an unconfigured project. There is no
  migration shim: `visibility` and `rules` are rejected as unknown keys.

- **`--format csv`'s `file` cell is now the project-root-relative path**
  (breaking, pre-1.0), the same value `--format json` already reported under
  `file`. It used to go through the same renderer the table column does, so
  the two machine formats disagreed by whichever directory produced them: from
  `tasks/`, `--format json` said `tasks/a.typ` and `--format csv --columns
  file` said `../tasks/a.typ` for one task.

  CSV is a record format, not a rendering — the docs already called it a data
  contract, and it already substituted the task's stable `id` for the table's
  ephemeral `#` handle for exactly this reason. `file` is the second column to
  follow that rule: where the table shows something only the render that
  produced it can resolve, CSV emits the value a program can keep. So a CSV
  row now agrees with a JSON record field for field, and its `file` is the
  value the `path` filter compares and `mt import` reads back.

  The cwd-relative form is unchanged and still has two surfaces of its own —
  the table's `file` column, and `--format path`/`path0`, which exist to be
  pasted back into a command from the directory that produced them. Those two
  stay pinned to each other.

- **The filter/sort field `file` is now `path`** (breaking, pre-1.0). One name
  covered two values: the field compared the project-root-relative path, while
  the `file` column and `--format path` render the same task's path relative
  to the working directory. Inside the project root the two coincide, which is
  why it went unnoticed; under `mt -w <other-project>` they diverge and a
  filter built from what the listing showed matched nothing — silently, an
  empty listing being also what a correct filter over an empty result set
  looks like.

  The display is unchanged and stays working-directory-relative: it is the only
  form openable from where the reader is standing. The comparable value stays
  root-relative, so a filter saved in `.mindtape/config.toml` means the same
  thing wherever `mt` runs from — and is the same value `--format json` reports
  as each task's `file`. Only the name moved.

  `file` in a filter (`mt ls 'contains("x", file)'`, `has(file)`) and in a sort
  spec (`--sort file`, `[query] default_sort`) is now an error naming `path`,
  rather than a silent miss. No shipped view, `[[ls.rules]]` entry or example
  config filtered on it.

- **Every task writer now validates before it writes** — `mt add` already
  rendered into a buffer, checked the candidate at the exact path it would
  occupy, and wrote only if it passed. The rest wrote first and reported
  after: `mt recur`'s spawn created a new file exactly the way `add` used to,
  so a rejected occurrence left the same broken artifact behind; `mt import`
  landed the whole batch and then `remove_file`d the failures; `mt flip`,
  `mt review` and `mt link` rewrote a file that was valid before they touched
  it and reported afterwards without restoring it. All of them go through one
  seam now, and a rejected write leaves nothing to repair.

  A rejected rewrite is **not restored** — it is simply not written, so the
  file still holds exactly what its author put there. Restoring would be a
  second write, which can fail on its own and would leave the file in a third
  state nobody can name; not writing cannot fail.

  For `mt import` the answer to "one bad record, or the whole batch" is
  unchanged: **land the rest**, and the `N filed, M skipped` report says which.
  What changed is that the bad one is never created, so `--dry-run` no longer
  needs a temp mirror of the project to preview against, and a batch rejected
  end to end leaves not even an empty destination directory.

  A batch goes to the validator as one set (`mindtape::check_candidates`),
  because the rules that span tasks have no answer for one member alone: id
  uniqueness, edge resolution, cardinality, cycles, and the dependency ranks a
  `required` filter reads. That is what lets `mt link --apply` add an edge in
  both directions at once and `mt recur --advance` chain occurrences that name
  their predecessor. `mt flip` keeps its post-batch revalidation as the net for
  what a landed flip does to *other* tasks, which no per-candidate check sees.

### Fixed

- **A write that closes a dependency cycle is refused.** `mt check` names a
  cycle on its first member, and the pre-write check kept only the violations
  naming the tasks being written, so `mt link beta depends-on alpha` landed
  `alpha → beta → alpha` and left `alpha` refusing every later write. A cycle
  the batch closes now refuses it wherever `mt check` reports it — `mt link`,
  `--apply`, `mt import` and `mt add` alike — while a cycle already on disk
  still never blocks an unrelated write.

- **`mt eval` takes a task reference.** It accepted a file path only, so the
  skill's `mt eval <task>` failed on a stem with `no such file`. A stem or `#`
  handle now resolves like `flip`, `set` and `link`, a path still evaluates a
  `.typ` under no project, and tab completion offers stems. An unknown stem, on
  every command, names the nearest one (`did you mean: test?`).

- **`compute`, `filter`, `page`, `roll` and `switch` pass the imperative-title
  rule.** All five are in `mindtape-dev-verbs` now (91 entries); the
  frequency-derived `mindtape-verbs` stays untouched.

- **An unknown `--sort` direction is an error.** `validate_sort_spec` checked
  only the field before the `:`, and `parse_spec` treated every suffix but
  `desc` as ascending, so `--sort priority:wtf` — or a saved
  `[ls] default-sort` — sorted ascending and said nothing. Only `asc`, `desc`,
  and no suffix are accepted now, with the error naming the direction, and
  `sort_by_specs` delegates to the same validator.

- **A saved filter expression can reach a project `[dicts.<name>]` again.**
  Config load compiled the dictionaries *after* running the semantic pass, and
  that pass dry-runs every saved expression against a probe task — so a
  `[views]` filter, a `[[tasks.limits]]` filter or a `required` rule spelling
  `in_dict("verbs", ...)` was checked against a config whose only dictionaries
  were the two built-ins, and failed at load naming a dict the project plainly
  declares. The two steps are swapped; nothing is lost by the order, because
  compiling the section runs the same `[dicts]` checks the semantic pass does,
  so a broken dictionary is still refused by whichever pass sees it first.

- **`mt eval` no longer hijacks a foreign `lib/` as the `@local/mindtape`
  package.** It registered `<root>/lib` under that name whenever a
  `lib/typst.toml` merely existed, never reading the name the manifest
  declares, so in any worktree whose `lib/` is some other Typst package the
  import resolved there and the command died with `package manifest contains
  mismatched name '<other>'` — while `mt ls` and `mt check` on the same tree
  were fine. The batch loader has read that name since the feature landed;
  `mt eval` built its own world and carried a second, hand-written copy of the
  decision that never learned it. There is one copy now: `mindtape` exports
  `package_overrides`/`package_root` and `mt eval` asks them, so the two paths
  cannot drift again.

  Two things follow from asking the same function. `mt eval` loads the
  enclosing project's config, so its `[[typst.local-packages]]` entries apply
  there as they already did everywhere else — and a config that does not parse
  is now an error from `mt eval` too, where it used to be ignored. And an
  unoverridden `@local/…` resolves under `$MINDTAPE_DATA_HOME` rather than the
  platform data directory, closing the same install-and-load mismatch the
  loader closed. What `mt eval` renders against is unchanged: the embedded
  floor, not the project's columns.

- **`just check-secrets` no longer logs an `ERR` over `www/generated/` on a
  green run.** The walk bound in `.gitleaks.toml` was believed to close that
  race; it does not, and strace says why: gitleaks 8.30 `lstat`s a directory
  entry one syscall *before* it consults the allowlist that skips it, so the
  bound stops the walk descending and nothing more. `www/generate.sh` `rm -rf`s
  that directory on entry and `just check` runs its arms in parallel, so a
  deletion landing between `readdir(www)` and that `lstat` still logged
  `could not get info` at ERR while the gate reported no leaks and exited 0 —
  8 runs in 20 under a synthetic churner. The scan now holds the same
  `target/www-generated.lock` flock every other reader of that directory takes,
  released the moment the walk is over; 0 runs in 20 after. An assertion beside
  the acquisition fails the gate if the scan is ever running without the lock,
  since removing it would restore the race in silence.

- **`mt recur` on an already-spawned occurrence wrote a task `mt check`
  refuses** — a spawn's source is its predecessor's source, so an occurrence
  spawned from an occurrence already carried a `recurred-from` edge, and the
  spawn appended a second one. `recurred-from` allows one edge per task, so
  every hop of the ordinary loop after the first produced an invalid file: the
  post-write check reported it, and the file stayed. The edge is now replaced
  rather than appended — the chain is a linked list, each occurrence naming
  only the one before it — and every other edge kind is carried forward
  verbatim, notes included.

### Added

- **`[tasks.tags] allowed` and `[tasks.fields.<name>] values` can point at a
  `[dicts.<name>]` instead of restating it.** Two projects sharing a taxonomy
  used to copy the vocabulary, and a list long enough to be worth naming had
  nowhere to live but pasted inline — even though `[dicts]` already gave that
  vocabulary a name.

  ```toml
  [tasks.tags]
  allowed = ["@areas", "swept", "size"]

  [tasks.fields.kind]
  values = ["@kinds"]
  ```

  `@<dict>` expands to that dictionary's entries at config load, before any
  rule reads the list, so every downstream reader — validation, `mt add` slot
  completion, the generated schema — sees a plain, already-resolved set and
  needs no change of its own. A list may mix literal entries and `@<dict>`
  references freely; the result is their union in declaration order,
  deduplicated. An unknown dictionary name is refused at load with a
  nearest-match suggestion, the same treatment an unknown `include` target
  gets.

  `@` is the same sigil `mt ls` reads as a view reference, and it means the
  same thing here: always a dictionary, never a literal. There is no escape
  for a tag or field value that itself starts with `@` — an entry like
  `"@urgent"` is looked up as a dictionary named `urgent` and fails the load
  unless one by that name exists, in which case it expands to that
  dictionary's entries rather than surviving as the literal string.

- **`[[tasks.rules]]` holds a convention MindTape does not know by name.**
  Every constraint the tool enforced was one it had a field for — a status
  legend, a numeric scale, a tag allow-list, a heading path — so a project
  wanting to hold any other convention had nowhere to put it. A rule is the
  general form: an assertion in the filter language that must hold for every
  task it selects. It constrains no field in particular, because the
  expression already names the field it is about.

  ```toml
  [[tasks.rules]]
  name = "imperative-title"
  assert = 'in_dict("verbs", word(title, 1))'
  when = '!terminal'
  message = "start the title with an imperative verb"
  severity = "error"
  ```

  `assert` is the only required key; `when` gates which tasks are asked (absent
  means all of them), `message` replaces the default advice — which names the
  `assert` expression itself — and `severity` is `"warn"` (the default) or
  `"error"`.

  Rules are evaluated inside the validation pass, not in `mt check` alone.
  Every write command validates before it writes, so `mt add`, `mt flip`,
  `mt link`, `mt import`, `mt recur` and `mt review` enforce a rule for free:
  an `"error"` breach refuses the write before anything lands, leaving nothing
  to repair. A convention that only fails hours later at check time is a
  convention nobody keeps, and that is what makes this the feature rather than
  a new `mt check` arm.

  Carrying `warn` through a pass that used to be all-fatal is the one shape
  change. `Violation` gained an optional `severity`: `None` for every check
  MindTape makes on its own — a malformed artifact is not a matter of degree —
  and `Some` only for a rule, where the project chose the level. `mt check`
  counts the fatal ones for its exit status and the pre-write seam splits its
  verdict in two, so a warned write prints its line and still lands while an
  errored one is refused whole. A breach reports at the task's `task.with(…)`
  line:

  ```
  error: tasks/loader-refactoring.typ:3: rules.imperative-title: start the title with an imperative verb
  ```

  An unnamed rule reports as `rules[0]`, its index in `[[tasks.rules]]`. Both
  expressions are dry-run once at config load against a probe task, exactly as
  `required` and `[[tasks.limits]]` are, so a rule naming an unknown field,
  view or dictionary is one config error rather than one violation per
  artifact.

- **A filter can read a word out of a field and look it up in a dictionary.**
  The filter language compared a field against a literal and nothing else:
  every function returned a boolean and every argument was a field name or a
  literal, so "the first word of the title" was unsayable and a `[dicts]` set
  was unreachable from an expression.

  ```sh
  mt ls 'in_dict("verbs", word(title, 1))'   # titles opening with a known verb
  mt ls '!matches(word(title, 1), "ing$")'   # ...and not a gerund
  ```

  The load-bearing change is that a function's argument may now be another
  function's result, not only an identifier or a literal. Three functions buy
  it: `word(field, n)` — the n-th whitespace-separated word of a string field,
  counting from 1, trimmed of everything non-alphanumeric at either end;
  `matches(field, "regex")`, the escape hatch a closed vocabulary needs; and
  `in_dict("name", <string>)`, membership in a `[dicts.<name>]` set or a
  built-in one, under that dictionary's own `case`. All three work wherever a
  filter already runs — `mt ls`, a `[views.*]` filter, a `[[tasks.limits]]`
  filter, a `required` rule, an `[[on.flip]]` `when` gate.

  `word()` takes no position on Typst markup: a title opening with a code span
  yields the token inside the ticks, which no verb dictionary carries.
  Exempting such a title is the rule author's business, which is what
  `matches()` is for.

  A value that is not there — a field the task does not carry, a word past the
  end of the title — reads as a non-match, the same missing-field rule
  comparisons follow. Everything else is an error naming itself: an unknown
  dictionary (with a nearest-match suggestion and the known names), a pattern
  `regex` refuses, a numeric field where a string is required, and a wrong
  argument count.

- **`[dicts.<name>]` names a set of strings once and reuses it.** Every
  vocabulary used to be written out where it is used — `[tasks.tags] allowed`,
  `[tasks.fields.<name>] values` — so a set two rules share was two lists that
  drift, and a set too large to paste had nowhere to live at all.

  ```toml
  [dicts.verbs]
  values = ["add", "wire", "fix"]
  path = "dicts/extra-verbs.txt"
  include = ["mindtape-verbs", "mindtape-dev-verbs"]
  case = "fold"              # fold (default) | exact
  ```

  It is a top-level section, sibling of `[views.<name>]`, because a set is not
  a property of tasks. All three sources feed one set and any combination is
  legal so long as one is present: `path` is relative to the project root (and
  may leave it, the same rule `[[typst.local-packages]] path` follows), one
  entry per line, `#` comments and blank lines skipped; `include` unions
  another dictionary's entries in raw, so the including dictionary's `case`
  decides how they compare. `case = "fold"`, the default, is ASCII
  case-insensitive; `exact` is byte equality.

  Two dictionaries ship embedded in the binary and need no declaration to be
  included: `mindtape-verbs` (the 500 most frequent English verbs in base
  form) and `mindtape-dev-verbs` (86 software-work verbs the frequency list
  ranks too low to carry). The `mindtape-` prefix is reserved for them —
  naming a project dictionary `mindtape-anything` is a config error, so the
  namespace stays the tool's.

  Everything is checked at config load, once for the project rather than once
  per task: the reserved prefix, a dictionary with no source, an unknown
  `include` (with a nearest-match suggestion and the known names), an
  `include` cycle, a `path` that does not resolve or read, and a dictionary
  that resolves to no entries. This entry is the section, its compiler, and
  `mt config show` / the JSON schema carrying it; reading one from a filter is
  the entry above, and the allow-list references are a separate change.

- **A column condition can read the rows it would render** — four predicates
  join the condition language for `[[ls.columns.<name>.rules]]`: `nonempty()`
  (any cell in this column carries text), `contested()` (the rows disagree
  about it), `named()` (the caller asked for this column by name), and
  `blocked()` (any listed task is blocked). They are exactly the facts the
  built-in `auto` visibility already decided from — `file` means `named()`,
  `assignee` means `contested()`, `ready` means `blocked()`, everything else
  means `nonempty()` — so a project that wants a different answer for one
  column can now write it in config instead of asking for a code change.

  `ready` kept a predicate of its own rather than folding into `nonempty()`:
  on an all-ready listing every `ready` cell carries a label, so the two
  genuinely disagree, and reusing `nonempty()` would have grown a column on
  every listing.

  The four are listing facts, not ambient ones, so they only have an answer
  while a column is being resolved; in an `[[ls.rules]]` condition they read
  `false`. A typo is still a load-time error naming the column
  (`Condition::parse` dry-runs every expression), so `nonemtpy()` fails on
  `mt ls` rather than becoming a column that silently never renders.

  Mechanically, a column's `rules` are now resolved by the renderer's own
  `resolve_order` — the one place that has the rows — instead of being folded
  onto the config beforehand. Both the table and CSV go through it, so they
  still cannot disagree about the column set, and `mt eval` and a `[[on.flip]]`
  `list` table now resolve column rules like `mt ls` does rather than skipping
  them.

- **A load failure on a literal `#` in a task body now names `\#`** — task
  bodies are real Typst markup, so prose like `Channel #N carries it` opens an
  inline expression and the file fails with `eval error: unknown variable: N`,
  a message naming a variable the author never wrote and no remedy. Every
  command that warns about load errors now follows them with one note saying
  to write `\#` or move the prose into a code block. Derived once from the
  whole load outcome, next to the existing `mt install` hint, so a backlog
  where every body trips reports it once and both notes can appear together.
  The Task DSL page grew a *Task bodies* section saying the same thing to the
  author who has not hit it yet. Auto-escaping stays out: bodies are
  intentionally real Typst, and escaping would break the markup that works.

- **`links:` has a CLI writer — `mt link`** — it was the one task field with
  none, so cross-linking a batch of tasks meant hand-editing files, and the
  thing that actually gets reached for at that point is a regex one-shot over
  the tree. `mt link <task> <kind> <target>...` upserts one edge per target;
  `mt link --apply <file>` (`-` for stdin) reads an adjacency map — a YAML or
  JSON mapping of stem to kind to targets, where a target is a bare stem or a
  `{target, note}` mapping — so a `jq` program emitting that shape pipes
  straight in. Both forms merge and dedupe on (kind, target), with a target's
  `.typ` optional on the way in, which is what makes a map safe to re-apply as
  it grows rather than a thing you run once: re-running writes nothing.
  `--replace` (`--apply` only, since the positional form upserts one edge and
  clearing the others there would be a different command wearing the same
  name) makes the map the complete edge set per stem; `--dry-run` prints the
  plan instead. The kind, the target and the kind's note rule are checked
  *before* anything is written, so a refused edge leaves no file to repair —
  `related` without a note is refused naming the edge, and an unknown kind
  lists the configured ones with a nearest match. A per-entry failure is
  reported and the rest still land, the way `mt rm` and `mt flip` report a
  batch, and the landed batch revalidates over one reload, so a cycle the
  batch created is reported by the run that created it. Edges serialize
  through the prelude's own typed helpers with the note markup-escaped, so a
  written file reads as hand-authored and `mt check` stays green. One
  deliberate refusal: `--replace` on a task whose `links:` carries a SWHID
  history pointer, since rewriting the value from the evaluated edges would
  erase provenance `mt rm` went to a git round trip to record — merge still
  works there.

- **A truncated listing now says what it cut** — `mt ls -n 10` printed ten rows
  and nothing about the other thirty-seven, so a bounded listing and a short
  backlog were the same output and a caller reported the one as the other. The
  cut is now reported in whatever the format has room for: a dim
  `truncated: showing 10 of 47` under the table, beside the `applied:` trailer;
  a `"truncated": {"shown": 10, "total": 47}` key in `mt ls --json`'s envelope;
  and deliberately nothing in `csv`, `path` and `path0`, which are record
  streams a trailing prose line would corrupt for the `csv` reader or the
  `xargs -0` on the other end — the exact failure `-n` exists to avoid. The
  JSON key is always present, `null` on a complete listing, so a consumer reads
  one lookup rather than testing whether a key exists; being additive it breaks
  no `mt import` or `jq` consumer, so `schema_version` does not move. `total`
  is the match count taken before the cut, which makes it the number a bigger
  `-n` would show rather than the number printed, and the cut still happens
  after the filter and the sort, so `-n` composes with `--sort` and a view's
  own sort chain. `mt find` inherits all of it by flattening the same flag
  struct. `mt ls -i` carries the line into the picker's footer, where it
  matters most: the loop can only open what it lists, so a capped view would
  otherwise read as a missing task.

- **`mt flip` reports the handles it just invalidated** — a handle's ordinal is
  the task's position in the default view, recomputed on every command, and a
  flip re-sorts that view: `1b` is stale the moment the write it named lands,
  and nothing the flip printed said so. The caller found out on the NEXT
  command, from a check mismatch naming a task they never meant — the guard
  firing exactly as designed, one command too late and about the wrong
  question. A landed batch now closes with `handle <stem>: 1b → 3b`, one line
  per flipped task whose ordinal moved. A task the flip filtered OUT of the
  view has no ordinal left to hand back — `mt done` under the config floor's
  `!terminal` filter, so the common case rather than an edge — and its line
  says `→ none` naming the filename stem, which still addresses it; a flip
  back INTO the view is the mirror and reads the same. A flip that moved
  nothing says nothing, because a same-status re-stamp reorders nothing and a
  line asserting that would train the reader to skip the line that matters.
  The two views it diffs are taken once each, before the batch and after,
  never once per target: the second rides the project reload the post-flip
  revalidation already pays for, and a per-target recomputation would report
  ordinals that existed only mid-batch — in `mt flip wip 1x 2f`, the second
  target moving from the ordinal the first target's write had just vacated,
  which was on nobody's screen. Only the flipped tasks are reported, not every
  handle they displaced: the caller is holding the one they typed, and a
  batch that renumbered a hundred rows would otherwise answer a question
  nobody asked.

- **`topo` and `blocks`, the dependency graph as two sortable numbers** —
  `links:` already carried the true working order of a backlog and no sort
  read it: `--sort ready` ranks on a boolean, separating ready from blocked
  without ordering either group, and `--sort depth` is folder nesting. A
  scoring framework only makes it worse, ranking a task on its own properties
  alone — so the decision five others depend on scores low for being expensive
  and sorts last. `topo` is position: 0 for a task nothing blocks, else one
  more than the deepest task it depends on, so ascending is a valid working
  order. `blocks` is leverage: how many tasks transitively wait on this one.
  Both are levels rather than queue numbers, so independent tasks tie and the
  next spec decides — `--sort topo --sort priority:desc` is graph order with
  priority inside a layer. Both are also filter fields (`mt ls 'blocks > 3'`),
  because they enter the same field registry that already gives `--sort` and
  the filter language one wiring. They read the same edges `ready` does
  (`depends-on` plus any kind a project flags `blocking = true`), and differ
  only on status: `topo` ignores it, since a finished dependency still sits
  before its dependent in the graph, while `blocks` counts only unfinished
  dependents and walks only through them — a done task blocks 0, so the number
  shrinks as work closes, which is the point. `mt check` still rejects a
  dependency cycle, but neither rank needs it to: a task on a loop keeps the
  level its resolved dependencies gave it, and nothing hangs.

- **`[handle] check-len`, the handle guard's width, per project** — a list
  handle is `<ordinal><check>`, where the ordinal is the task's position in the
  default view recomputed on every command and the check is derived from the
  task's stable id. The ordinal is what goes stale: a flip re-sorts the view
  under anyone still holding a handle, so the check is the only thing between a
  stale handle and an operation on the wrong task. One base-26 letter lets a
  wrong handle through about once in 26; `check-len` buys more at a character
  apiece, up to the six a 32-bit hash can fill. Omitted, it stays 1, so a
  project that says nothing keeps the handles it has. Raising it invalidates the
  short handles rather than accepting them — under `check-len = 2`, `2y` is
  refused with the expected length named, because `2y` is a valid handle for
  whichever task hashes to `y` at length 1, not a short spelling of `2yq`.

- **`key@path`, a third `mt add` token operator** — reads a slot's value out of
  a file, in the same grammar and both carriers as `=` and `+=` (`mt add "..."
  -- summary@notes.txt`, `--set summary@notes.txt`). It serves the long value:
  a `summary` or a `scope` awkward to type as a shell argument and not worth a
  whole `-f` document, and above all a `raw` slot fed a `.typ` written and
  checked before the task was filed, instead of markup that has to survive a
  shell argument. The operator carries bytes and converts nothing — the slot's
  KIND still decides, so a `text` slot markup-escapes them, a `raw` slot
  splices them verbatim, and any other kind coerces them and fails naming
  itself. Notably nothing turns Markdown into Typst, which is why the examples
  are `.txt` and `.md` is a trap. Trailing newlines are stripped and nothing
  else is, so `@` is a drop-in for the `--set "key=$(cat path)"` it replaces.
  The first operator separates, `=` or `@` whichever comes first, extending the
  rule that already made `summary=a=b` a value holding an `=`. No `key+@path`
  (a file is one value; a list goes through `-f` as an array) and no `key@-`
  (`-f -` is the stdin channel, and a YAML block scalar already carries a prose
  slot verbatim).
- **`check-secrets` gate** — gitleaks over the working tree, so a committed
  credential fails the build instead of waiting to be noticed. `.gitleaks.toml`
  extends the default ruleset with a walk bound naming the gitignored roots
  (unbounded, the scan walks `target/`: 1m44s over 2.09 GB, against 1.5s over
  7.5 MB) and one allowlist for the `outdatty.lock` sha256 digests
  `generic-api-key` reads as auth secrets — scoped to that rule, that file and
  the digest shape, never by fingerprint, which the next re-record expires. A
  control fixture runs through the real config first, because "no leaks found"
  is equally what an over-broad allowlist and a config that loaded no rules
  print; a third arm set-differences `git ls-files` against mmz's own expansion
  of the rule's inputs, so the enumeration standing in for "the whole tree"
  fails by name instead of going quietly false-fresh. History stays out of
  scope: `gitleaks git` reports the absolute paths this repo has already
  committed and would need a `.gitleaksignore` entry per past mint.
- **`assignee` as an `mt ls` column** — the field was filterable and sortable
  but could not be displayed, so a mixed listing could be narrowed by who works
  each task and never show it. It joins `LS_COLUMNS` between `due` and `title`,
  renders in the table and in `--format csv`, and reads `auto` visibility as
  "do the rows disagree": a listing already narrowed to one actor (`mt ls
  @agent`) does not grow a column repeating that name down every row, while a
  mixed one shows the split unasked. An unassigned row counts as its own value,
  so a half-assigned listing is contested. Naming the column does not override
  the rule — `[ls.columns.assignee] visibility = "visible"` does, matching how
  an explicitly listed `priority` still hides when nothing scores one.
- **`crap-baseline-check` gate** — the CRAP arm that never skips: it reads
  `crap-baseline.json` itself rather than the lcov, failing on an entry that
  carries a machine-local absolute path (or one climbing out of the repo with
  `..`) and on a baseline scoring no functions at all, which would pass the
  ratchet by reporting every function as new. Carries a positive control over
  a known-bad fixture, because a path filter matching nothing reads exactly
  like a clean baseline. gitleaks cannot cover this: its default ruleset's
  global allowlist matches `/home/<user>/` and drops any custom rule's finding
  on it, and an extending config can only add allowlists, never remove one.
- **`just fork <slug> [base]`** — stands up a worktree for a delegated agent:
  a new branch off `[base]` (default `HEAD`) at the sibling
  `../mindtape-<slug>`, `direnv allow`-ed with its dev shell already evaluated,
  and the base read back out of the created worktree and compared to the sha it
  was created from. Refuses a dirty `HEAD` — `git worktree add` copies commits,
  never the working tree, so forking off one hands the agent a base missing the
  changes on screen — and reports, without refusing, a base behind or diverged
  from the current checkout. Warms no build cache on purpose: `kache` already
  serves a fresh worktree's build at a 99.7% hit rate, and seeding a 17 GB
  `target/` on a filesystem where `cp --reflink` fails would cost a full copy
  per worktree.
- **`site-pages-fixture-check` gate** — pins the three failures
  `www/generate-site-pages.sh` and `check-doc-facts`'s pages arm exist to
  catch (two pages declaring the same route, a page with no `<page-meta>`
  block, a page missing from `www/utils/site.typ`'s NAV), each demonstrated
  by hand once when they landed and never proven again since. Runs the first
  two against a `cp -r www` replica of the real producer; the NAV
  set-difference was extracted to `.just/scripts/_nav-diff.sh` (now called by
  `check-doc-facts` itself) so the third is pinned against a synthetic pair
  of route lists instead of a full regenerate. Every case asserts a non-zero
  exit AND that the failure names the offending page or route.
- **`[tasks.estimate.drift] exclude`** — glob patterns naming paths the
  working-tree drift check never counts against a task's declared file
  budget. A generated output nobody hand-edits (`AGENTS.md`,
  `CONTRIBUTING.md`, `README.md`, `skills/mindtape/SKILL.md`,
  `docs/contributing/*.md`, everything under `www/generated/`) was counting
  double against the source that produced it — a two-file docs edit reported
  four — which trained authors to inflate their estimate rather than read the
  warning. Matched repo-root-relative with the same glob syntax as
  `[tasks.roots] derive-tags`. This repo's own `.mindtape/config.toml` now
  excludes its generated set.
- **Derived doc facts** — `www/generate-facts.sh` extracts the crate map and
  dependency versions (`cargo metadata`, `rust-toolchain.toml`,
  `[workspace.package]`, `lib/typst.toml`), the task DSL's core field list
  (read back off `lib/prelude.typ`'s own `<mindtape>` marker), the `just check`
  gate table, and the `.linecop.yaml` caps into `www/generated/*.json`. The
  Internals and Task DSL docs pages now read the crate list and field list
  from those artifacts instead of restating them by hand — the drift that had
  Internals claiming "three general-purpose crates" for a Workspace that ships
  seven can no longer happen silently. `just check-doc-facts` set-differences
  each artifact against its hand-written prose dict
  (`www/utils/crate-notes.typ`, `www/utils/dsl-field-notes.typ`) and fails
  naming any entry with no prose.
- **Typst-sourced root docs** — `docs/src/*.typ` sources render to the
  repo-root Markdown they each declare, via typlite (`docs/generate-md.sh`,
  `just docs-md`; typlite ships in nixpkgs' `tinymist`, added to the dev
  shell). AGENTS.md is the first port: its crate lists, versions, and the
  Typst/`evalexpr`/`clap` major versions now read
  `www/generated/crate-map.json` instead of being hand-listed.
  `just docs-md-check` (in `just check`) regenerates into a temp directory and
  diffs against the committed file, so a hand-edit or a stale render fails
  naming the file rather than self-healing on regeneration. `.linecop.yaml`'s
  caps move onto the `.typ` sources — a generated `.md` cannot be edited to
  fix an oversize — and `outdatty.yaml`'s dependents repoint from AGENTS.md to
  `docs/src/agents.typ`, dropping it entirely where a fact is now derived.
  README.md is the second port: its centered `<p align="center">` header
  cannot survive typlite (`<p>` is silently stripped), so it is rebuilt from
  `<h1 align="center">`/`<h4 align="center">`, which both typlite and GitHub's
  HTML sanitizer preserve. Its `@local/mindtape` example version now reads
  `www/generated/crate-map.json` too, dropping README.md from
  `outdatty.yaml`'s `typst-pkg-version` group.
- **Five more Typst-sourced docs** — `docs/src/contributing/{gates,windows,
  coverage-crap,mutation-testing,signal-sweep}.typ` join `agents.typ` and
  `generated-docs.typ` as ports of the matching `docs/contributing/*.md`
  deep dives. `windows.typ` reads the pinned rustc version from
  `www/generated/crate-map.json`'s `rust-toolchain.channel` instead of
  hand-typing it; the other four have no fact to derive and port as
  content-preserving prose.
- **CONTRIBUTING.md and `skills/mindtape/SKILL.md` join the generated docs**
  — the last two hand-written root files, and the payoff for the two facts
  `www/generate-facts.sh` derived with no consumer of their own until now.
  `docs/src/contributing.typ`'s Quality Gates table renders from
  `www/generated/gates.json` (the sub-gates `just check` actually declares,
  in order): the Gate column and row order are derived, so a gate added to
  `just check` and left undocumented fails `just docs-md` naming it, and one
  removed from `just check` drops off the table with no edit at all — it
  caught `fmt-fixture-check`, a real gate the hand-written table had never
  listed. `docs/src/skill.typ` is the first source to use the
  `<mindtape-md>` block's `frontmatter` dict: `docs/generate-md.sh` queries
  it out and prepends it as YAML (`yq`, ahead of typlite, which cannot emit
  YAML) — SKILL.md's trigger-critical `description` passes through
  unchanged, verified byte-identical against the committed file. Both
  sources read their `@local/mindtape` example version from
  `www/generated/crate-map.json`, same as `agents.typ`/`readme.typ`.
  `.linecop.yaml` flips both files from a numeric cap to `exclude: true`
  (the cap moves to their `docs/src/**` sources, already pre-staged), and
  `outdatty.yaml` repoints their dependent entries at the two new sources —
  dropping neither group, since both still restate content (CLI flags, the
  full DSL shape, non-`just check` recipes like `cover`/`crap`/`release`)
  that none of the four generated-fact artifacts carries.
- **Size gate over the hidden dirs** — `.linecop.yaml` sets
  `include_hidden: true` (linecop 0.4.0), so the recipe bodies extracted into
  `.just/scripts/` are finally under the `Shell: 300` cap the extraction was
  justified by; they had moved from one blind spot (`Justfile`, no extension)
  into another (a dot-prefixed directory the walker skipped). `.mindtape/
  templates/*.typ` comes under the Typst floor with them. `just
  size-fixture-check` (in `just check`) locks the scan SCOPE rather than the
  caps: an oversized script at `.just/scripts/` inside a replica tree must be
  reported, and must go unreported with `include_hidden` deleted — a
  directory the walker never enters prints the same "All files within size
  limits." a clean tree does. The `check-file-size` mmz rule gains the
  `policy` and `examples` scopes it had been missing since Typst got a base
  limit, so a Typst file crossing its cap can no longer serve a fresh cache
  hit.
- **A declared read/write classification for the CLI (CQRS gate)** —
  `crates/mindtape-cli/src/command_class.rs` names every visible `mt` leaf
  command's class (`Read`, `Write`, or `Exempt` with a reason — `go`'s home)
  as data, replacing the hand-typed lists CONTRIBUTING.md's "Command
  Semantics" section used to carry, which had drifted twice: nine wrong
  entries (`mt show`/`mt export` never existed; `find`, `graph`, `views`,
  `worktrees`, `eval` were missing) were only caught by an agent asked to go
  looking. `cli_dump.rs` rides each leaf's class along on `mt _cli --json`
  (so it reaches `www/generated/cli.json` for free); its own tests
  set-difference `command_class::CLASSIFICATION` against the reflected
  command tree in both directions, failing naming the offender the moment a
  command joins the CLI unclassified or a classified one stops existing.
  `docs/src/contributing.typ` renders the two bullets from `fact("cli")`
  instead of retyping them — surfacing `config schema` and `preset ls` as
  read commands for the first time, neither of which had a test before.
  `tests/read_no_write_e2e.rs` generates one byte-identical-tree check per
  `Read`-classified command from that same JSON, on top of (not instead of)
  the hand-written per-flag variants already there.

### Changed

- **BREAKING: the `typst` slot kind is renamed `raw`.** The old name claimed
  more than the kind delivers. Nothing builds a Typst value out of a `typst`
  slot and nothing type-checks one — the value is pasted, and the only check
  reads it as top-level markup. That check covers a `#slot(…)` in the body,
  which is what the kind is for, but it cannot see a slot spliced into an
  argument position: there the value is an expression, and markup treats
  parens and `*` as ordinary text, so `(files: 2` and `*emphasis*` both parse
  clean and reach the file, after which `mt add` fails loading what it wrote.
  `raw` names the splice, which is the actual contract, and the gap is now
  documented rather than implied away.
- **BREAKING: a slot declaring an unknown `kind:` is an error.** It previously
  fell back to `string` silently, so a typo — or a template still saying
  `kind: "typst"` after the rename above — filled as a quoted, escaped string
  with exit 0 and no diagnostic anywhere. `fill` now fails before coercing any
  value, naming the slot, the unrecognised spelling, and the valid kinds. This
  is what makes the rename safe to take: an un-migrated template fails loudly
  instead of quietly writing a wrong task.
- **BREAKING: one `key OP value` token grammar for `mt add` values.** A value
  is `key=value` (set) or `key+=value` (append to a list slot), and the same
  token is accepted after `--set` or bare after `--` —
  `mt add "Practice arpeggios" -- estimate.files=2 tags+=docs`. Both carriers
  reach one parser, so the shorthand cannot drift into a second grammar. The
  `key[]=value` append spelling is retired: it encoded a verb in the key's
  suffix where nothing else did, and needed shell quoting to survive globbing.
  A `[]=` token now fails naming its replacement rather than being silently
  accepted. `+word` tag sugar is unchanged — it carries no `=`, so it cannot
  collide with a token — and a bare positional stays title text, which is why
  the `--` is required: `mt add "fix a=b parsing"` sets no slot.
- **Nesting a slot needs no new syntax.** A slot's name is an arbitrary
  string and values are looked up flat, so a template that writes
  `slot("estimate.files")` already accepts `--set estimate.files=2`. The dot
  is a naming convention for template authors; MindTape never parses it.
- **The docs lead with YAML for `-f`.** `-f` always accepted YAML (stdin and
  unknown extensions sniff the content, and YAML is a JSON superset), but
  every example was JSON — the shape that makes an author escape exactly the
  prose slots that need it least. The `-f` sections on Templates & slots and
  in the skill now open with a YAML heredoc and a block scalar.
- **`script-scope-check` grew a third arm, over `outdatty.yaml`.** Its
  `dev-docs` group lists `.just/scripts/*.sh` one by one and outdatty reacts
  only to files it lists, so a script never added was coupled to no dependent
  and reported by nothing — the arriving-is-enough hole the first arm already
  closes for `.mmz` scopes, one manifest over. The arm demands a decision
  rather than membership: a script is a `dev-docs` source or carries an
  exemption naming the reason no doc restates it (`_nav-diff.sh` is the one,
  left out on purpose), and a new script is neither, so it fails. An exemption
  that contradicts a source line, or names a file that is gone, fails too.
- **`www/gates.jq` no longer reports a check dependency that does not exist.**
  A recipe carrying `[group("gate")]` but absent from `check:` emptied one side
  of the membership diff, and `halt_error` writes its string with no trailing
  newline — so `www/generate-facts.sh`'s next stderr line landed after that
  empty colon and read as a dependency named
  `generate-facts.sh: refusing to install malformed gates.json`. An empty side
  now renders `(none)` and the message ends where it ends.
- **`crap-baseline.json` records repo-relative paths.** `cargo crap
  --workspace` reports the absolute member roots `cargo metadata` gives it and
  cargo-crap 0.3.1 has no flag to shorten them, so every one of the 1058
  entries published the minter's own machine layout. `just crap-baseline` now
  strips the workspace root after minting. Nothing is given up: cargo-crap
  keys its first matching pass on `(file, function, line)`, so a baseline
  minted under one checkout root already missed that pass under CI's and
  paired through the suffix-aware pass instead — 1056 of 1058 entries pair as
  unchanged, and the two that do not are one file's two same-named
  `fake_editor` functions, ambiguous by suffix under any root.

### Fixed

- **`mt add` wrote an invalid task before it validated one.** The render landed
  on disk and validation ran after, so a rejected add left the broken file
  behind and every subsequent `mt` command failed on it until someone deleted
  it by hand — one mistyped slot cost a repair rather than a retry. Observed
  filing a single task: `estimate=5` parses as a number where that template's
  slot wants `(files: 5)`, and the rejection that was right about the value was
  the one that made the board unreadable. `mt add` now validates the rendered
  source *before* it reaches the filesystem, at the exact path it would occupy
  — so folder-derived tags, nesting depth, and `links:` resolution against the
  rest of the project are the ones the written file would have — and writes
  nothing when that fails. The error names the offending field and ends
  `nothing was written`, with no note about repairing a file, because there is
  none. What made this possible without a file is `mindtape::check_candidate`,
  a public API the other writers (`mt recur`'s spawn, `mt import`) can adopt on
  the same terms; only `mt add` moves in this release. Two seams keep their old
  behaviour: `--no-verify` still writes whatever it was handed, and a
  destination outside the project root (`mt add --into /elsewhere`) has no
  project-relative path to evaluate a candidate at, so it still writes first
  and reports the file as kept. A rejected interactive `--edit` buffer is
  parked in a temp file and named on stderr, so the typing that the refused
  write would otherwise discard stays recoverable — outside every task root,
  where no later command reads it.

- **`mt ls @view -i` ignored the view's `columns`.** The interactive picker
  kept the raw project config and dropped the one a resolved listing folds a
  view's column set into, so it drew columns `mt ls @view` would not — against
  a module whose stated contract is rendering the same table `ls` prints. Both
  now build on the same resolved listing, which is also what carries the new
  `truncated:` line into the picker's footer.

- **`mt init` offered a recurrence syntax the binary could not expand.** The
  wizard's `rrule` choice writes `[tasks.recur] rrule = true`, but that key only
  opens the config gate — expanding a rule needs the engine behind the `rrule`
  cargo feature, which is off by default and which `just install-bin` does not
  build. So the composed config was accepted, `mt init` reported success, and
  the failure surfaced later and elsewhere: the first task authored with
  `recur: rrule("FREQ=MONTHLY")` failed `mt check` with "requires mindtape to be
  built with the `rrule` cargo feature (not compiled into this binary)". A real
  project lost that answer to a re-file with `every(months: 1)` and a comment in
  its own config warning the next reader off the wizard's own menu entry — the
  documentation you write when a tool contradicts itself. An axis choice now
  declares whether THIS build can honour it: unhonourable choices are absent
  from the menu, refused when typed, and refused by name and reason from the
  flag, so `--recurrence rrule` fails `mt init` naming the cargo feature instead
  of scaffolding a project that breaks on its first recurring task. An axis left
  with one honourable choice is not a question either, and is not asked. The
  variant stays compiled in both builds, so `--recurrence rrule` keeps parsing
  and keeps appearing in `--help` — a flag that vanishes reports "invalid value"
  and names the feature nowhere. Every other axis is untouched: availability
  defaults to available, and a build carrying the feature offers the choice as
  before (`just check-rrule`).
- **`just cover` printed 2137 `PASS` lines to report a green suite.**
  `cargo llvm-cov nextest` accepts `--status-level fail` and ignores it, so the
  flag `just test` has always carried never reached the coverage run: 2316
  lines, one per test, ahead of the table anyone actually reads. The level moved
  to `.config/nextest.toml`, where both runners read it — 179 lines, same
  verdict, and a failure still prints in full, which is what `fail` means rather
  than `none`. A flag that is accepted and ignored is worse than one that is
  absent, because the recipe then claims something the output disproves. The
  file is a `dev-docs` outdatty source and a `gates-meta` mmz input, since it
  decides what the suite prints and, the day it grows retries or a default
  filter, what the suite runs.
- **A reader that stops reading no longer panics `mt`.** `mt ls | head -3`, a
  pager quit part way, an `xargs` whose child failed — each closes the pipe
  under a command still writing to it, and libstd ignores `SIGPIPE`, so the
  write came back `EPIPE` and the prelude's `println!` unwrapped it into exit
  101 and a `failed printing to stdout: Broken pipe` backtrace naming a line in
  `library/std/src/io/stdio.rs`. A crash report, with an internal libstd
  address, for the normal end of a pipeline. A closed reader is now an ordinary
  write error: the first one mutes every later write and the command runs to
  completion, exiting on its own merits. Stopping the process at the broken
  write was the other candidate and is wrong for a tool that also writes files —
  `mt add`, `mt flip` and `mt rm` print as they go, so bailing out there would
  abort them half way, either claiming a success that never happened or
  reporting a failure for work that mostly landed. Only `BrokenPipe` is
  swallowed; every other write error still panics, because a full disk under
  `mt ls > out.txt` must not read as a clean run. Enforced at one seam rather
  than at the ~110 print sites: `main.rs` shadows `print!`/`println!` crate-wide
  with macros that route through `crate::stdout`, since a policy holding at some
  call sites is one no caller can rely on at any of them. The shadow is
  `cfg(not(test))` — libtest captures a test's output through `print!`'s
  internals, which a direct `io::stdout()` write bypasses.
  <br>Filed as `mt` exiting **141** on roughly one call in ten, and that part of
  the report was about something else: `mt` never raised `SIGPIPE`. It ignores
  the signal from startup, and a read command is single-threaded and spawns
  nothing, so there was no handler to restore and no worker to reach. The 141
  came from a `mt` shell wrapper on the reporter's `PATH`, whose staleness check
  ran `find … | sort -nr | head -1` under `set -euo pipefail`: `head` closes the
  pipe as soon as it has its line, `sort` dies of `SIGPIPE` if it has not
  finished writing, and `pipefail` inside a command substitution under `set -e`
  exits the wrapper 141 before it ever executes `mt`. That is the race behind
  the fraction, and behind the two symptoms that had ruled the binary in: no
  output at all, and an identical rate with stdout redirected to a regular file.
  `tests/broken_pipe_e2e.rs` asserts `mt`'s side of it — never a signal death,
  never 141 — so the binary can never quietly acquire the behaviour it was
  blamed for.
- **The `site-pages-fixture-check` replica no longer copies the docs site's
  build output.** Its snapshot was a faithful `cp -r www`, which carried
  `www/public/` (6.1 MB of built site and its pagefind index) and `www/.tola/`
  (1.9 MB of tola's vdom cache) into every one of the three variants. Neither
  is read by a page compile, both are gitignored, and both belong to the
  `docs-check` arm `just check` runs in PARALLEL with this gate — so the
  snapshot was a reader of a tree another arm rewrites underneath it, for 8 MB
  per variant of nothing. The copy now walks `www/` entry by entry and skips
  those two by name; an assertion beside it fails the gate if either reappears,
  so a revert to the whole-tree form cannot land unnoticed. Excluded by name
  rather than by copying an allow-list of what the pages need, because an
  allow-list owes a line for every directory `www/` grows and fails the
  positive control for an environmental reason on the day someone forgets one —
  the failure mode this gate exists to be free of. One claim in the report that
  filed this was measured and is false: neither `tola build` nor `pagefind`
  wipes `public/` on entry, and a sentinel file dropped there survives both, so
  this was never a `cp` racing an `rm -rf`. What is measured is that `tola
  build` rewrites every `.tola/cache/*.vdom` and `index.json` in place on each
  run, which leaves the copy walking a tree whose files another process is
  replacing — a question about tola's write discipline that the gate has no
  reason to be asking.
- **`site-pages-fixture-check` no longer fails on a cold checkout.** The gate
  replicates `www/` with `cp -r` and runs the copy's own
  `www/generate-site-pages.sh`, which needs `www/generated/*.json` — a
  gitignored directory written by the `docs-gen` arm, which `just check` runs
  in PARALLEL with this one. Its own header stated the assumption it broke on:
  that the directory "rides along in the same `cp -r`", already on disk by the
  time any arm reaches this gate. True of a warm checkout and only there. On a
  fresh clone or a `just fork` worktree — which warms no cache by design — the
  first page query died on `file not found` for `crate-map.json`, the positive
  control failed, and two of the three cases reported "failed, but never
  named": a red that is indistinguishable from the gate catching a real break,
  arriving on the run that decides whether a change is green, naming a gate the
  change never touched, and answered correctly by the same re-run a flaky-test
  habit would order. The gate now produces its own inputs, `check-doc-facts`'s
  idiom for the identical problem: `www/generate.sh` under
  `target/www-generated.lock`, with the `cp -r` snapshot taken inside the same
  lock hold and the lock dropped before the three replica compiles that need
  nothing from the repo. Stubbing the captures instead was rejected as the
  cheaper answer that costs the fixture — one `<page-meta>` query is a full
  page compile, so a stub owes every artifact any page reads at the top level
  and owes one more with every capture `www/generate.sh` grows; a stub that
  falls behind fails the positive control for an environmental reason, which is
  this bug again. Declaring a `docs-gen` dependency was rejected for a
  different reason: it releases the lock before the copy runs, and the copy
  reads a directory the concurrent `docs-check` arm `rm -rf`s on entry, so the
  ordering it fixes is not the only one that matters. Costs one
  `www/generate.sh`, 4s warm, against a verdict that used to depend on which
  arm finished first.
- **A wrong-typed required field is reported once, naming its type.** The shape
  channel and the `required` rules both fired on the same field: `status: "wip"`
  is a status the projection cannot read, so it degrades to absent, and
  `mt check` printed the type report and then `status: missing required status`.
  The second line was false — the author demonstrably wrote a status — and it
  sent them to add a field already on the page, the exact misdirection the shape
  reports were added to end. Every required core field a shape error empties
  read the same way (`priority`, `difficulty`, `estimate`, `assignee`, and any
  `[[tasks.fields]]` enum). A FIELD-level shape error now masks the `required`
  answer for that field, and it masks it at the INPUT to each check rather than
  as a filter over the finished violations: after the fact the only handle on
  the report to drop is its rendered message, prose no gate holds still, spelled
  independently by five checks, and which a genuinely different violation could
  share. An ELEMENT error is deliberately not masked — it leaves the field
  itself readable, so `estimate: (days: "two")` still reports both the
  unreadable `estimate.days` and the empty `estimate` the required rule demands,
  which are two true statements about one mistake.
- **A wrong-typed element inside a well-shaped field is reported, not
  dropped.** `Task::from_marker` caught a wrong-typed FIELD (`links: "a.typ"`,
  `estimate: "2 days"`) and stopped there: one bad entry inside an otherwise
  correct field was discarded in silence. `estimate: (days: "two")` loaded as
  no estimate, `priority: (8, 9)` as no priority, `kind: 7` / `assignee: 5` /
  `due: 42` as unset, `every(weeks: 2, on: ("mon", "funday"))` as a period
  narrowed to one day the author never asked for. Every projection now reports
  the position an author can count to in their own source — `tags[1]`,
  `estimate.days`, `priority.inputs.ease`, `status.date`, `recur.on[1]`,
  `recur.weeks` — the way `links[1][0]` already did, and `mt check` fails on it
  regardless of config, since a type mismatch is never a matter of policy. The
  entries that DO read survive, so one mistyped tag costs one report and not
  the field. A `none` dict entry still reads as an unset key (`ice(impact: 8)`
  passes `confidence: none` through by construction); a `none` array element is
  a value the author wrote, and is reported.
- **The card render stopped standing in for validation.** `tags: ("x", 7)`,
  `status: "wip"` and an `on:` set mixing types used to fail at *eval* — not by
  design, but because the prelude's card reached for `raw(tag)`,
  `status.at("name")` and `on.join(", ")` on values that had no such method.
  That message named neither the field nor the task, and would have vanished
  the moment the render stopped making the call. The render is total now (every
  accessor type-guards; `_scalar` falls back to `repr`), so those values reach
  the marker and get the report above. The prelude still rejects the argument
  shapes it would otherwise DROP before the marker — that half is unchanged,
  and remains the only thing it can usefully assert.
- **An injected argument brings its own separator.** `mt add` writes its
  `status:` (and `id:`) into whatever `task.with(…)` the template declares, and
  the insertion landed just before the closing paren — correct only when a
  trailing comma already sat there. A template written on one line
  (`#show: task.with(title: slot("title"))`) or simply without a trailing comma
  got `title: "…"status: proposed(…)` instead: a file that cannot parse,
  reported as `written task failed to load: expected comma` *after* it was on
  disk. The insertion anchors after the last argument's value now and carries
  its own comma, so the template's shape is the template's business; several
  insertions share that anchor and each arrives separated from the one before.
  An empty argument list is the opposite case and keeps its trailing comma
  instead of taking a leading one. No in-tree template is written that way,
  which is exactly why nothing caught it — the regression tests are the other
  shape, at every level: the splice, `add`'s header injection, and a
  single-line template filed end to end through the binary.
- **`$MINDTAPE_DATA_HOME` moved the install but not the load.** The variable
  replaced the data directory `mt install` writes the `@local/mindtape` package
  into, while resolution went one level below `mt` — through `typst-world`,
  which looked the directory up with `dirs` and had never heard of it. Point it
  at a directory and the package landed there while every task file kept
  importing the machine's own, so the one knob documented as working on every
  platform (the reason it exists is that `dirs` reads no environment on
  Windows) isolated a write and not a read. `mt` now hands the resolver the
  directory it settled on — via `World::with_package_root`, added in
  `typst-world` 0.3.1 — so the install and the load can no longer name
  different directories. Set the variable empty and no global package resolves
  at all, matching what empty already meant for the config layer; the harvest
  cache folds the directory into its build key, so moving it routes reads to a
  fresh cache instead of replaying markers harvested against another prelude.
  `just check-examples` uses that to prove each fixture reads the checkout's
  prelude, replacing an `XDG_DATA_HOME` that only isolated the gate on Linux.
- **Example fixtures resolved the prelude from a machine-global symlink.**
  `@local/mindtape` resolves through one directory per machine — pointed at
  whichever checkout last ran `mt install` — so `just check-examples` in a
  worktree or a second clone validated `examples/learn-piano` and
  `examples/minimal` against the MAIN tree's `lib/prelude.typ`, not the one
  beside them, and the harvest cache (whose build key folds each override's
  file contents, and with no override folds nothing about the prelude) would
  not even re-evaluate. A fork editing the DSL therefore gated its fixtures
  against a prelude it had not changed. Both fixtures now declare a
  `[[typst.local-packages]]` entry pointing at this checkout's `lib/`, and the
  gate runs with the machine-global directory emptied so the entry cannot
  silently rot back — which also makes the gate, and `just check` as a whole,
  pass on a machine that never ran `mt install`, CI included. The repo's own
  root already resolved correctly: the loader auto-registers `<root>/lib` when
  its `typst.toml` names `mindtape`. `just fork` no longer prints its note
  about the symlink, which existed only to make the old behaviour visible.
- **`[tasks.estimate] units` is checked on open tasks only.** `check_estimate`
  checked the unit list before the terminal exemption the bounds already had,
  so a project retiring a unit failed `mt check` against every closed task
  that ever carried it — the list governing the work still to do was policing
  finished history too. The unit check now sits behind the same terminal
  return as the bounds: a closed task keeps whatever unit it was estimated in,
  and retiring a unit is one edit to `units`, refused only on an open task
  still carrying it.
- **`[tasks.tags] allowed` is checked on open tasks only.** `check_tags`
  checked authored and folder-derived tags on every task regardless of
  status, so dropping a tag from `allowed` failed `mt check` against every
  closed task ever tagged with it. It now skips terminal tasks, matching the
  estimate unit check: a closed task keeps the tags it was filed with,
  authored and derived alike, and dropping a tag refuses only an open task
  still carrying it.

## [0.3.0] - 2026-08-10

### Added

- **`mt find TERM…`** — sugar over the filter language, and deliberately nothing
  more. `mt find login bug` desugars to `contains("login") && contains("bug")`
  and runs the same code path as `mt ls`, so every `ls` flag forwards verbatim
  and the desugared expression lands in the `applied` facet — visible in the
  table footer and in `--json`, never a hidden query. There is no flag `ls`
  lacks and no second matcher: field targeting stays in the filter language
  (`mt ls 'contains("bug", title)'`). Pluggable external search backends were
  weighed and rejected on measurement, not taste: over the 2000-task
  `examples/large` fixture a filtered `mt ls` costs ~90 ms against `rg -l`'s
  11 ms, but that 90 ms is loading and evaluating tasks rather than matching
  text, and a backend still needs them loaded to render a row. Only a
  persistent index would win, and that contradicts the daemon-free premise.
- **`mt ls --format path` / `--format path0`** — a bare list of task file paths,
  one per line or NUL-delimited, with no header, colour, hyperlinks or
  `applied:` footer: output for a program, not a reader. Paths render through
  the same core as the `file` column, so the two agree by construction.
  Previously the only way to hand a selected set of files to another tool was
  `mt ls --json | jq -r '.tasks[].file'` — a hard `jq` dependency for the most
  basic composition there is. `contrib/fzf-mt-go.sh` ships the picker recipe
  rather than a picker: `mt ls '' --format path | fzf | xargs mt go`.
- **`[ls] contains-fields`** — names the default `contains()` haystack, so a
  project states its own answer to what "the prose surface" means. Unset keeps
  the shipped `title`, `id`, `body`. Two text-only surfaces join the searchable
  vocabulary for projects that want them: `note` (every `links[].note`, prose
  that lives outside `body`) and `tags` (authored and derived both, as
  `has_tag()` already sees them). The shipped default stays narrow on purpose —
  a default applied uniformly inherits the strictest constraint of anything that
  could land in it, so widening it for everyone is the wrong move; opting in is
  a line of config. Validated at load against the same vocabulary an explicit
  `contains("x", field)` call uses: an unknown name is a config error naming the
  field and listing the valid set, never a silent drop that would leave a
  project believing it searches a surface it does not.
- **`$MINDTAPE_CONFIG_HOME` / `$MINDTAPE_DATA_HOME`** — cross-platform overrides
  for the per-user directories `mt` reads and writes: the global `defaults.toml`
  layer, `config.schema.json`, `presets/` and `worktrees.toml` under the config
  home, the installed `@local/mindtape` Typst package under the data home. Both
  replace the platform lookup wholesale for reading and for what `mt install`
  writes; set but empty means "no such directory" (the global layer is dropped,
  never silently resolved from the machine). The platform lookup reads the
  environment on Unix only — Windows goes through a known-folder API no variable
  can move — so the test suite now isolates the developer's own `mt` setup on
  every platform, and the global-config regression is no longer `cfg(unix)`.
- **Preset `[overlay]`** — a preset for `mt init` may carry a top-level
  `[overlay]` table holding raw config the wizard's axes cannot express (custom
  link kinds, named views, body-header rules). `mt init` deep-merges it over the
  composed base last, so it beats every axis fragment, then flattens the result
  to one `.mindtape/config.toml` — the loader never sees a literal `[overlay]`.
  A preset whose whole body is under `[overlay]` composes to exactly that
  overlay. Composition now also hard-errors when two non-overlay fragments each
  declare the same array-typed section (status legend, priority bands, difficulty
  anchors, roots), naming both sources instead of silently letting one win.
- **`mt preset save <name>` / `mt preset ls`** — capture the current project's
  `.mindtape/config.toml` and `templates/` as a reusable user preset under
  `~/.config/mindtape/presets/<name>` (one directory per preset, so a broken one
  breaks only itself), so a shape arrived at once travels into every later `mt
  init --preset <name>`. The saved config is a raw copy — no reverse-engineering
  of the wizard answers that produced it. `--force` overwrites an existing user
  preset; without it the command refuses and names what is already there. `mt
  preset ls` lists built-in and user presets together, marking which user
  preset shadows a built-in of the same name, and the default.
- **Flip hooks (`[[on.flip]]`)** — one mechanism for "when a task changes status,
  do something". Each entry pairs a `when` — a filter expression over the task as
  it stands before the write, plus an `into("status")` predicate for the status
  being stamped — with an ordered list of actions: `{ show = "text" }` prints a
  message, `{ list = "filter or @view", title = "..." }` prints the matching
  tasks (how an agent starting work sees what else is in flight, with no
  materialised index), and `{ run = "cmd", gate = true, message = "..." }`
  executes a command whose non-zero exit refuses the flip, prints its output, and
  stops: nothing is written for that task and later actions do not run. `--force`
  waives a failing gate and records the waiver in the status note, so an override
  stays legible. Gate commands run from the project root with
  `MINDTAPE_TASK_FILE`, `MINDTAPE_TASK_STEM` and `MINDTAPE_TASK_STATUS` in the
  environment. `show` and `list` fall silent under `mt flip --quiet` or when
  stdout is not a terminal; gates still run. Only `mt flip` runs actions —
  `mt check`, `mt ls` and every other read command execute nothing, since a read
  that shells out is a surprise in the editors and commit hooks `mt check` runs
  inside. The trust boundary is a `Justfile`'s or a git hook's: the command comes
  from the in-repo config and is not sandboxed. The `gated` preset ships both
  shapes — the open set on the way into `wip`, and a gate on the way into `done`.
- **Work-in-progress limits (`[[tasks.limits]]`)** — cap how many tasks may sit
  in a matched set at once and have `mt check` report a breach with the cap, the
  count and every task in the set. A limit is a `filter` plus a `max`, so no
  status name is hardcoded: the same primitive caps a status, a named view
  (`@stuck`), a priority band or a `kind`. Optional `message` replaces the
  default advice; `severity` is `"warn"` (default — a breach is a state of the
  backlog, cleared by finishing work, not by editing a file) or `"error"`, which
  fails the command for projects wanting a hard stop. Rests on the filter
  language's missing-field rule (an absent field is unequal to every literal), so
  a `kind == "x"` / `kind != "x"` pair partitions the whole backlog.
- **External artifact links (`[[tasks.links.kinds]] validate`)** — a link kind
  can now point outside the backlog: the commit, pull request or build that
  discharges a task. `validate` defaults to `local-task-path` (exactly today's
  behaviour, so every existing config is unchanged); the other bundled
  validators are `url` (RFC 3986), `git-sha` (hex, 7–64 chars — abbreviated
  SHA-1 through full SHA-256), `uuid`, `regex` (with a required sibling
  `regex = "..."` pattern; its absence is a config error) and `any`. Graph
  semantics follow from that one field rather than a second flag: a kind whose
  `validate` is not `local-task-path` is excluded from stem resolution,
  cardinality, cycle detection and readiness (`blocking`/`acyclic` are ignored
  on it), while note rules still apply. A malformed target is a `mt check`
  violation naming the kind, the validator and the value; `mt ls` never follows
  one, and `mt graph` renders it as its own dashed leaf node instead of
  traversing it.
- **typstyle-clean writes (`[format]`)** — `mt` now runs typstyle over the Typst
  source it writes (`add`, `flip`, `review`, `recur`, `import`), so surgical
  edits and slot fills land in one canonical shape instead of leaving drift like
  stray body indentation. Uses `typstyle-core` as a library — in-process,
  deterministic, idempotent (a clean file is unchanged beyond the edited span),
  and error-tolerant (malformed source passes through) — pinning the
  `typst-syntax` version already resolved via `typst-harvest`, so no second
  `typst` enters the tree. `[format] enabled = false` opts out, keeping writes as
  raw byte splices for minimal diffs.
- **JSON Schema for `.mindtape/config.toml`** — generated from the config
  structs (`mindtape::config_schema_json`, draft 2020-12), so it never drifts
  from the accepted shape and its field docs come from the type doc comments.
  `mt install` now also writes it beside the app config
  (`~/.config/mindtape/config.schema.json`; skip with `--no-schema`), and
  `mt init` scaffolds a config whose first line is a `#:schema` comment
  directive pointing at it, so a schema-aware TOML editor (tombi, Taplo) assists
  filling values. `mt init --schema URL|PATH` retargets the directive (e.g. the
  published URL for a portable, checked-in config); `--no-schema` opts out.
  `mt install-lib` is now equivalent to `mt install --no-defaults --no-schema`.
- **`mt import [FILE|-]`** — import canonical task-JSON (a file arg or stdin
  `-`; the bare array `mt ls --json` emits by default, or its `{"tasks": [...]}`
  envelope) as native `.typ` tasks. Maps each record onto the project's config
  policy (`mindtape::map_canonical`: status legend, priority/difficulty scale,
  `kind` enum, tag allow-list, the `x-<ns>` extras discipline), mints a
  destination stem from the title (`mindtape::stem_for_title`, disambiguated
  `-2`/`-3`/... on collision), emits it (`mindtape::emit_task`), then writes
  the whole batch before reloading and validating the destination as one set —
  so records that link to each other resolve. A record that fails validation
  is reported and removed, never left half-written; `--dry-run` stages into a
  temp mirror of the project instead of touching the real destination.
  `--into`/`--to` mirror `mt add`'s destination resolution; `--status` defaults
  a missing status, `--map OLD=NEW` renames a foreign status before legend
  validation, `--drop-unmapped` drops (instead of keeping and reporting) a tag
  outside the allow-list. No `--from`/`--via` source selector — a foreign
  tracker's export becomes canonical JSON via a `jq` adapter, upstream of the
  pipe. "`mt add` without a template, from a full record."
- **`contrib/jq/{taskwarrior,github}.jq`** — source adapters mapping
  `task export`'s JSON and `gh issue list --json
  number,title,body,state,labels,assignees,createdAt,url`'s JSON onto the
  canonical task-JSON schema `mt import` consumes. Pure structural jq, no
  Rust and no `config.toml` read (`mt import` owns the per-project policy
  map): status maps onto neutral names shared by both sources (`pending`/
  `open` -> `todo`, `completed`/`closed` -> `done`, `deleted` -> `cancelled`),
  Taskwarrior's H/M/L priority maps onto MindTape's default 0..9 scale
  (8/5/2), tags come from `project`+`tags` or issue `labels`, and
  uuid/urgency/annotations/issue-number/url/assignees carry through
  losslessly as `x-tw`/`x-gh` extras. Both escape a literal `#` (and `\`)
  before splicing source text into the raw-Typst `body` field, since an
  unescaped `#` would open Typst code mode and fail to compile. `jq` is now
  a flake devShell dependency; fixtures and goldens live under
  `contrib/jq/fixtures/`.
- **`mt graph [filter]`** — emit the task dependency graph as Graphviz DOT
  (default) or Mermaid (`--format mermaid`). Nodes are tasks labelled by title
  and status and filled by readiness (ready/blocked/terminal); edges are the
  typed `links:` relations, styled per kind. An optional filter scopes the
  induced subgraph (edges to out-of-scope tasks drop) and `--no-terminal` prunes
  closed tasks. A read-only projection reusing the same graph model that
  computes ready/blocked (`mindtape::edges`) — no new store, a diffable view of
  the same source.
- **Recurring tasks (`recur`, `mt recur`)** — a task's `recur: every(weeks: 1,
  on: (mon, wed, fri), anchor: "due", until: on(2027, 1, 1), count: 10,
  materialize: "spawn")` shapes a `(rule, anchor, until, count, materialize)`
  dict; no date math in the prelude — `[tasks.recur]` validates it and
  `mindtape::recur_dates` computes the next occurrence (chrono period
  arithmetic; month/year-end clamped). `rrule("FREQ=...")` shares the same
  shape as the escape hatch, gated behind `[tasks.recur] rrule` (parsed;
  expansion is a separate `rrule` cargo feature — see below). `mt recur X` is
  the only writer, never a side effect of a read (`mt ls`/`mt check` are
  unaffected, enforced by test): the default `spawn` materialize writes the
  next instance as a new file (stem minted by `[tasks.naming]`, or the
  built-in `{slug}-{date}` fallback) with
  a `recurred-from` edge back to its predecessor, refusing a second spawn
  unless `--force`; `mutate` refreshes the same file in place. `anchor:
  "completion"` (default) always advances from today; `anchor: "due"` catches
  up from the stale due date until strictly after today. `--dry-run` previews
  without writing; `--advance N` pre-materializes N future spawns bounded by
  `[tasks.recur] max-advance`; `--skip` advances the anchor/`count` without
  materializing anything. `mt done` stays a pure close on a recurring task —
  it never strips `recur` — and prints a notice pointing at `mt recur`.

- **`rrule(...)` expansion engine** — the `rrule("FREQ=MONTHLY;BYDAY=1MO")`
  escape hatch (see above) now expands, for the calendar-complex cadences
  `every()` cannot express: first Monday of the month (`BYDAY=1MO`), last day
  of the month (`BYMONTHDAY=-1`), every weekday (`BYDAY=MO,TU,WE,TH,FR`), and
  the rest of RFC 5545. Wrapped by a new `rrule` cargo feature (off by
  default) over the `rrule` crate; the common `every()` path stays free of
  its `chrono-tz` + `regex` dependencies unless a project actually authors an
  `rrule(...)`. `rrule(...)` is date-anchored only — it implies `anchor:
  "due"` and rejects `anchor: "completion"` (RFC 5545 has no
  completion-relative primitive); DTSTART is seeded from the task's `due` and
  the next occurrence is the rule's first instance strictly after it,
  bounded by the crate's own iteration cap so a pathological rule errors
  instead of hanging. `mt check` grammar-checks the raw rule string at parse
  time and quotes the offending rule in a malformed one's error.

- **Pinnable "today" (`$MINDTAPE_NOW`)** — every date `mt` stamps or reads —
  status/`reviewed` dates, `now()`/`stale()` in filters, the overdue marker,
  `mt recur` advances — resolves "today" once per process from `$MINDTAPE_NOW`
  (a `YYYY-MM-DD` date or an RFC 3339 timestamp; the date part is used), else
  the system clock (UTC). Pin it for deterministic output — docs generation,
  golden-file tests, a CI snapshot. A malformed value is a hard error naming the
  variable, never a silent fall-through. `$SOURCE_DATE_EPOCH` is deliberately
  not honoured: `mt`'s "today" is an operational query concept, not a
  build-artifact timestamp (Nix dev/CI shells set it to 1980-01-01).

- **Worktree override** — a global `-w/--worktree <path>` flag (and
  `$MINDTAPE_WORKTREE` fallback; the flag wins) points `mt` at a chosen project
  instead of discovering one by walking up from the cwd. The path must hold a
  `.mindtape/`; relative paths resolve against the cwd. Every command routes
  through the single discovery entry point, so `ls`, `add`, `check`, `flip`,
  `go`, `rm`, and `config` all honour it.

- **Worktree aliases** — name worktrees in an app-scoped
  `~/.config/mindtape/worktrees.toml` and select one with a leading `@name`:
  `mt @sn ls` expands to `mt -w <path> ls`. Entries are tried in order — an alias
  (`name` + `path`) matches by name; a `base` directory matches `@<token>` when
  `<base>/<token>` is a project, so one `base` covers a whole tree of worktrees.
  `~` expands to home; `mt worktrees` lists the configured entries. Only the
  leading argument is sugar, so the `@view` filter sigil is unaffected.

- **Named views** — declare reusable query bundles as `[views.<name>]` tables
  (a `filter` plus optional `sort`, `columns`, `description`). Reference one with
  an `@name` sigil: a bare `mt ls @broken` applies the view's filter, sort, and
  columns; `@name` inside a filter (`mt ls '@summer && !@broken'`) is the filter
  facet only, equal to the new `in_view("name")` predicate. New `mt ls --columns`
  picks columns for a run (`@view` for a view's set) and `--sort @view` splices a
  view's sort chain. New `mt views` lists and inspects views (`--json`). Views
  are validated at config load: filters parse, columns are legal, references
  resolve, and the reference graph is acyclic.
- **Conditional `ls` rules (`[[ls.rules]]`)** — bare `mt ls` adapts to ambient
  state: the first entry whose `condition` holds wins, composing `import-views`
  (last-wins) plus a manual override, else `default-view`. New env-only
  condition language: `on_branch(glob)`, `env(name)`, `weekday()`, `now()`.
- **Persistent content-hash task cache** — warm `mt ls`/`mt check` over a
  mostly-static backlog now read each unchanged task's harvested marker from a
  git-ignored `.mindtape/cache/` instead of re-evaluating it through Typst; only
  edited or new files re-eval (stacking with the parallel batch harvester on
  that cold path). Entries are keyed by the file's content hash under a build
  key (cache format + `mt` version + config + prelude/`lib` contents), so any of
  those changing busts the whole cache; a stale, corrupt, or wrong-format entry
  falls back to a live eval, never a wrong task. The cache is scratch — reads
  populate it without violating the read/write (CQRS) invariant, which now
  guards the authored tree only. Set `MINDTAPE_NO_CACHE=1` to bypass it.
- **Windows binaries** — every tagged release now attaches
  `mindtape-x86_64-pc-windows-msvc.exe` alongside the four Linux and macOS
  targets, and CI runs the test suite on `windows-latest`, so the platform is
  supported rather than merely unclaimed. The install page derives each
  download's filename (extension included) from the release matrix, so a new
  target never needs transcribing. `README.md` states the support level and the
  two seams that differ; the pty-driven terminal tests stay Unix-only, since
  their harness is util-linux `script`.
- **A fourth `just ask-refactoring` arm: undertested functions** — per-function
  cyclomatic complexity against per-function coverage (`--cc-floor 5`,
  `--fn-coverage-floor 80`), read from `cargo crap --format json` over the lcov
  `just cover` writes. It sees what the file-shaped coverage arm structurally
  cannot: an untested function inside a well-covered file, like `hval_kind` at
  40% covered inside a `model.rs` no gate reports. It bands on the conjunction of
  the two numbers, never on the CRAP score that combines them — that score
  degenerates into complexity alone above ~90% coverage, which is why the
  score-shaped arm was cancelled on measurement. The two coverage arms cannot
  file the same gap twice: this one skips any function whose file the file-shaped
  arm reports, by calling that arm's own predicate rather than a copy of it.

### Changed

- **`search()` and `contains()` collapse into one filter function** — the
  language carried two overlapping text predicates: `contains(field, "substr")`
  matched a single field, while `search("query")` was documented as full-text
  but read only `title` and `id`, never `Task.body` — which is loaded already
  and marker-free. One variadic function replaces both, needle first:
  `contains("bug")` sweeps the prose surface (`title`, `id`, `body`),
  `contains("bug", title, body)` names its own. Tags keep `has_tag()` and every
  scalar field keeps its comparison operator, so neither joins an implicit
  sweep. The field vocabulary derives from the `fields` registry plus the
  text-only `body`, so `contains()` never hand-matches names of its own; every
  named field is validated before any is matched, so an unknown name errors
  whatever the argument order and whether or not an earlier field already hit.
  Both retired spellings — the old `contains(field, "substr")` order and
  `search(…)` — are hard errors naming the replacement rather than silently
  accepted alternatives.
- **"Today" is the local calendar date, not UTC** — every date `mt` stamps or
  compares (`status`/`reviewed` dates, `now()` and `stale(N)` in filters, the
  overdue marker, `mt recur` advances) now resolves through the machine's local
  timezone (`$TZ` on Unix, the platform zone on Windows) instead of UTC epoch
  seconds. A task filed at 01:00 east of Greenwich used to be stamped the
  previous day, and the date filters skewed with it. `$MINDTAPE_NOW` still
  overrides everything, and its value is taken verbatim — a trailing RFC 3339
  offset shifts nothing, since the pinned value *is* the date to stamp. Dates in
  task files remain plain civil dates with no offset attached. Costs `chrono`'s
  `clock` feature (chrono resolves the zone itself, no `localtime_r`, so no
  multithreaded-soundness hazard).
- **`[[on.flip]]` `run` commands go through the platform shell** — `sh -c` as
  before on Unix, `cmd /C` on Windows. A `run` command is shell text rather than
  an argv, so it is written for the platform the project builds on, the same way
  the `Justfile` recipes and git hooks it stands in for are.
- **`mt add --edit` falls back to `notepad` on Windows** — the `[add] editor` →
  `$VISUAL` → `$EDITOR` chain is unchanged; only the last-resort floor is now
  per-platform, since `vi` does not exist there.
- **`jsonschema` dev-dependency drops its default features** — the schema under
  test is generated in-process and resolves no external `$ref`, so
  `resolve-http`/`resolve-file` bought nothing while pulling
  reqwest → rustls → `aws-lc-sys` into every test build (a C and assembly crypto
  build needing NASM on windows-msvc). No behaviour change; a much smaller
  `Cargo.lock` and faster test builds on every platform.

### Fixed

- **The pre-commit gate has a scoped path** — `.githooks/pre-commit` fused one
  expensive assertion (`mmz --is-fresh --tag gate`, which needs a full `just
  check` behind it) to two cheap ones (task formatting, `mt check`). In a fresh
  worktree there are no gate records, so committing one line meant running a
  cold Rust build plus a Typst docs site and a Pagefind index — and the only
  escape, `git commit --no-verify`, switched off all three. `MINDTAPE_GATE_SCOPE
  =<reason>` now skips only the freshness assertion; the cheap two always run.
  A new `commit-msg` hook refuses the commit unless the message carries a
  matching `Gate-Scope: <reason>` trailer, so a scoped commit can never look
  identical to a fully gated one in `git log`. The variable was chosen over
  scoping the assertion to the staged diff because `.mmz/config.yaml` documents
  its own scopes as deliberately coarse — a diff-to-rule mapping built on that
  would fail silently open, which is the bug being fixed, not a fix for it.
- **`mt rm` no longer suggests committing outside a git repository** — every
  successful removal ended with `review the changes and commit when satisfied`,
  including in projects with no repository, where there is nothing to commit
  and the hint names a tool the user deliberately does not have. Both emitters
  now route through one helper gated on the same repository discovery that
  already resolves the default `[rm]` strategy, rather than a second probe.
- **`--sort assignee` / `--sort reviewed`** — both fields were already
  filterable but silently rejected by `--sort` (`unknown sort field`) since
  `SORT_FIELDS` never listed them. Filter and sort now share one field
  registry (`crate::fields`), so a field wired in is filterable and sortable
  together — no second match statement to forget. As part of the same fix,
  every field now sorts a missing value consistently: previously a missing
  optional field (e.g. `kind`) sorted *first* ascending (a stray `""`
  fallback) while `priority`/`difficulty` sorted a missing value *last* —
  every field now follows the same last-ascending, first-descending rule.
- **`mt add` can satisfy a required strict-framework priority** — a project
  whose `[tasks.priority]` sets `framework` with `allow-manual = false` and
  `required` had no `mt add` invocation that could file a valid task unless it
  shipped a template declaring the framework call: naming the field failed
  (`unknown --set/-f key(s): priority`), and omitting it wrote the file and then
  failed validation on it. A strict framework has no scalar form, so the splice
  now writes the framework call itself — the generic `framework("ice", impact:
  slot("impact", kind: "float"), …)` spelling with one leaf slot per declared
  input (number → `float` with its bounds in the prompt, bool → `bool`, enum →
  `choice` over the declared values). The leaves carry no defaults, so an
  unfilled one is the usual missing-slot error before anything is written rather
  than a misprioritised task on disk. `mt init --priority ice-strict` composed a
  config in exactly this shape, so it used to be refused as an incoherent answer
  set; it now scaffolds like any other.
- **The DSL helpers reject arguments they used to drop** — `depends-on(link(
  "a.typ"))` built no edge at all: `link()` returns content, content is the
  trailing note, so the call yielded an empty edge array. Nothing downstream
  could see it — the marker carried an empty `links`, `mt check` passed and the
  task listed as ready — because the value was destroyed before anything the
  Rust layer validates. Every such drop is now an eval error naming the helper:
  a `link()`/non-string target, a second note block or a named argument in an
  edge helper (`depends-on`, `parent`, `recurred-from`, `supersedes`,
  `duplicates`, `related`, `edge`), a second/non-content note in a status
  constructor (`wip(2026, 6, 14, "note")`), and a positional input to a
  priority framework (`framework("ice", 8)`). The split it draws: the prelude
  asserts only what would never reach the marker, everything that does reach it
  stays `mt check`'s to report, where the config lives.
- **`mt add` escapes text slot fills into Typst markup** — a text slot carrying
  ordinary prose used to land in markup verbatim, so a value containing `*`, `_`,
  `$`, `#`, `[`, `]`, `<` or `@` produced a task file that failed to load the
  instant it was written (an asterisk opened strong emphasis and the task died
  with `unclosed delimiter`). Every fill source now routes through the same
  escaper status notes already used — `--set`, `-f`, the positional title, the
  `MINDTAPE_ADD_*` env vars and the `--edit` YAML buffer — while numeric, choice
  and date slots stay unescaped, being literals rather than markup. A write that
  still fails verification now says the file was kept on disk, instead of
  reporting `wrote <path>` and leaving a broken artifact behind unmentioned.

### Removed

- **`mt ls --all`** — pass an empty filter (`mt ls ''`) to drop the default and
  list everything. Omitting the filter applies the default view.
- **`[ls] default-filter`** — replaced by `default-view` (see Changed).

### Changed

- **`required` is a filter expression (breaking)** — `[tasks.priority] required`
  (and its `difficulty` / `estimate` / `[tasks.fields.<name>]` siblings) still
  takes `true` and `false`, but the third form is now any filter expression
  instead of the one-off `"non-terminal"` enum value: the field is required for
  exactly the tasks it matches (`"kind == \"bug\""`, `"@active"`). Rename
  `"non-terminal"` to `"!terminal"` — identical meaning, and no alias is kept, so
  the old spelling fails at config load with the replacement named. A malformed
  expression is one config error at load, not one violation per task.
- **`mt ls --json` envelope (breaking)** — now `{"tasks": [...], "applied":
  {"rule": ..., "facets": {...}}}`, not a bare array; update a `jq '.[...]'`
  consumer to `jq '.tasks[...]'`.
- **`[ls] default-filter` → `default-view`** — the default `mt ls` listing is now
  a view, not a bare filter, so it carries sort and columns too, not only a
  filter. Omitting the filter resolves `[ls] default-view` (a view name), else the
  implicit `[views.default]` view, else match-all. The embedded floor ships
  `[views.default]` with the previous default filter, so bare `mt ls` is unchanged
  out of the box; redeclare `[views.default]` (or set `default-view`) to add a
  sort/columns bundle. `[ls] default-filter` is removed.
- **Repointed onto published crates.io substrate** — the seven extracted
  crates (`typst-world`, `typst-harvest`, `typst-cst`, `typst-edit`,
  `typst-cleanup`, `swhid-mint`, `artifact-gate`) are now `crates.io`
  deps (`0.1.0`), not in-tree path crates; workspace is down to `mindtape`
  + `mindtape-cli`. No CLI-visible change. Version reset to `0.2.0`.
- **`typst-harvest` split over a new `typst-world` crate** — the eval-only
  `typst::World` (file-provider seam, `@local` package resolution, dependency
  tracking) and the Typst-free `HVal` value tree now live in `typst-world`;
  `typst-harvest` is a thin layer that adds only `metadata()` marker harvesting
  on top. Internal reshape: no CLI-visible or behavior change. Render support in
  `typst-world` sits behind an off-by-default `render` feature, so eval-only
  consumers pull no layout/render/font dependencies.
- **Binary renamed `mindtape` → `mt`** — the installed executable is now `mt`;
  invocations become `mt ls`, `mt check`, `mt add`, etc. Crate names
  (`mindtape`, `mindtape-cli`), the `@local/mindtape` Typst package, the
  `.mindtape/` config directory, and the `MINDTAPE_ADD_*` env prefix are
  unchanged.
- **`mindtape::naming::disambiguate` returns a `Result` (breaking)** — stem
  collision disambiguation (used by `mt add`, `mt import`, `mt recur` spawn)
  now returns `Result<String, NamingError>` instead of looping forever
  against an `exists` predicate that never reports a free stem. Past
  `MAX_DISAMBIGUATE_ATTEMPTS` (10,000) numeric suffixes it returns the new
  `NamingError::Exhausted(stem, ceiling)` instead of hanging; callers surface
  it as an ordinary command error. Breaking change to a public library API —
  the crate is still on its 0.2 pre-release line.
- **`mt recur`'s due-anchor catch-up no longer hangs** —
  `mindtape::recur_dates::next_date`'s catch-up loop (advancing a stale `due`
  forward by the rule until strictly after today) is now bounded by
  `MAX_CATCH_UP_STEPS` (10,000 steps) and returns the new
  `RecurError::CatchUpExhausted(due)` instead of spinning forever when the
  rule never advances (e.g. `every(days: 0)`) or an expansion engine keeps
  returning its own reference date. Previously `mt recur` would hang in that
  case.
- **Recur/date internal cleanup** — the `rrule` cargo feature's off-state now
  lives in one stub module (`recur_rrule_off.rs`, selected by `#[path]`)
  rather than `#[cfg]` twin functions inside `recur.rs`/`recur_dates.rs`, so
  every function in those two files compiles under any feature set;
  `epoch_days_to_ymd` also dropped an unreachable negative-`shifted` branch of
  Hinnant's civil-from-days algorithm (the input is `u64`, so it is always
  `>= 719_468`). Both are dead-code/coverage hygiene, no behaviour change.

### Added

- **Status transitions** — a status entry may declare `to = [...]`, the allowed
  next statuses, turning the flat legend into a state machine. Opt-in: with no
  `to` on any entry the legend stays any-to-any as before. Once declared,
  `mt flip` rejects a move that is not a listed edge (printing the allowed next
  states; `--force` overrides), and `mt check` flags a task whose status is
  unreachable from the initial one under the graph. A same-status re-stamp and
  the first stamp on a status-less task are always exempt.
- **`mt ls --interactive` (`-i`)** — an interactive picker over the `ls`
  list: type a task's `#`-column ordinal to highlight it (reverse-video, prefix
  match), `Enter` opens the first highlighted task in the editor (`go`),
  `Ctrl+Enter` opens every listed task, `Space` re-evaluates the project, and
  `q` / `Ctrl+C` leaves. Table-only and TTY-only. Backed by `crossterm`, which
  negotiates the terminal keyboard-enhancement protocol so `Ctrl+Enter` /
  `Ctrl+Backspace` carry their modifier where supported (`Del` is the always-
  reliable clear).
- **`typst-cst` crate** — package-agnostic parse-plane structure extraction:
  the heading tree (depth, text, source span) parsed via `typst-syntax`, with no
  eval, World, or IO. A consumer-blind substrate for gating document structure.
- **`artifact-gate` crate** — plane-agnostic structural rule engine: match a
  generic node tree against three rule shapes — path cardinality
  (`require`/`forbid`/`unique`, exact or regex segments, direct-child or
  recursive), document ordering (one path's matches must precede another's), and
  closed-world allowlists (only named nodes permitted under a scope) — and emit
  diagnostics. Consumer-blind — no Typst, task model, or domain concepts.
  `mt check`'s body-header gating rides it over the `typst-cst` heading
  tree; speconaut can reuse it for its own artifact structure rules.
- **`typst-edit` crate** — consumer-blind in-place Typst source editing:
  parse-plane span queries plus an atomic, non-overlapping changeset (`apply`)
  over `typst-syntax`, no eval/World/IO. `find_calls(source, name)` locates any
  function call with its positional/named argument value spans and trailing
  `[…]` content block (`find_link_targets` is the thin link-only case); `line_of`
  maps an offset to a 1-based line. Foundation for `mt rm` link
  rewriting and the speconaut scanner collapse (see
  `docs/decisions/003-swhid-cleanup-tombstones-and-shared-edit-substrate.md`).
- **`swhid-mint` crate** — mint and resolve Software Heritage identifiers
  (SWHID, ISO/IEC 18670:2025). VCS-agnostic core (`render`/`parse`/`is_swhid_url`)
  plus a `git` backend feature (via the `gix` crate, no subprocess, no libgit2)
  that mints a revision-plus-path id for a tracked file and resolves one back. The
  tombstone substrate for `mt rm`.
- **`typst-cleanup` crate** — consumer-blind orchestration for retiring Typst
  artifacts: dirty-tree guard, per-file SWHID tombstone minting, and atomic
  inbound-`link()` rewriting, planned in memory before any write. The consumer
  supplies eligibility, link identity, and tombstone policy via a `Policy` trait.
  `mt rm` rides it; speconaut adopts it under a separate task. See
  `docs/decisions/004-gix-git-substrate.md` for the git substrate.
- **`mt rm`** — retire terminal-status tasks: deletes each terminal
  task file and rewrites every surviving `link("<stem>.typ")` that pointed at it
  into an SWHID tombstone `link("swh:1:rev:<commit>;path=<path>")`. Does not
  commit — the user reviews the diff and commits. Refuses a dirty working tree on
  a real run; `--dry-run` previews the plan without writing. Atomic — every mint
  and rewrite is planned in memory before a byte is written, so a wrong-shaped
  file or an unrecoverable commit aborts with no changes. `swh:` link targets are
  treated as history pointers by the loader/validator (not task graph edges).
- **`mt flip`** — set a task's status without opening the file:
  `flip <status> <task>...` stamps `<status>` dated today onto each target (the
  same shortcut/verbose rendering `add` uses, via `mindtape::status_value`).
  Every configured legend value also works as a top-level shortcut verb —
  `mt done 1b 2c` == `mt flip done 1b 2c` — generated from the legend
  (built-in subcommands win on a clash; a shadowing value is warned about);
  `mt flip --list` prints them. `-m/--note` sets the status note. Targets
  are resolved before any write (a status change reorders the default view), then
  valid edits apply and any resolution or post-edit validation problem is
  reported without rolling back the rest.
- **`mt go`** — open a task in the configured editor by handle, slug, or
  path (`[go] editor` → `$VISUAL` → `$EDITOR`; a `{task}` token expands to the
  resolved path, else the path is appended).
- **`mt review`** — stamp `reviewed:` with today's date on a non-terminal
  task; idempotent within the same day.

### The rewrite (2026-06-14)

Per-file rewrite. See `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.

#### Added

- **Per-file task model** — one task per file via `#show: task.with(...)`, with
  a `(name, date, note)` status, configurable enum/number core fields, task
  links, tags, and an `x-<ns>` extension seam for downstream tools
- **`typst-harvest` crate** — a package-agnostic, eval-only Typst World plus
  labelled-metadata-marker harvesting; a reusable substrate for file-based tools
- **`.mindtape/config.toml`** — per-project status lifecycle, enum/number field
  constraints (required-unless-terminal, bounds), tag allow-list, default filter
- **`mindtape check`** — config-driven validation incl. link resolution
- **`mindtape install-lib`** — installs the Typst library (once per machine)
- **`mindtape init`** — scaffolds project config (once per project)
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

[0.3.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.3.0
[0.2.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.2.0
[0.1.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.1.0
