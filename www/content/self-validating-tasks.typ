#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/self-validating-tasks/",
  label: "Self-validating tasks",
  title: "Self-validating tasks",
  summary: "A repo's tasks can carry a structure it invented, checked by the repo's own Typst function on every write — no separate linter to maintain.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A MindTape task is a Typst file, so the check can be a Typst function the
project already ships, called on every `mt check` and before every write. This
page proves it live: a task that skips a rule the project invented for itself,
refused on the spot.

= A structure the repo invented

This repo (MindTape's own) scores every open task against goals it sets for
itself, in `truth/goals.typ`, and every task's `x-goals` field says how. The
fixture below (`examples/self-validating/`) shrinks that idea to two goals:

#raw(read("../generated/self-validating-goals.typ"), lang: "typ", block: true)

= The function that checks it

`lib.typ` is the check: every goal above needs a verdict in the task's
`x-goals`, or the function names the one that is missing.

#raw(read("../generated/self-validating-lib.typ"), lang: "typ", block: true)

= The rule that wires it in

`[[tasks.rules]]` names the function and the field it reads — this is the
project's whole config, `[[typst.local-packages]]` and status legend
included. What `check`'s three keys mean, and what a project can do with them
beyond this, is at #link(u("/config/"))[Configuration, "Checking a field with
  a Typst function"]:

#raw(
  read("../generated/self-validating-config.toml"),
  lang: "toml",
  block: true,
)

= Two tasks, one refusal

One task addresses both goals:

#raw(
  read("../generated/self-validating-task-pass.typ"),
  lang: "typ",
  block: true,
)

One does not — it never says how it serves `no-lock-in`:

#raw(
  read("../generated/self-validating-task-fail.typ"),
  lang: "typ",
  block: true,
)

`mt check` catches it:

#terminal(read("../generated/self-validating-check.txt"))

It exits
#raw(read("../generated/self-validating-check-exit.txt").trim())
— a refusal, the same one a real `mt add` or `mt flip` would give while
writing the file, never a crash that panics the whole check.
