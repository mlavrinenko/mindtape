# docs.just — the `docs` module (`mod docs` in Justfile), split out under
# truth/tasks/split-the-justfile-into-modules.typ. Lives at the repo root,
# beside Justfile, not under docs/ — every recipe body here shells out with a
# repo-root-relative path (`www/generate.sh`, `docs/generate-md.sh`, …), and
# `[working-directory("www")]` resolves relative to THIS file's directory, so
# a module file anywhere but the repo root would break both.
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
set quiet := true
set unstable := true

# Capture live `mt` output for the docs (www/generated/*.txt), so the site's
# CLI reference, help, version, and transcripts cannot drift from the binary.
# Then derive the crate/DSL/gate/linecop facts (www/generate-facts.sh) — after
# generate.sh, never before: generate.sh `rm -rf`s the whole generated/ dir on
# entry, which would erase anything written here first.
# Builds `mt` release, runs it against examples/learn-piano at a pinned clock.

[doc("Capture live mt output and derive doc facts")]
[group("docs")]
gen:
    bash www/generate.sh
    bash www/generate-facts.sh

# Build the documentation site (www/ -> www/public/mindtape) with tola, then
# index the built HTML with Pagefind (bundle -> public/mindtape/pagefind/).
# gen runs first so the injected transcripts reflect the current binary.

[doc("Build the docs site and index it with Pagefind")]
[group("docs")]
[working-directory("www")]
build: gen
    tola build && pagefind --site public/mindtape --silent

[doc("Serve the docs locally with hot reload (no search index)")]
[group("docs")]
[working-directory("www")]
serve: gen
    tola serve

# Build the docs, index with Pagefind, then validate internal links + assets.
# Pagefind runs before validate so the /pagefind/ bundle exists on disk.

[doc("Build, index, and validate the docs site")]
[group("docs")]
[group("gate")]
[working-directory("www")]
check: gen
    tola build && pagefind --site public/mindtape --silent && tola validate

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
    #!/usr/bin/env bash
    set -eo pipefail
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    bash docs/generate-md.sh "$tmp"
    drift=()
    while IFS= read -r -d '' f; do
        rel="${f#"$tmp"/}"
        if [ ! -f "$rel" ] || ! diff -q "$rel" "$f" >/dev/null 2>&1; then
            drift+=("$rel")
        fi
    done < <(find "$tmp" -type f -print0)
    if [ "${#drift[@]}" -gt 0 ]; then
        echo "docs::md-check: drifted from docs/src/ (run \`just docs::md\` to refresh):" >&2
        for f in "${drift[@]}"; do echo "  - $f" >&2; done
        exit 1
    fi
    echo "docs::md-check: $(find "$tmp" -type f | wc -l) generated file(s) match committed"
