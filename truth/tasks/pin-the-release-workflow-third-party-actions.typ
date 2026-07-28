#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Pin the release workflow third party actions",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("gating",),
  estimate: (files: 1),
  status: done(
    2026,
    7,
    28,
  )[three actions pinned to the SHAs ci.yml already vets],
)

== Summary

ci.yml pins third-party actions to commit SHAs and states that as policy; release.yml floats dtolnay/rust-toolchain\@stable, Swatinem/rust-cache\@v2 and softprops/action-gh-release\@v2. Release is the workflow holding the crates.io token and release write permission, so a moved tag there is the one that matters most.

== Scope

- Pin the three third-party actions to SHAs, reusing the ones ci.yml already vets.
- State the policy in release.yml too, so the next addition does not float.
- First-party actions stay on release tags, as ci.yml has it.
