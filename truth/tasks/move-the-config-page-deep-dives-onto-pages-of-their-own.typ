#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Move the config page deep dives onto pages of their own",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("www", "docs"),
  estimate: (files: 8),
  assignee: "agent",
  status: proposed(2026, 9, 11),
)

== Summary

`www/content/config.typ` sits at 925 of its 1000-line cap. Two tasks still in
this chain — `[[tasks.rules]]` and the allow-list dict references — both land
a section on that page, and either one alone takes it over.

The page already knows the fix: `[tasks.priority]` and `[views]` are deep
dives on pages of their own, with the Sections table pointing at them. The
remaining long sections have simply never been moved.


== Scope

+ Pick the deep dives whose length is buying the least on a page that is meant
  to be a map — the named-dictionaries section that just landed is one — and
  move each to its own page under `www/content/`, leaving the Sections table
  row as the pointer.
+ Keep every route that exists today working, and keep `www/utils/site.typ`'s
  NAV, the `page-meta` blocks and the generated page manifest in step: the
  docs manifest gate fails on a page missing from any of the three.
+ Move the prose, do not rewrite it. A reflow here is unreviewable against a
  page three pending tasks are about to edit.
+ Leave headroom, not just clearance: the page should come back under 85% of
  its cap, which is where `linecop --baseline 85` stops reporting it.

