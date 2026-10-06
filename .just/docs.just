# .just/docs.just — the `docs` module (`mod docs '.just/docs.just'` in
# Justfile). The declaration spells the path out because a bare `mod docs`
# only looks for `docs.just` or `docs/mod.just` next to the file declaring it.
#
# `set working-directory := '..'` below makes a module recipe run at the repo
# root. Without it, just runs a module recipe in the module file's directory
# (verified, just 1.46: a recipe here running `pwd` prints `<repo>/.just`).
# Every recipe body in this file shells out with a repo-root-relative path
# (`www/generate.sh`, `docs/generate-md.sh`, …), and
# `[working-directory("www")]` resolves relative to the same directory. So
# without the setting the bodies miss their scripts, and the attribute looks for
# `.just/www` and fails with a confusing `could not find the shell: No such file
# or directory`. With it, a plain recipe runs at the repo root and
# `[working-directory("www")]` composes on top to give `<repo>/www` (both
# verified). Scripts extracted out of these bodies live in .just/scripts/ and
# are invoked the same repo-root-relative way.
#
# `[working-directory(source_directory() / "www")]` does not work (verified):
# just 1.46 accepts const expressions in that attribute but not function calls
# (`error: Expected ',', '=', or ')', but found '('`).
#
# `just docs::<name>` and `just docs <name>` both work (verified, just 1.43+).
# www/generate-facts.sh mangles the `::` invocation path to `-` for
# www/generated/just.typ, so `docs::md` becomes `docs-md`, the identifier every
# doc source already calls as `#just.docs-md()`. The mangling keeps this module
# boundary out of doc prose.
#
# Settings are per-file. A module does not inherit them from the Justfile that
# declares `mod docs` (verified: a module recipe with no `set quiet := true` of
# its own echoes its command line even though the root Justfile sets it). This
# file redeclares `set quiet` so `just check`'s docs::check and docs::md-check
# arms stay quiet like every other gate.
#
# `set unstable := true` makes this file valid as a standalone justfile. The
# root Justfile's own setting already covers the `mod` declaration. A bare
# `--fmt` pass does not descend into module files (verified, just 1.46), so the
# formatter gate points `--justfile` at each file in the Justfile's
# JUST_FMT_PATHS list. `--fmt` is unstable, and it reads that setting from the
# file it is pointed at, not from the Justfile that declares the `mod`. Without
# this line the file silently drops out of `just fmt` and `just fmt-check`.

set quiet := true
set unstable := true
set working-directory := '..'

# The same per-file rule holds for script-interpreter (verified the same way):
# a module does not inherit it from the root Justfile, so a `[script]` recipe
# added to this module would silently run under the default interpreter
# without this line. No recipe here uses `[script]`. The line is here so the
# first one runs under bash strict mode.

set script-interpreter := ["bash", "-eo", "pipefail"]

# just runs a module's first recipe when the module is named with no recipe.
# Without this recipe a bare `just docs` would run `gen`: a full fact
# regeneration, and a cold release build the first time, from a command that
# reads like a question about what the module holds. Listing is the right
# answer to a bare namespace, as it is at the repo root. The recipe that leads
# this file is what a bare `just docs` runs, so this one has to stay first.

[doc("List available recipes")]
[group("docs")]
default:
    @just --list docs

# Capture live `mt` output for the docs (www/generated/*.txt), so the site's
# CLI reference, help, version, and transcripts cannot drift from the binary.
# Builds `mt` debug, runs it against examples/api-service at a pinned clock.
#
# generate.sh itself derives the crate/DSL/gate/linecop facts
# (www/generate-facts.sh) and then the page manifest (www/generate-site-pages.sh)
# at its own end, in that order. It cannot run them before the captures:
# generate.sh `rm -rf`s the whole generated/ dir on entry, which would erase
# anything written first. The page manifest is a full compile of every
# www/content/*.typ page, which needs both generate-facts.sh's crate-map.json
# (site.typ's PKG_VERSION) and generate.sh's own captures (hero-source.typ,
# cli.json, …); see www/generate-site-pages.sh's header. The explicit second
# call below is a redundant, idempotent, lock-protected re-run. It makes `gen`
# produce the facts without depending on generate.sh's trailing call having
# been the last step.

