#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "verify the generated README header renders on GitHub",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("docs",),
  estimate: (files: 1),
  links: related(
    "generate-the-root-markdown-from-typst-with-typlite.typ",
  )[the port that rebuilt the header this checks],
  status: proposed(2026, 8, 13),
)

== Summary

README.md's centered header could not survive typlite: it strips \<p align="center"> silently. It was rebuilt from \<h1 align="center"> and \<h4 align="center">, which both typlite and GitHub's HTML sanitizer preserve (\<center> survives typlite but GitHub strips it). Every automated gate passes, but no gate can see github.com — only a human loading the repo page can confirm the logo, title, tagline and doc links still center, and that the h4 tagline does not read as a heading.

== Scope

- Load the repo page on GitHub after the next push and compare against the pre-port render
- Confirm: logo centered, title centered, tagline centered, the Documentation/Quickstart/Comparison links centered
- The tagline and link rows are h4 now, so they render bold where they used to be body weight — decide whether that is acceptable or worth another pass
- Record the outcome here either way; a failure gets its OWN task, linked back to this one and to the port
