#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Keep path and dir rules out of the template check",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt", "tests", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a template check reports only a breach the template can fix],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
  links: depends-on("unify-mt-add-and-mt-import-destinations-under-dir.typ"),
)

== AS-IS

+ Add a rule `when = 'kind == "fix"' assert = 'matches(path, "^tasks/fixes/")'` and a fix template whose example is a fix.
+ 🔴 `mt check` fills the template with its example, checks it at the first root, and reports the path rule on the template, though the template is not a task file.
+ The same false positive is waiting for the `dir` field.


== TO-BE

- 🟢 The template pass drops rules that read `path` or `dir`, as it already drops rules that read the whole set.
- 🟢 A rule about the template's own shape still reports, so a stale template is not hidden.
- 🟢 `path` and `dir` share one `path_dependent_fields` list, so the add whitelist and the template pass cannot drift.

```
$ mt check
ok: 968 tasks
```


== Changes

- `crates/mindtape/src/filter/mod.rs`: expose `reads_path` and the shared `path_dependent_fields` names.
- `crates/mindtape-cli/src/cmd/check/templates.rs`: drop path-dependent breaches in the template pass.
- `crates/mindtape-cli/tests/it/check_templates_e2e.rs`: a path rule absent from the template report, and a shape rule still present.
- `www/content/rules.typ`: document what the template pass ignores.


== Alternatives

- Drop every rule from the template pass: hides a template that no longer has a valid shape.
- Add a template-only flag: config surface for a check with one answer.
- Check a template at a synthetic real path: it is still not a task, and its derived tags would lie.


== Out of scope

- The add pre-write check; this is the template pass only.
- The `dir` field itself; this task reads the name, and the add-hook task adds it.
- Folder-derived tags from the template's example path.


== Dogfood

This repo's fix and feat templates stop reporting the directory rule once the backlog gains directories.

