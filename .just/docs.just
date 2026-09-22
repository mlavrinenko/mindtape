# .just/docs.just — the `docs` module (`mod docs '.just/docs.just'` in
# Justfile), split out under truth/tasks/split-the-justfile-into-modules.typ
# and moved here under
# truth/tasks/move-the-just-module-and-scripts-into-a-dot-just-folder.typ.
# That declaration spells the path out because this file no longer sits beside
# the Justfile: a bare `mod docs` only looks for `docs.just`/`docs/mod.just`
# next to the file declaring it.
#
# `set working-directory := '..'` below is load-bearing, not cosmetic. A module
# recipe runs with its working directory set to the MODULE FILE's directory,
# not the repo root (verified, just 1.46: a recipe here running `pwd` prints
# `<repo>/.just`). Every recipe body in this file shells out with a repo-root-
# relative path (`www/generate.sh`, `docs/generate-md.sh`, …), and
# `[working-directory("www")]` resolves relative to that same directory — so
# without the setting the bodies miss their scripts and the attribute looks for
# `.just/www`, failing with a confusing `could not find the shell: No such file
# or directory`. With it, a plain recipe runs at the repo root and
# `[working-directory("www")]` composes on top to give `<repo>/www` (both
# verified). Scripts extracted out of these bodies live in .just/scripts/ and
# are invoked the same repo-root-relative way.
#
# What does NOT work, also verified: `[working-directory(source_directory() /
# "www")]`. just 1.46 accepts const expressions in that attribute but not
# function calls — `error: Expected ',', '=', or ')', but found '('`.
#
# Invoked as `just docs::<name>` or `just docs <name>` — both work (verified,
# just 1.43+). www/generate-facts.sh mangles the `::` invocation path to `-`
# for www/generated/just.typ, so `docs::md` mangles to `docs-md` — the
# identifier every doc source already calls via `#just.docs-md()`. That
# mangling is what keeps this module boundary from touching doc prose.
#
# Settings are per-file, not inherited from the Justfile that declares `mod
# docs`: verified that a module recipe with no `set quiet := true` of its own
# echoes its command line even though the root Justfile sets it. Redeclared
# here so `just check`'s docs::check / docs::md-check arms stay quiet like
# every other gate.
#
# `set unstable := true` is NOT dead config, and it is not here for `mod` (the
# root Justfile's own setting covers the module declaration). It is what makes
# this file valid as a STANDALONE justfile: a bare `--fmt` pass does not
# descend into module files (verified, just 1.46), so the formatter gate points
# `--justfile` at each file in the Justfile's JUST_FMT_PATHS list — and `--fmt`
# is itself unstable, read from the file it is pointed AT, not from the
# Justfile that declares the `mod`. Delete this line and this file silently
# drops out of `just fmt` and `just fmt-check`.

set quiet := true
set unstable := true
set working-directory := '..'

# Same per-file rule, verified the same way for script-interpreter
# (truth/tasks/adopt-the-script-attribute-for-the-shebang-recipes.typ): a
# module does NOT inherit it from the root Justfile, so a future `[script]`
# recipe in this module would silently run under the default interpreter
# without this line. No recipe here uses `[script]` today — this is the
# setting arriving before the first user, same as `set quiet`.

set script-interpreter := ["bash", "-eo", "pipefail"]

# just runs a module's FIRST recipe when the module is named with no recipe of
# its own, so `just docs` used to run `gen`: a full fact regeneration, and a
# cold release build the first time, from a command that reads like a question
# about what the module holds. Listing is the right answer to a bare namespace
# for the same reason it is at the repo root. Whichever recipe leads this file
# is what a bare `just docs` runs, so this one has to stay first.

[doc("List available recipes")]
[group("docs")]
default:
    @just --list docs

