#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Require an origin link on every finding",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "tooling", "gating", "stateless"),
  estimate: (src-files: 3),
  x-goals: (
    product: verdicts(
      (
        implementable-tasks: helps[a finding carries the task that surfaced it, so the next reader follows the graph without rereading a report],
        enforced-practice: helps[the template and mt check both refuse a finding with no origin],
      ),
      ..(
        "readable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (sound-codebase: helps[a finding cannot be filed untraceable]),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 4),
  links: depends-on("accept-links-slot-values-in-mt-add.typ"),
)

== AS-IS

+ Run `mt add --describe -t finding` and read the slots. The template has no `links` slot, so `mt add` cannot ask for the task that surfaced the finding.
+ 🔴 The AS-IS body says the finding "fell outside the task that surfaced it" without naming that task, and no rule requires a `surfaced-by` edge.
+ `mt ls 'has_tag("found")'` lists a finding whose origin is only in the closed session that filed it.

```
$ mt add --describe -t finding | grep links
$
```

== TO-BE

- 🟢 `mt add -t finding` requires a `links` slot with one `surfaced-by` edge, and refuses a value that holds none.
- 🟢 `surfaced-by` is a project kind with `max = 1`, non-blocking, so the origin can close while the finding stays open.
- 🟢 A rule refuses an open `found` task with no `surfaced-by` edge, and the finding template's filled example passes it.
- 🟢 The project prelude exports `surfaced-by(...)`, so a hand-written task writes `links: surfaced-by("origin.typ")`.
- 🟢 The AS-IS sentence is gone; the edge is the one place naming the origin.

```
# .mindtape/config.toml
[[tasks.links.kinds]]
name = "surfaced-by"
max = 1
blocking = false
acyclic = true

[[tasks.rules]]
name = "finding-has-surfaced-by"
when = '!terminal && has_tag("found")'
assert = 'links("surfaced-by")'
```

== Changes

- `.mindtape/config.toml`: the `surfaced-by` kind and the `finding-has-surfaced-by` rule.
- `.mindtape/templates/finding.typ`: the required `links` slot, and drop the AS-IS sentence.
- `truth/lib.typ`: import `edge` and export `surfaced-by(..args)`.
- `docs/src/contributing.typ`: say to name the origin in the template's `links` slot, then regenerate `CONTRIBUTING.md` with `just docs md`.
- `truth/tasks/list-a-flip-gate-with-no-message-in-mt-rules-and-mt-tags.typ`: backfill its `surfaced-by` edge to `read-has-tags-and-spaced-has-tag-calls-in-mt-tags`.

== Alternatives

- `parent`: a finding is out of scope and may never close, so a parent edge would keep the origin from closing.
- `related` with a note: it cannot tell the origin from any other related edge, so no rule can require exactly one.
- A required `surfaced-by` string slot plus `mt link`: two commands, and the `links` slot kind carries the origin and extras in one.

== Out of scope

- Backfilling the 191 closed findings. The rule's `!terminal` leaves history alone.
- A pack that ships the kind and template. The prelude task decides the pack/prelude seam; this repo keeps the kind local.

== Dogfood

The next out-of-scope finding is filed with `mt add -t finding -f -`, and `mt ls 'links("surfaced-by")'` walks from any finding to the work that exposed it.
