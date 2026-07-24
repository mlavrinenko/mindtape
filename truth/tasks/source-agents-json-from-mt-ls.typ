#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Source the agents JSON example from mt ls --json",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 8,
  ),
  tags: ("docs",),
  links: depends-on("generate-docs-cli-reference-from-the-mt-binary.typ"),
  status: done(
    2026,
    7,
    12,
  )[Generator captures `ls 'id == "fur-elise-polish"' --json | jq '.[0]'` → ls-task.json; the page reads it. Purged every wire-oauth / Wire OAuth mention site-wide (concepts stem, dsl example task, templates + agents `mt add` examples), re-themed to piano.],
)

== Summary

Last hand-authored `wire-oauth` example in the docs. The for-agents page
(`www/content/agents.typ`) shows a `mt ls --json` task as a hand-typed `json`
code block for the retired `wire-oauth` task — it survived the generate-docs
single-seam sweep because it is a `json` fence, not a `#terminal`. Every other
transcript is piano-themed and binary-sourced; this one drifts.

== Approach

- Generator (`www/generate.sh`): capture `mt ls 'id == "fur-elise-polish"'
  --json` sliced to one object (`jq '.[0]'`) → `www/generated/ls-task.json`. The
  `ls` path scores priority and derives folder tags, so the object is complete
  (unlike `mt eval`, single-file, which leaves priority null / derived_tags
  empty).
- Page: read the captured JSON into the `json` code block via `raw(read(...),
  lang: "json", block: true)`; reword the lede to the piano task.
- Purge the remaining `wire-oauth` prose stem in `www/content/concepts.typ`
  (identity-is-the-filename example) to a piano stem for consistency.

== Scope

- generator JSON capture; agents page reads it; concepts stem swap.
- verify the block matches live `mt ls --json` for the fixture task.
