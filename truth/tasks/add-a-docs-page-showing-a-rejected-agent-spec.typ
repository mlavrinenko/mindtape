#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add a docs page showing a rejected agent spec",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("docs", "www"),
  estimate: (files: 5),
  kind: "docs",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("rewrite-the-readme-and-docs-site-around-spec-linting.typ"),
)

== Summary

Once the README leads with spec linting, the claim needs one page that proves it: an agent-written task that fails `mt check`, and the output that names each field it broke. A hand-typed transcript would rot the first time a message changes, which is why every other CLI example on the site is captured live.


== Scope

Pick one plausible agent-written task that trips several rules at once, a non-verb title, a Done-when restating the gate, an oversized estimate, and commit it with a minimal config as a fixture project. Capture its `mt check` output in `www/generate.sh` beside the existing captures, add the page under `www/content/` showing the task and the capture, and register it in the NAV in `www/utils/site.typ`.

