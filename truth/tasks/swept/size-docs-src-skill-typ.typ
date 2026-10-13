#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "rewrite docs/src/skill.typ under its cap",
  kind: "refactor",
  priority: ice(
    impact: 10,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  assignee: "agent",
  tags: ("swept", "size"),
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
      sound-codebase: helps[a file under its cap changes without reading its
        neighbours],
      docs-match-code: neutral,
    ),
  ),
  status: proposed(2026, 10, 13),
)

== Problem

+ Run `linecop --format json`.
+ 🔴 #raw("docs/src/skill.typ") stands at 144 of its 144-line budget: 0 lines of headroom left, 100% of the cap spent.
+ 🔴 #raw("skills/mindtape/SKILL.md") stands at 67 of its 76-line budget: 9 lines of headroom left, 88.2% of the cap spent.

Prose: the seam is a section boundary, and the question is which sections a reader needs in front of them every time versus which ones they go looking for.

`just ask-refactoring` filed this, and the sweep re-reads the signal every run,
so the task is worth only as much as that measurement still is.

== TO-BE

- 🟢 `just check-file-size` passes and `linecop --baseline 90` no longer names
  this file.

== Changes

- Follow "Rewrite the whole file" in `docs/contributing/documentation.md`:
  rewrite the source whole for its reader, or move an argument where a reader
  reaches it deliberately, such as a deep dive or a skill's `references/`.
  Trimming words to fit is not a fix.
- Re-cap only a file no session loads whole. The `.linecop.yaml` override of an
  always-loaded file says it is rewritten, never raised.
- If you re-cap, write the reason next to the pattern in `.linecop.yaml`, in the
  style its `SKILL.md` / `AGENTS.md` / `CONTRIBUTING.md` entries already use. A
  written reason is the only thing that suppresses this finding on the next
  sweep; there is no ignore list anywhere else, deliberately.

== Out of scope

- Behaviour changes: a split moves code and changes nothing it does.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
