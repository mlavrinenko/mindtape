#import "@local/mindtape:0.2.0": *

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

Let a project refuse to commit a task file that still carries source comments, so review annotations must be resolved into the task rather than accumulating inside it.

The workflow this serves: an agent files tasks, a human reads them and leaves `// no, use in_view() instead` inline, the agent reframes the task and removes the annotation. The comment is a conversation, not content. Without a gate the conversation silently becomes the artifact — the note survives into git, the reframe never happens, and the next reader cannot tell a live objection from a settled one.

Detection must be on the parse plane, not a regex. A `//` inside a string literal or a raw block is not a comment, and a regex cannot tell the difference.

== Scope

- Extend `typst-cst` with `parse_comments(text) -> Vec<Comment>`, a sibling of the existing `parse_headings`, returning kind (line or block), text and span. `typst-cst` already wraps `typst_syntax::SyntaxKind` for headings and is described as package-agnostic parse-plane structure extraction, so comment extraction belongs there rather than in `mindtape` — and it stays reusable for speconaut, per the convergence directive.
- New `[tasks.comments]` config section: `policy` of `"allow"` (default), `"warn"` or `"forbid"`, plus an optional `allow-pattern` regex so a project can permit a deliberate form while forbidding stray notes.
- A violation reports the file, the line and the comment text, so the offending note is visible in the failure rather than requiring a hunt.
- Applies to the whole task file, not only the body — an annotation dropped inside the `task.with(...)` call is the same problem.
- MindTape sets `policy = "forbid"` on its own backlog. The `gated` preset does too; the others leave the default.
- Tests: a line comment and a block comment are both caught; `//` inside a string literal and inside a raw block are not; `allow-pattern` permits a matching comment and still rejects a non-matching one; `"warn"` does not fail the command where `"forbid"` does; a file with no comments passes.
