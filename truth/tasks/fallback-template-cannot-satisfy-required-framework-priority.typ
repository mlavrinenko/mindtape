#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "The fallback task template cannot satisfy a config that requires a framework priority",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 2),
  status: done(
    2026,
    8,
    1,
  )[the strict-framework splice writes framework("name", input: slot(…), …); mt init no longer refuses the answer set],
)

== Summary

In a project with no \`.mindtape/templates/task.typ\`, \`mt add\` falls back to a built-in template whose slots are title, difficulty and estimate. If the project's config requires a priority — and especially if \`\[tasks.priority\]\` declares \`framework = "ice"\` with \`allow-manual = false\` — no invocation of \`mt add\` can produce a valid task, and there is no error that says so.

== Reproduction

Observed 2026-07-31 in \`~/projects/home/speconaut\`, whose config sets
\`framework = "ice"\`, \`allow-manual = false\`, \`required = "!terminal"\` on
\`\[tasks.priority\]\`, and which has no \`.mindtape/templates/\` directory.

Naming the field fails, because it is not a slot:

```
mt add restore the workspace build --set 'priority=ice(impact: 9, confidence: 0.9, ease: 6)'
Error: unknown --set/-f key(s): priority; known slots: title, difficulty, estimate
```

Omitting it fails too, after the file is already written:

```
mt add restore the workspace build --set difficulty=5 --set 'estimate=(hours: 6)'
wrote .../truth/tasks/restore-the-workspace-build.typ
warning: restore-the-workspace-build: priority: required field is missing
Error: ... was kept on disk exactly as written; fix or delete it before the next command
```

The framework leaves are the slots a template should expose here —
\`--set impact=9 --set confidence=0.9 --set ease=6\` — exactly as this project's
own \`.mindtape/templates/task.typ\` does. The fallback does not.

== Scope

- \`mt add\` slot resolution and the built-in fallback template
- either the fallback honours the config's required fields (emitting the
  framework's leaf slots when one is declared), or \`mt add\` refuses up front
  and names the missing template, rather than writing a file it knows is invalid
- out of scope: the deliberate keep-the-invalid-file-on-disk behaviour, which is sound
