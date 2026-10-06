#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Put the misspelled-field error on the task file in the editor",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.6,
    ease: 3,
  ),
  difficulty: 4,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: helps[the editor marks the misspelled field where it is written],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7),
)

== Problem

+ Capturing tinymist's diagnostics on examples/lsp/invalid/misspelled-field.typ for the Overview
+ 🔴 #raw("lib/prelude.typ"): tinymist publishes the \`unknown task slot\` error on the prelude's assert and publishes nothing for the task file, so an editor draws no squiggle under the misspelled field.

```
$ typst compile tasks/b.typ   # a file that writes `titel:`
error: assertion failed: unknown task slot `titel`; tool extensions must use an x-<ns> slot
    ┌─ @local/mindtape:0.2.0/prelude.typ:672:4
```
tinymist 0.14.10 sends the same error with the URI of `prelude.typ` and no related information.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("lib/prelude.typ").

== TO-BE

- 🟢 In an editor, a task file that writes `titel:` shows the error under `titel`, in the task file.
- 🟢 `mt check` still names the file and the slot.


== Changes

- `lib/prelude.typ`: replace the slot assert with a call that Typst itself refuses. Build a closure whose named parameters are the call's own `x-` names, `eval("(" + xs.map(k => k + ": none, ").join() + ") => none")`, and spread the sink into it. A spread argument keeps its span, so Typst 0.14.2 prints `unexpected argument: titel` under `titel` in the task file, and `x-` fields stay flat.
- `docs/src/*.typ`, `www/content/*.typ`: wherever a page quotes the `unknown task slot` error, quote the new one.
- `www/content/index.typ`: drop the sentence that says the error sits at the prelude's check, and draw the captured diagnostic under `titel`.


== Rollback

- Revert the `lib/prelude.typ` change.


== Out of scope

- Other assertions in the prelude. Each one that names a value the user wrote gets its own task if it matters.


== Dogfood

Every misspelled field in `truth/tasks/` shows up under the word in tinymist.
