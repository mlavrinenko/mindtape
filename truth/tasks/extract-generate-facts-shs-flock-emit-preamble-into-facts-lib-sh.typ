#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Extract generate-facts.sh's flock/emit preamble into facts-lib.sh",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (files: 2),
  tags: ("found", "www", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a shared preamble stops living in the one file it also grows past, and generate-facts.sh gets room for its next artifact without a re-cap],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 22),
)

== Problem

+ filing the first real finding while adding the finding-filing channel, checking linecop headroom across the repo
+ 🔴 #raw("www/generate-facts.sh"): sits at 298 of its 300-line linecop cap, so the next fact has nowhere to go

`linecop --baseline 85`:

```
~~~ ./www/generate-facts.sh: 298 lines (limit: 300, 99% of limit)
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/facts-lib.sh").

== TO-BE

- 🟢 `just check-file-size` passes and `linecop --baseline 90` no longer names `www/generate-facts.sh`.
- 🟢 The flock re-exec guard, `emit()` and `emit_raw()` live in one sourced file instead of being copied into every artifact script that wants the same atomic-write discipline.

== Changes

- `www/facts-lib.sh` — new: the flock re-exec guard (`MINDTAPE_GENERATED_LOCK`), `emit()` and `emit_raw()`, moved here verbatim from `www/generate-facts.sh`.
- `www/generate-facts.sh` — `source "$SCRIPT_DIR/facts-lib.sh"` in place of the preamble it used to carry; every `emit`/`emit_raw` call site is unchanged.

== Rollback

Delete `www/facts-lib.sh` and restore the preamble it replaced in `www/generate-facts.sh` (still in this commit's git history).

== Out of scope

- Deduping the *other* three scripts that already carry their own near-identical `emit()` copy (`www/generate.sh`, `www/generate-site-pages.sh`, `www/generate-examples.sh`, each says so in its own comment). That is a real second finding, not this one: it changes call sites this task does not touch.

== Dogfood

This is the template's own first real use: filed with `mt add --template finding` to prove the channel this task added actually works, rather than left in a closing report.
