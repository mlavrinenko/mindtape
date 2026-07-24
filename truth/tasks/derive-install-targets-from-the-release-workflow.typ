#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Derive install targets from the release workflow",
  priority: ice(
    impact: 8,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("www", "tooling"),
  estimate: (files: 4),
  status: done(2026, 7, 24),
)

== Summary

The quickstart lists only nix run and build-from-source while the release workflow ships prebuilt binaries for several targets and publishes to crates.io. Generate the install options from the workflow so the page cannot claim an artifact the pipeline does not produce.

== Scope

- Parse the release workflow build matrix and cargo publish steps in www/generate.sh
- Render the prebuilt target list and the cargo install line from that data
- Compute any stated count from the data so no hardcoded number can drift
- Add the yaml reader to the dev shell since it joins the docs build loop
