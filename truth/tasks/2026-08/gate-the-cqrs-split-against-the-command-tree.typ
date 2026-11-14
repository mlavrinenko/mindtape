#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "gate the CQRS split against the command tree",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("cli", "tests", "gating", "docs"),
  estimate: (files: 3),
  links: parent("de-slop-the-docs-and-close-the-drift-seams.typ"),
  status: done(2026, 8, 17),
)

== Summary

CONTRIBUTING.md's read/write lists rotted quietly and badly: the read list
named `mt show` and `mt export`, neither of which has ever existed as a
command, while omitting `find`, `graph`, `views`, `worktrees` and `eval`. The
write list carried `done`, a legend-derived verb rather than a subcommand, and
missed `import`, `review`, `install` and `preset save`. Nine wrong entries in
two lines, in the file every agent pulls into context.

The lists are correct and `crates/mindtape-cli/tests/read_no_write_e2e.rs` covers
the read arm per command. A new command joins neither list and no test notices.
The lists can rot again.

== Scope

Make the classification a declared thing with a gate on it, in the shape this
repo already uses for `check-doc-coverage`:

- Two `const` arrays in `read_no_write_e2e.rs` (or a small shared module), with
  the per-command read tests generated from the read array rather than written
  out.
- An assertion that the union of the two arrays equals `mt _cli --json`'s
  visible command list. A new command then fails the suite until someone
  classifies it. `go` needs a home: a third array, or an explicit exemption with a
  reason.
- CONTRIBUTING.md's two bullets rendered from that classification rather than
  retyped, once it has a machine-readable form.

== Why not just fix the lists

They were just fixed. That is the second time prose has been the only thing
holding this true. The audit that caught it was a human asking an agent to go
looking, not a mechanism. Everything else drift-prone in these docs reads its
fact from `www/generated/`. This one has no source because nobody has written
the classification down anywhere a machine can see it.
