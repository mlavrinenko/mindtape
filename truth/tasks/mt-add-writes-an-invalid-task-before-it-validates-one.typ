#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mt add writes an invalid task before it validates one",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  assignee: "agent",
  tags: ("cli",),
  estimate: (files: 3),
  status: done(
    2026,
    9,
    7,
  )[validates before writing; check\_candidate is the shared seam, other writers tracked separately],
)

== Summary

`mt add` renders the template to disk and validates afterwards, so a rejected
add leaves the broken file behind — and the error says so: "was kept on disk
exactly as written; fix or delete it before the next command". Every subsequent
`mt` invocation then fails on that file until someone removes it by hand, which
means one mistyped slot costs a repair rather than a retry.

Observed 2026-09-05 in the kimmable repo. Four consecutive `mt add` attempts
were needed to file one task: `tags` is not a settable slot in that project's
template, `estimate.files=5` is not a path into one, and `estimate=5` parses as
a *number* where the template's slot wants a dict — `(files: 5)`. Only the
last of those four failed late enough to write anything, and it is the one that
left a file needing manual repair before the board could be read again.

== Scope

- Validate the rendered task *before* it reaches the filesystem, and write
  nothing when validation fails. A render into a temporary buffer, checked, then
  committed to its path — the file either exists and is valid, or was never
  created.
- The failure message then names what is wrong with the *input* and stops.
  Nothing about repairing a file, because no file exists to repair.
- `mt add --print` already renders without writing, so the render and the write
  are separable today; this is about which side of that seam validation sits on.

== Watch for

*A slot's accepted shape is the template's, not mindtape's.* `estimate` is a
text slot defaulting to `(files: 2)` here and a dict-valued one in kimmable's
template, so an error naming a shape has to read it off the template in hand
rather than state a fixed grammar. Should the message quote that template's own
default as a copyable example, rather than describing the grammar in prose?

*Do not fix this by making the validator lenient.* The rejections above were
all correct; the defect is the order of write and check, not the strictness of
the check. Widening what a slot accepts would trade a repairable error for a
malformed board.

One open question, and it is about how far the seam moves: does `mt add` alone
validate before writing, or does every writer — `mt set`, a status flip, any
command that rewrites a `.typ` — go through the same
render-check-then-commit path?

== Resolution

`mt add` alone moves. The capability is public
(`mindtape::check_candidate`, `crates/mindtape/src/candidate.rs`), so the other
writers can adopt the same path without a second implementation, and
#link("extend-the-pre-write-validation-seam-to-the-other-writers.typ")[a
  follow-up task] carries them. They were left out deliberately: `mt recur`'s
spawn is the same defect and a mechanical port, but `mt flip` and `mt review`
rewrite a file that was already valid, where the question is whether a rejected
rewrite is *restored* rather than whether it is written — a different decision,
not this one applied twice.

Two seams keep the old order, both because they cannot have the new one.
`--no-verify` writes whatever it was handed, which is what the flag means. A
destination outside the project root (`mt add --into /elsewhere`) has no
project-relative path to evaluate a candidate at, so it writes first and still
reports the file as kept.

The message question above ("should the error quote the template's own
default") is *not* answered. The pre-write error names the field and the shape
the Rust layer expected — `estimate: expected a dict, found a number` — and
says `nothing was written`; it does not reach back into the template for a
copyable example. That is worth doing and is a separate change to the violation
text, not to the write order.

What did not change: the check is `validate` only, exactly the pass
`verify_written` ran. `mt check`'s other gates — body headers,
`[tasks.body.content]`, `[[tasks.limits]]`, the comment policy — still run only
at `mt check`, so `mt add` can still write a task that `mt check` then rejects
on a body rule. Pulling those into the pre-write check would refuse the default
template's own `- TODO` scope placeholder on every `mt add`, which is a
decision about what `add` is for, not a fix to the order of write and check.
