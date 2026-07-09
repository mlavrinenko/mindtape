#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add Pagefind search to the docs site",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  tags: (),
  status: done(2026, 7, 9),
)

== Summary

Wire Pagefind (static, post-build, SSG-agnostic) into the tola docs build so the site has client-side search. Deferred from the initial www rebuild; add once the page count justifies it.

== Scope

- TODO
