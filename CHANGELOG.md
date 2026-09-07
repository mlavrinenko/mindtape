# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
