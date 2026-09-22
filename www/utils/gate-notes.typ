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
    `cargo fmt` over Rust, `tombi format` over the tracked TOML
    (`.gitleaks.toml`, `.mindtape/config.toml` and the per-example
    configs), `typstyle`
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
    The full workspace suite, run through nextest: one process per test, so a
    panicking or hanging test is isolated, and `--status-level fail` keeps a
    green run to a header and a summary — only a failure prints, per test,
    rather than every binary's own result block. nextest does not run
    doctests, and there is no separate `cargo test --doc` pass: the tree has
    none (`cargo test --doc` reports 0 tests in both crates), so a doctest
    added later must come with a pass of its own. #just.test("<filter>") (a
    developer query, no `mmz`) runs this recipe directly with the filter as
    `ARGS`, so a filtered run never pollutes the cached full-suite record
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
  "check-secrets": [
    gitleaks over the working tree under `.gitleaks.toml` — its default
    ruleset, a walk bound naming the gitignored roots (an unbounded scan
    walks `target/`: 1m44s over 2.09 GB, against 1.5s over 7.5 MB), and one
    allowlist for the `outdatty.lock` digests `generic-api-key` reads as auth
    secrets. History is out of scope: `gitleaks git` reports the absolute
    paths this repo has already committed and would need a `.gitleaksignore`
    entry per past mint. A control fixture is scanned first and must report an
    exact set of findings, because "no leaks found" is equally what an
    over-broad allowlist and a config that loaded no rules print — that one
    assertion pins the ruleset being live, both allowlists firing, each
    staying scoped to the rule and to the shape it silences rather than to the
    whole file, and none being wide enough to swallow the tree. A third arm
    set-differences
    `git ls-files` against mmz's own expansion of this rule's inputs, so the
    enumeration that stands in for "the whole tree" (`.mmz/config.yaml` has no
    negative glob, and `truth/tasks/**` may not be named by a gate-tagged
    rule) fails by name rather than going quietly false-fresh. A fourth arm
    (`.just/scripts/_gitleaks-bound-check.sh`, run against a synthesized
    control pair first for the same false-green reason) holds the walk bound to
    BOTH `.gitignore` files: every anchored entry either appears in the bound
    or carries a `# gitleaks-bound: exempt — <why>` line above it, and no bound
    pattern may match a file git tracks. Uncovered scratch has the walk descend
    into a tree a parallel arm is rewriting; an over-wide pattern silences a
    committed file, which is the only way an allowlist can cost coverage.
    Two entries are exempt and say so where they live — `/.cargo/*`, whose
    tracked `!/.cargo/mutants.toml` exception RE2 has no lookahead to spare,
    and `/result`, a nix symlink gitleaks never follows. The bound stops
    there — the `lstat` of the bounded
    directory itself precedes the allowlist check, so the scan holds the
    `target/www-generated.lock` flock across its walk rather than trusting the
    bound to stop the `ERR` a concurrent `rm -rf` logs on a run it still exits
    0, noise that teaches the reader to skim a security gate. What it cannot
    do is catch a machine-local path — that is #just.crap-baseline-check()'s
    job, argued there. See `.just/scripts/check-secrets.sh`.
  ],
  "outdatty-check": [
    See Dependency Drift below for how `outdatty.yaml` couples a source to
    the dependents it must stay in sync with.
  ],
  "check-examples": [
    `mt check` against each example project: `examples/learn-piano/` (full
    config surface; feeds the docs generator and the skill's triage evals),
    `examples/minimal/` (the floor config; feeds the skill's trap evals),
    and `examples/links/` + `examples/estimates/`, the single-subject
    fixtures the docs site slices its blocks out of. Keep the recipe and the
    mmz `examples` scope in step. A fixture no gate validates rots silently.
  ],
  "check-doc-coverage": [
    Set-differences `mt _cli --json`'s reflected command names against
    `www/utils/cli-notes.typ`'s dict keys; fails naming any visible
    subcommand with no matching entry, so a new verb can no longer ship
    silently undocumented.
  ],
  "check-doc-facts": [
    Regenerates
    `www/generated/{crate-map,dsl-fields,frameworks,gates,sweep-flags}.json`
    (`www/generate-facts.sh`) and set-differences each against its
    `www/utils/{crate-notes,dsl-field-notes,framework-notes,gate-notes,sweep-flag-notes}.typ`
    prose dict; fails naming any crate, DSL field, priority framework, gate,
    or sweep flag with no matching entry — this file included. A further arm
    set-differences the value-taking arguments `cli.json` reflects out of clap
    against the ones `crates/mindtape-cli/src/completion.rs` names a value
    source for, so a new completable argument cannot ship silently
    uncompleted. Both sides come out of the same reflection, so a mention the
    parser never sees cannot satisfy it; a `ValueEnum` argument is excluded
    because clap completes a `possible_values` list itself, and "no project
    value fits" counts as an answer — it is recorded with its reason rather
    than left blank.
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
    `docs-gen` — a dependency — captures live `mt` output first
    (`www/generate.sh`), so this gate always sees fresh CLI examples before
    `pagefind` indexes the build for client-side search and `tola validate`
    checks links and assets. Both `tola` steps print only on failure.
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
  "script-attr-check": [
    Strict-mode self-test in two arms. No recipe body may open with a `#!`
    shebang — every script body reaches the interpreter through `[script]`,
    so `set script-interpreter` applies to it — and every justfile must
    declare a setting that actually aborts a failed pipeline. `shebang` in the
    dump cannot tell the two apart (a `[script]` recipe reports `true` exactly
    as a `#!` body does, measured on just 1.46), so the first arm pairs that
    field with the absence of a `script` attribute. The second runs a
    `false | true` probe under a replica justfile carrying the file's own
    setting line verbatim, rather than matching its text: `-euo pipefail`
    would fail a literal match while being strictly better, and `-e` alone
    would pass a looser one while dropping the exact flag that let
    #just.count-tests() report a partial sum and exit 0. Every justfile is
    checked, not only those with `[script]` recipes today — a module does not
    inherit the root's setting, so `.just/docs.just`'s currently userless one
    is load-bearing and cannot be tidied away; see
    `.just/scripts/script-attr-check.sh`.
  ],
  "size-fixture-check": [
    Scan-scope self-test for #just.check-file-size(): an oversized shell
    script at `.just/scripts/` inside a replica tree must be reported by
    `linecop` reading this repo's own `.linecop.yaml`, and must go unreported
    with `include_hidden` deleted from it. The caps need no self-test — a
    wrong number is visible — but a directory the walker never enters reports
    the same "All files within size limits." a clean tree does, which is how
    every recipe body extracted into `.just/scripts/` sat uncapped while the
    extraction's stated reason was bringing them under a cap. Runs against a
    replica rather than the real directory because #just.check() runs its arms
    in parallel and a probe in the tracked tree would fail the real size gate
    mid-run; see `.just/scripts/size-fixture-check.sh`.
  ],
  "script-scope-check": [
    Enumeration lock over the two manifests that list `.just/scripts/` file by
    file, in three arms. Every `.just/scripts/*.sh` must be named by some
    `.mmz/config.yaml` scope as a literal path; every memoized rule whose
    recipe body REACHES a script must declare a scope covering it — the second
    because a script listed in a scope no rule reads passes the first while
    still serving a stale hit; and every script must be an `outdatty.yaml`
    `dev-docs` source or carry an exemption naming the reason no doc restates
    it. It exists because those scopes replaced a `.just/scripts/**` glob every
    rule carried, and the glob had the property an enumeration cannot have: a
    recipe body extracted out of the Justfile was covered by arriving. A
    wildcard never counts as naming a script, so putting the glob back
    satisfies nothing; and the third arm demands a DECISION rather than
    membership, because `_nav-diff.sh` is outside that group on purpose and
    forcing it in would ask for a review of seven dependents no change to it
    can rot.
    Arm two walks a call graph (`.just/scripts/_script-graph.sh`), so a body
    reached one hop further counts: through another script
    (check-secrets.sh's two gitleaks-bound scripts) and through a nested `just`
    (check-crap.sh shelling out to #just.crap()) alike, both being the same edge
    into the next body. It claimed direct invocation only until four such scopes had been
    wired by hand, which is what a missing derivation looks like from the
    outside. A wrong edge fails a CORRECT manifest, so the reading is narrow
    where it can over-read — a `just` counts only at command position, never
    inside the `echo` hints these scripts use to name a recipe they do not run —
    and loud where it cannot read at all: an unclassified flag or a target that
    is not a bare recipe name fails rather than being skipped, since a skipped
    call is the silent miss the walk exists to remove.
    `.just/scripts/_script-graph-control.sh` drives it over a synthetic tree
    carrying every one of those shapes at once, both directions being invisible
    from the real tree, where a correct answer and a broken reader print the
    same sentence. See `.just/scripts/script-scope-check.sh`.
  ],
  "site-pages-fixture-check": [
    Locks the docs manifest gate itself: two pages declaring the same route,
    a page with no `<page-meta>` block, and a page missing from
    `www/utils/site.typ`'s NAV were each demonstrated by hand once, when they
    landed, and nothing since has proven they still fire. Runs a temp-tree
    replica's own copy of `www/generate-site-pages.sh` for the first two
    (never the working tree — #just.check()'s arms run in parallel, so a
    mutation there would be read as real by a concurrent gate), and pins the
    third by calling `.just/scripts/_nav-diff.sh` directly against a
    synthetic pair of route lists rather than driving a whole
    #just.check-doc-facts() regenerate — that script is the extracted
    set-difference #just.check-doc-facts() itself now calls, so the thing
    this gate pins is the thing that actually runs. A positive control
    asserts the unmutated replica passes, and every case asserts both a
    non-zero exit and that the failure names the offending page or route,
    never the exit code alone. It regenerates `www/generated/` itself first
    (`www/generate.sh`, inside the flock, the snapshot taken before the lock
    drops) rather than reading what the parallel #just.docs-gen() arm may
    have left: that directory is gitignored, so on a cold checkout the
    replica had no captures to compile against and every case failed for want
    of them — indistinguishable, on the run that decides, from the gate
    catching a real break. The replica is copied entry by entry so it can skip
    `www/public/` and `www/.tola/` — gitignored build outputs no page compile
    reads, both written by the parallel #just.docs-check() arm, so copying
    them made this gate a reader of a tree another arm rewrites underneath it
    for 8 MB of nothing. An assertion beside the copy fails the gate if either
    reappears. See `.just/scripts/site-pages-fixture-check.sh`.
  ],
  "check-crap": [
    Editing any source stales the lcov, so it skips on the commits that
    matter; it catches the free case only. #just.check-full() is the arm
    that is always correct.
  ],
  "crap-baseline-check": [
    The CRAP arm that never skips: it reads the committed baseline itself
    rather than the lcov, so no coverage run has to be fresh for it to be
    right. It fails on an entry carrying a machine-local absolute path —
    what #just.crap-baseline() strips after minting, and what a re-mint by
    hand puts straight back — and on a baseline scoring no functions at
    all, which passes the ratchet by reporting every function as new.
  ],
)