# Capture live `mt` output for the docs (www/generated/*.txt), so the site's
# CLI reference, help, version, and transcripts cannot drift from the binary.
# Builds `mt` debug, runs it against examples/learn-piano at a pinned clock.
#
# generate.sh itself derives the crate/DSL/gate/linecop facts
# (www/generate-facts.sh) and then the page manifest (www/generate-site-pages.sh)
# at its own end, in that order and never before: generate.sh `rm -rf`s the
# whole generated/ dir on entry, which would erase anything written here
# first, and the page manifest is a full compile of every www/content/*.typ
# page, which needs BOTH generate-facts.sh's crate-map.json (site.typ's
# PKG_VERSION) and generate.sh's own captures (hero-source.typ, cli.json, …) —
# see www/generate-site-pages.sh's header. The explicit second call below is
# the same redundant, idempotent, lock-protected re-run this recipe has
# always made — it keeps `gen` honest about what it produces without
# depending on generate.sh's own trailing call to have been the last word.

[doc("Capture live mt output and derive doc facts")]
[group("docs")]
gen:
    bash www/generate.sh
    bash www/generate-facts.sh

# Build the documentation site (www/ -> www/public/mindtape) with tola, then
# index the built HTML with Pagefind (bundle -> public/mindtape/pagefind/).
# gen runs first so the injected transcripts reflect the current binary.
#
# `tola build --clean`: tola does not track a `json()` read as a page
# dependency, so a page whose only source is www/generated/*.json (every
# fact-reading page — config.typ, cli.typ, dsl.typ, internals.typ,
# priority.typ, and the config-slice pages) keeps its last-rendered HTML
# across a changed fact, incremental, byte-identical, and wrong (truth/tasks/
# rebuild-the-docs-site-clean-so-a-changed-fact-cannot-validate-stale-html.typ).
# `--clean` forces every page to re-render on every `build`/`check`, so a
# build can never validate HTML the current facts did not produce. What that
# costs is stated in docs/src/contributing/generated-docs.typ, not left for
# someone to discover by timing it.

[doc("Build the docs site and index it with Pagefind")]
[group("docs")]
[working-directory("www")]
build: gen
    tola build --clean && pagefind --site public/mindtape --silent

[doc("Serve the docs locally with hot reload (no search index)")]
[group("docs")]
[working-directory("www")]
serve: gen
    tola serve

# Build the docs, index with Pagefind, then validate internal links + assets.
# Pagefind runs before validate so the /pagefind/ bundle exists on disk.
#
# `tola build`'s `[build]` progress bar and `tola validate`'s `[sitemap]`/
# `[validate]` lines print on every run, success included — verified against
# tola 0.7.1's `build`/`validate` `--help`: neither subcommand offers a quiet
# flag, `--color never` does not touch them (the escape sequences are the
# progress redraw, not colour), and `CI=true`/`NO_COLOR=1` make no
# difference either. So each is captured and replayed only on failure: a
# clean gate run prints nothing, a failing one still prints tola's report.
#
# `--clean` for the same reason as `build` above: this is the gate that runs
# `tola validate` over the built HTML, so an incremental `tola build` here
# would let `check` pass on a page no current fact produced.

[doc("Build, index, and validate the docs site")]
[group("docs")]
[group("gate")]
[working-directory("www")]
check: gen
    out=$(tola build --clean 2>&1) || { printf '%s\n' "$out"; exit 1; }
    pagefind --site public/mindtape --silent
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

# Freshness gate for md: regenerate into a TEMP directory and diff each
# rendered file there against the COMMITTED copy at its real path — never
# regenerate in place and read `git status`/`git diff`. An in-place
# regenerate would heal a hand-corrupted committed file before the diff ever
# ran, so the gate would read clean on a fix that is no longer actually
# represented; a fresh temp render diffed against the untouched committed
# tree cannot self-heal that way, so a drifted file fails by name every time.

[doc("Fail if generated Markdown has drifted from docs/src")]
[group("docs")]
[group("gate")]
md-check:
    bash .just/scripts/docs-md-check.sh
