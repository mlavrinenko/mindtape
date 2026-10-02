// docs/src/contributing/docs-site-build.typ — source for
// docs/contributing/docs-site-build.md. Rendered by docs/generate-md.sh via
// typlite, same as every other docs/src/*.typ source; see docs/src/lib.typ
// for the render path's constraints.
//
// Nothing here is derived: `--clean` is a literal flag in `.just/docs.just`,
// chosen because tola does not track a `json()` read as a page dependency.
// generated-docs.typ covers the other render path (the typlite Markdown
// sources); this page covers the tola site's build.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/docs-site-build.md",
)) <mindtape-md>

#doc-title[The docs site build]

Deep dive behind #just.docs-check(): why the tola site under `www/content/`
is rebuilt clean every time, and where that guarantee stops.

= The site build always rebuilds clean

The docs site build pays a cost of its own.
#just.docs-build() and #just.docs-check() pass `--clean` to `tola build`:
tola does not track a `json()` read as a page dependency, so an incremental
build can leave a page's rendered HTML unchanged after the fact it reads
changes — every fact-reading page (`config.typ`, `cli.typ`, `dsl.typ`,
`internals.typ`, `priority.typ`, and the config-slice pages) is exposed, and
#just.docs-check() would then validate HTML no current source produced
(truth/tasks/rebuild-the-docs-site-clean-so-a-changed-fact-cannot-validate-stale-html.typ).
`--clean` trades the incremental build's speed for that guarantee: every page
under `www/content/` recompiles on every `build`/`check`, not only the ones
whose own `.typ` source changed. #just.docs-serve() keeps the incremental,
watch-loop build on purpose — it is not a gate, and a full rebuild per
keystroke would defeat hot reload.
