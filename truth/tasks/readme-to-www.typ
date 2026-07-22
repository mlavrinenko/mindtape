#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "readme to www",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  status: proposed(2026, 7, 22),
)

== Summary

Cut `README.md` down to a landing page and move its reference material into the docs site under `www/`, which already has pages for every section it duplicates.

The README sits at its 250-line `linecop` cap. Every new feature now costs a deletion somewhere else, which is standing pressure to under-document rather than to split — the preset work already had to reflow three unrelated paragraphs to buy the lines it needed. That is a structural problem and it will recur on every feature until the file stops carrying reference content.

No `docs/` directory. `www/` is the documentation site and the only place reference material should live.

== Scope

- README keeps: what MindTape is, why a file-based tracker, install, one worked example end to end, and links into the docs site. Target well under the cap so it has room to breathe.
- Move out, into the existing pages: the config layering explanation (the largest block) to `www/content/config.typ`, the DSL walkthrough to `www/content/dsl.typ`, the filter reference to `www/content/filters.typ`, the command tour to `www/content/cli.typ`. Merge rather than append — several of these already say part of it.
- Nothing is lost in the move. Anything the README says that the site does not is content, not duplication, and gets a home before the README line goes.
- Update the `outdatty` groups: README is a dependent of four groups today, and some of those dependencies move to the site pages with the prose.
- Re-check every inbound link. The README is what a repository visitor reads first, so a moved section needs a pointer, not a silent deletion.
- Tests: `just check` passes, including `check-file-size` and `outdatty-check`; the docs site builds.
