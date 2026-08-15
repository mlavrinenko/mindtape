// Hand-written rationale, one entry per `just check` gate, keyed by gate
// name. The gate list and the command each one runs come from
// www/generated/gates.json (www/generate-facts.sh, reading `just --dump`);
// this dict carries only what a machine cannot state — why the gate exists,
// or what it does beyond its own one-line command. An entry must never
// restate the command itself: docs/src/contributing.typ already renders it
// alongside, derived, so a copy here would be the same drift this file
// exists to end. `just check-doc-facts` fails naming any gates.json entry
// with no matching key here.
//
// A cross-recipe mention below goes through `#just.<name>()`, like every
// other doc source — a hand-typed recipe name is exactly the drift this
// milestone ended, and `just check-doc-recipes` scans this directory too.
//
// That makes this file a READER of www/generated/, which is only safe
// because the one gate that queries it — check-doc-facts — holds the
// www/generated/ flock across its typst queries as well as its own
// regenerate. It did not always: querying outside the lock let a concurrent
// docs::check arm rm-rf the directory mid-read, failing with "file not found"
// on a correct tree. If you add another consumer of this dict, it must hold
// that lock for its whole run (see www/generate.sh, docs/generate-md.sh, and
// the check-doc-facts recipe for the fd-held idiom) or read no generated
// file at all, the way crate-notes.typ and dsl-field-notes.typ do.
//
// Refs: truth/tasks/derive-the-gate-table-command-column-from-the-justfile.typ

// Relative, not root-absolute: this file is read under TWO different Typst
// roots — the repo (docs/src/contributing.typ, and check-doc-facts's query
// driver) and www/ (the docs site). A relative import resolves against this
// file either way; `/www/generated/...` would double to `www/www/...` under
// the second.
#import "../generated/just.typ" as just

#let gate-notes = (
  "fmt-check": [
    `cargo fmt` over Rust, `tombi format` over the project config TOML
    (`.mindtape/config.toml` and the per-example configs), `typstyle`
    over the Typst sources, and `--fmt` over the justfiles — the exact
    path lists are below the table. One list feeds both `fmt` and
    `fmt-check`, so a path is added in one place, never to a single
    recipe. The justfiles need a list of their own because a bare `--fmt`
    formats the root `Justfile` only and silently skips every `mod` file;
    each file listed gets its own `--justfile <file> --fmt` pass and needs
    `set unstable := true` of its own, since `--fmt` reads that setting
    from the file it is pointed at, not from the `Justfile` declaring the
    `mod`.
  ],
  "clippy": [
    Every lint is workspace `deny` (see Code Style above); this recipe is
    only the enforcement point, not a separate lint surface.
  ],
  "test": [
    The full workspace suite. #just.test("<filter>") (a developer query,
    no `mmz`) runs this recipe directly with the filter as `ARGS`, so a
    filtered run never pollutes the cached full-suite record
    #just.check() relies on.
  ],
  "machete": [Flags a workspace dependency no crate actually imports.],
  "check-file-size": [
    Per-language caps — and the tighter ones on the always-loaded docs — are
    in #link(".linecop.yaml")[`.linecop.yaml`], argued in place.
    `linecop --baseline 85` lists what is nearing its cap; run it before
    adding to an existing file.
  ],
  "check-dry": [
    `jscpd` — fails on any duplicated Rust block ≥70 tokens; fix by
    extracting a shared helper.
  ],
  "outdatty-check": [
    See Dependency Drift below for how `outdatty.yaml` couples a source to
    the dependents it must stay in sync with.
  ],
  "check-examples": [
    `mt check` against each example project: `examples/learn-piano/` (full
    config surface; feeds the docs generator and the skill's triage evals)
    and `examples/minimal/` (the floor config; feeds the skill's trap
    evals). Keep the recipe and the mmz `examples` scope in step. A fixture
    no gate validates rots silently.
  ],
  "check-doc-coverage": [
    Set-differences `mt _cli --json`'s reflected command names against
    `www/utils/cli-notes.typ`'s dict keys; fails naming any visible
    subcommand with no matching entry, so a new verb can no longer ship
    silently undocumented.
  ],
  "check-doc-facts": [
    Regenerates `www/generated/{crate-map,dsl-fields,gates,sweep-flags}.json`
    (`www/generate-facts.sh`) and set-differences each against its
    `www/utils/{crate-notes,dsl-field-notes,gate-notes,sweep-flag-notes}.typ`
    prose dict; fails naming any crate, DSL field, gate, or sweep flag with
    no matching entry — this file included.
  ],
  "check-doc-recipes": [
    Fails on a hand-typed recipe mention in a doc source — literal `just`
    followed by a name, whether or not the name is real — that should have
    gone through the generated module (`www/generated/just.typ`, called as
    `#just.check()` and friends) instead. The three literal shapes it scans
    for (a prose backtick span, a fenced example's leading command, a quoted
    config value) are argued in place in
    `.just/scripts/check-doc-recipes.sh`.
  ],
  "docs-check": [
    `docs-gen` — a dependency, not part of the command shown — captures live
    `mt` output first (`www/generate.sh`), so this gate always sees fresh
    CLI examples before `pagefind` indexes the build for client-side search
    and `tola validate` checks links and assets.
  ],
  "docs-md-check": [
    Regenerates every `docs/src/*.typ` source with typlite into a temp
    directory and diffs each against the committed Markdown it declares;
    fails naming any drifted file rather than healing it in place.
    #just.docs-md() refreshes it — see
    #link("docs/contributing/generated-docs.md")[docs/contributing/generated-docs.md].
  ],
  "fmt-fixture-check": [
    Synthesizes a deliberately mis-formatted TOML and Typst fixture in a
    temp directory and asserts `tombi format --check` and `typstyle --check`
    both reject it (and accept a reformatted one); locks the formatter gates
    themselves, so a formatter silently accepting drift could not let
    `fmt-check` pass on an out-of-sync editor save.
  ],
  "arg-pattern-check": [
    Dispatch-only self-test for the `[arg(...)]` patterns on
    #just.release("VERSION"), #just.bench("N"), #just.gen-bench("N"),
    #just.eject("PCT") and the five #just.ask-refactoring() thresholds:
    asserts each rejects a known-bad argument and accepts a known-good one,
    and that every declared default satisfies its own pattern (`fix-check`
    depends on bare `eject`; the sweep's defaults are the only copy of its
    thresholds). One of the sweep's cases is a value containing a space —
    the word-splitting that `[positional-arguments]` used to guard that
    recipe against, refused at dispatch now instead. Every assertion goes
    through the `--dry-run --yes` flag pair, so no recipe body ever actually
    runs — `release`'s included; see `.just/scripts/arg-pattern-check.sh`.
  ],
  "positional-args-check": [
    Static self-test for the four passthrough recipes —
    #just.test("<filter>"), #just.build("<args>"), #just.mutants("<args>")
    and #just.mutants-diff("<args>") — asserting each carries
    `[positional-arguments]` and forwards `"$@"`
    rather than interpolating `{{ ARGS }}` into its body, which would
    re-split an argument containing a space and re-glob one containing a
    wildcard. Reads the `--dump --dump-format json` dump and greps the
    scripts the recipes forward into; no recipe body runs. See
    `.just/scripts/positional-args-check.sh`.
  ],
  "check-crap": [
    Editing any source stales the lcov, so it skips on the commits that
    matter; it catches the free case only. #just.check-full() is the arm
    that is always correct.
  ],
)
