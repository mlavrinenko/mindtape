set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[private]
list:
    @just --list docs

[doc("Capture live mt output and derive doc facts")]
gen:
    bash www/generate.sh
    bash www/generate-facts.sh

# `--clean`: tola does not track a `json()` read as a page dependency, so an
# incremental build keeps the HTML of a page whose facts changed.

[doc("Build the docs site, its Markdown copies and llms.txt")]
[working-directory("www")]
build: gen
    out=$(tola build --clean 2>&1) || { printf '%s\n' "$out"; exit 1; }
    pagefind --site public/mindtape --silent
    bash generate-md.sh

[doc("Serve the docs locally with hot reload (no search index)")]
[working-directory("www")]
serve: gen
    tola serve

# Regenerates through mmz, not a `gen` dependency, so the gates that read
# www/generated/ share one run per tree.

[doc("Build, index, and validate the docs site and its Markdown copies")]
[group("gate")]
[working-directory("www")]
check:
    cd .. && mkdir -p target && flock target/www-generated.lock env MINDTAPE_GENERATED_LOCK=1 mmz just docs::gen
    out=$(tola build --clean 2>&1) || { printf '%s\n' "$out"; exit 1; }
    pagefind --site public/mindtape --silent
    bash generate-md.sh
    out=$(tola validate 2>&1) || { printf '%s\n' "$out"; exit 1; }

[doc("Render docs/src Typst sources to repo-root Markdown")]
md:
    bash docs/generate-md.sh

[doc("Fail if generated Markdown has drifted from docs/src")]
[group("gate")]
md-check:
    bash .just/scripts/docs-md-check.sh

[doc("Fail if a visible mt subcommand has no CLI doc entry")]
[group("gate")]
coverage:
    bash .just/scripts/check-doc-coverage.sh

[doc("Fail if a derived crate, DSL field, framework, or sweep flag fact has no doc entry")]
[group("gate")]
facts:
    bash .just/scripts/check-doc-facts.sh

[doc("Fail on a hand-typed just recipe mention outside the generated module")]
[group("gate")]
recipes:
    bash .just/scripts/check-doc-recipes.sh

[doc("Fail on history words, all-caps emphasis, an em dash in site text or a typed product name in doc prose, config comments and schema descriptions")]
[group("gate")]
prose:
    bash .just/scripts/check-doc-prose.sh

[doc("Fail on a code block typed by hand into a doc source")]
[group("gate")]
blocks:
    bash .just/scripts/check-doc-blocks.sh

[doc("Fail if a doc code span names an unknown mt command, flag or config key")]
[group("gate")]
spans:
    bash .just/scripts/check-doc-spans.sh

[doc("Lock the docs manifest gate against three known-bad fixtures")]
[group("gate")]
pages-fixture:
    bash .just/scripts/site-pages-fixture-check.sh

# The sources only: a render beside its source would pair every sentence with
# itself.

DOC_PROSE_PATHS := "www/content/*.typ www/content/cli/*.typ www/utils/*.typ docs/src/readme.typ docs/src/agents.typ docs/src/skill.typ skills/mindtape/references/*.md"

[doc("Report near-duplicate prose across the doc sources")]
dup-prose:
    for f in {{ DOC_PROSE_PATHS }}; do jq -n --arg f "$f" --rawfile t "$f" '{file:$f,text:$t}'; done | jq -s -r -f contrib/dup-prose.jq
