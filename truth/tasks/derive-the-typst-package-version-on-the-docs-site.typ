#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "derive the Typst package version on the docs site",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("www", "docs", "tooling"),
  estimate: (files: 3),
  links: parent("de-slop-the-docs-and-close-the-drift-seams.typ")
    + depends-on("derive-the-docs-page-manifest-from-one-source.typ"),
  status: done(2026, 8, 16),
)

== Summary

`www/utils/site.typ` hand-types `PKG_VERSION = "0.2.0"` under a comment
telling the next reader to bump it in lockstep with `lib/typst.toml`. Every
`#import "@local/mindtape:..."` example on the docs site hangs off that one
literal, and the only thing holding it true is an `outdatty` group.

`www/generated/crate-map.json` already carries `typst-lib.version`, read from
`lib/typst.toml`. All three `docs/src` sources read it. The site is the last
consumer still typing it out.

== Scope

- `www/utils/site.typ`: read `typst-lib.version` out of
  `../generated/crate-map.json`, drop the sync comment.
- `outdatty.yaml`: drop `www/utils/site.typ` from the `typst-pkg-version`
  group, for the same reason `README.md` and `docs/src/agents.typ` are not in
  it — the file no longer states the fact.
- `outdatty.lock`: re-confirm.

== Constraint

`site.typ` is evaluated under two roots — `--root www` for the site build and
`--root .` for the page-manifest driver. A relative `json("../generated/...")`
resolves against the file either way; an absolute one does not.
