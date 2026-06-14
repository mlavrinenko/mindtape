#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Redesign www task examples for the per-file model",
  status: done(2026, 6, 14),
  priority: "normal",
  tags: ("www", "docs"),
)

== Summary

`www/index.html` had its factual bits fixed in the rewrite (version, install
instructions, daemon wording), but the themed task examples still show the old
inline-checkbox model (`- [ ] text #due #high #id`). That no longer reflects how
MindTape works.

== Scope

- Replace the inline-checkbox examples with per-file `#show: task.with(...)`
  artifacts matching `README.md` and `lib/prelude.typ`.
- Refresh the surrounding copy so the landing page describes the per-file,
  daemon-free model.
- Content redesign only; no factual/version regressions.
