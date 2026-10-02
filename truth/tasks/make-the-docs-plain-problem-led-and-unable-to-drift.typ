#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Make the docs plain, problem-led and unable to drift",
  priority: ice(
    impact: 9,
    confidence: 0.7,
    ease: 3,
  ),
  difficulty: 3,
  tags: ("docs", "www"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: helps[the first screen shows a typed task that `mt` enforces],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[code blocks and transcripts can only come from checked fixtures and real runs],
    ),
  ),
  kind: "milestone",
  assignee: "agent",
  status: proposed(2026, 10, 2),
)

== Problem

+ Run the README Usage block in a fresh `mt init` project.
+ 🔴 `mt add --set title="Wire OAuth" --set priority=7` prints `unknown --set/-f key(s): priority; known slots: title`, and `mt flip done wire-oauth` prints `illegal transition proposed → done`.
+ 🔴 README opens on 40 lines of YAML that MindTape never reads, and the site's examples are a piano practice backlog.
+ A review of all doc sources found about 80 defects: examples that fail when run, claims the code contradicts, a build machine's home path in published output, history narration and figures of speech.


== TO-BE

- 🟢 Every code block and transcript comes from a checked example or a captured run, and gates refuse a hand-typed block, a stale name, a machine path and a render trap.
- 🟢 README and the home page open on a software team's task and the problems MindTape solves for them; each problem has its own page.
- 🟢 Docs prose is short and direct, and a check refuses history narration and ALL-CAPS emphasis.


== Changes

- `truth/tasks/*.typ`: the child tasks, linked with `parent` to this one. This milestone changes no file itself.


== Out of scope

- New `mt` features, except the verb fix the README task needs.


== Dogfood

This repo's own gate runs every new docs check, and its README is the page the work rewrites.