[doc("Capture live mt output and derive doc facts")]
[group("docs")]
gen:
    bash www/generate.sh
    bash www/generate-facts.sh

# Build the documentation site (www/ -> www/public/mindtape) with tola, then
# index the built HTML with Pagefind (bundle -> public/mindtape/pagefind/),
# then write each page's Markdown copy and llms.txt (www/generate-md.sh).
# gen runs first so the injected transcripts reflect the current binary.
#
# `build` passes `--clean` because tola does not track a `json()` read as a
# page dependency. A page whose only source is www/generated/*.json (every
# fact-reading page — config.typ, cli.typ, dsl.typ, internals.typ,
# priority.typ, and the config-slice pages) keeps its last-rendered HTML after
# a fact changes, so an incremental build is byte-identical and wrong.
# `--clean` forces every page to re-render on every `build` and `check`, so a
# build never validates HTML that the current facts did not produce.
# docs/src/contributing/generated-docs.typ states what that costs.

[doc("Build the docs site, its Markdown copies and llms.txt")]
[group("docs")]
[working-directory("www")]
build: gen
    tola build --clean && pagefind --site public/mindtape --silent && bash generate-md.sh

[doc("Serve the docs locally with hot reload (no search index)")]
[group("docs")]
[working-directory("www")]
serve: gen
    tola serve

# Build the docs, index with Pagefind, write the Markdown copies and llms.txt,
# then validate internal links + assets. Pagefind runs before validate so the
# /pagefind/ bundle exists on disk. The Pages workflow deploys what this
# recipe builds, so the Markdown copies ship with the HTML.
#
# `tola build`'s `[build]` progress bar and `tola validate`'s `[sitemap]`/
# `[validate]` lines print on every run, success included. Verified against
# tola 0.7.1's `build`/`validate` `--help`: neither subcommand offers a quiet
# flag, `--color never` does not touch them (the escape sequences are the
# progress redraw, not colour), and `CI=true`/`NO_COLOR=1` make no
# difference either. So each is captured and replayed only on failure. A
# clean gate run prints nothing, and a failing one prints tola's report.
#
# It regenerates through `mmz just docs::gen` rather than depending on `gen`,
# so the four gates that read www/generated/ share one run per tree: the lock
# makes the others wait for it, and mmz then finds the record fresh
# (.mmz/config.yaml, the `just docs::gen` rule).
#
# `--clean` is here for the same reason as in `build` above: this gate runs
# `tola validate` over the built HTML, so an incremental `tola build` here
# would let `check` pass on a page that no current fact produced.

[doc("Build, index, and validate the docs site and its Markdown copies")]
[group("docs")]
[group("gate")]
[working-directory("www")]
check:
    cd .. && mkdir -p target && flock target/www-generated.lock env MINDTAPE_GENERATED_LOCK=1 mmz just docs::gen
    out=$(tola build --clean 2>&1) || { printf '%s\n' "$out"; exit 1; }
    pagefind --site public/mindtape --silent
    bash generate-md.sh
    out=$(tola validate 2>&1) || { printf '%s\n' "$out"; exit 1; }

# Render docs/src/*.typ to the repo-root Markdown each source declares
# (README.md, AGENTS.md, CONTRIBUTING.md, skills/mindtape/SKILL.md,
# docs/contributing/*.md) with typlite. See docs/generate-md.sh's header for
# what typlite can and cannot do, and docs/contributing/generated-docs.md for
# the pipeline this feeds and how to add a source.

[doc("Render docs/src Typst sources to repo-root Markdown")]
[group("docs")]
md:
    bash docs/generate-md.sh

# Freshness gate for md. It regenerates into a temp directory and diffs each
# rendered file there against the committed copy at its real path. It does not
# regenerate in place and read `git status`/`git diff`, because an in-place
# regenerate would overwrite a hand-corrupted committed file before the diff
# ran, and the gate would read clean. A temp render diffed against the
# untouched committed tree fails a drifted file by name every time.

[doc("Fail if generated Markdown has drifted from docs/src")]
[group("docs")]
[group("gate")]
md-check:
    bash .just/scripts/docs-md-check.sh
