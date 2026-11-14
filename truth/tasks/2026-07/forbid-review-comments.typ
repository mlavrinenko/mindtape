#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "forbid review comments",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("gating", "substrate"),
  status: done(2026, 7, 22),
)

== Summary

Let a project refuse to commit a task file that still carries source comments, so reviewers must resolve each annotation into the task rather than let it accumulate inside it.

Here is the workflow. An agent files tasks, a human reads them and leaves `// no, use in_view() instead` inline, and the agent reworks the task and removes the annotation. The comment is a conversation, not content. Without a gate the conversation silently becomes the artifact. The note survives into git, the rewrite never happens, and the next reader cannot tell a live objection from a settled one.

Detection must be on the parse plane, not a regex. A `//` inside a string literal or a raw block is not a comment, and a plain pattern match gets it wrong.

== Scope

- Extend `typst-cst` with `parse_comments(text) -> Vec<Comment>`, a sibling of the existing `parse_headings`, returning kind (line or block), text and span. `typst-cst` already wraps `typst_syntax::SyntaxKind` for headings and calls itself package-agnostic parse-plane structure extraction. Comment extraction belongs there rather than in `mindtape`, and it stays reusable for speconaut, per the convergence directive.
- New `[tasks.comments]` config section: `policy` of `"allow"` (default), `"warn"` or `"forbid"`, plus an optional `allow-pattern` regex so a project can permit a deliberate form while forbidding stray notes.
- A violation reports the file, the line and the comment text, so the offending note is visible in the failure rather than requiring a hunt.
- Applies to the whole task file, not only the body. An annotation dropped inside the `task.with(...)` call is the same problem.
- The project sets `policy = "forbid"` on its own backlog. The `gated` preset does too. The others leave the default.
- Tests: a line comment and a block comment both fail. A `//` inside a string literal and inside a raw block does not. `allow-pattern` permits a matching comment and still rejects a non-matching one. `"warn"` does not fail the command where `"forbid"` does. A file with no comments passes.
