#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Configurable task stem naming",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 5,
  ),
  tags: (),
  status: done(2026, 7, 10),
)

== Summary

Let a project configure how `mt add` mints a new task's filename stem, from a
template with macros. Default stays bare `{slug}` — today's behaviour, no
rename of anything. This is a general naming feature, independent of recurrence;
`mt recur` (spawn) reuses the same minting machinery for occurrence files.

Reference point: speconaut names artifacts by a filename sub-extension —
`<slug>.task-<yymmdd>.typ` for dated kinds, `<slug>.<kind>.typ` for date-less
ones, plus specials (`main`, `vision`) — in its own `src/artifact.rs`. The
convergence directive wants MindTape to host that generic capability, so the
template must be expressive enough to produce it, and speconaut can then delete
its classifier and depend on this.

== Decisions

- Config `[tasks.naming] stem = "{slug}"`. Unset ⇒ bare `{slug}` (current
  behaviour). Set ⇒ `mt add` and `mt recur` both apply it.
- Macros: `{slug}`, `{date}` (ISO), `{date:FMT}` (a Typst `datetime.display`
  format string), `{yy}` / `{mm}` / `{dd}`, `{n}` / `{n:0N}` (occurrence index,
  zero-padded). A `{kind}` macro is reserved for the convergence extension.
- Identity is still the stem. The template shapes minting only; link resolution
  is unchanged.
- Never rename an existing file. An authored task keeps its stem forever (git
  archaeology, commit-message references). The template applies at creation.
- `mt recur` spawn reuses the template. Its `{date}` macros resolve to the
  occurrence's anchor date (not today); `{n}` to the occurrence index. If naming
  is unset, recur falls back to a built-in `{slug}-{date}` so spawned stems stay
  unique while the original authored file stays bare.
- Collision guard: if a rendered stem already exists, append an incrementing
  `-{n}`; never overwrite.

== Constraints

- Pure, testable macro expander (date formats, counter, padding); no clock reads
  inside it — the caller passes the reference date.
- One shared minting helper, called by both the `add` write path and the recur
  spawn path, so the two never diverge.
- `mt check` and existing tasks stay green (default template is identity).

== Scope

- Parse `[tasks.naming]`.
- Macro expander + collision disambiguation.
- Wire the shared minter into `mt add` and (later) `mt recur` spawn.
- Tests: default bare, speconaut-style `{slug}.task-{yy}{mm}{dd}`, date/counter
  macros, collision, and reuse from a recur spawn.
- Convergence note: leave room for a `{kind}` macro and multi-kind classification
  so speconaut's `artifact.rs` scheme can converge onto this minter.
