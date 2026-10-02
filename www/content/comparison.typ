#import "../utils/page.typ": page
#import "../utils/site.typ": REPO, u

#let meta = (
  route: "/comparison/",
  label: "Comparison",
  title: "Comparison",
  summary: "MindTape compared feature by feature with other local CLI task trackers, and when one of them fits better.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Canonical landing page / repo for each neighbour, reused in the bullet list
// and the matrix header. Verified live (curl -sI) before use.
#let dstask-url = "https://github.com/naggie/dstask"
#let taskwarrior-url = "https://taskwarrior.org"
#let todotxt-url = "http://todotxt.org"
#let orgmode-url = "https://orgmode.org"
#let git-bug-url = "https://github.com/git-bug/git-bug"
#let beads-url = "https://github.com/gastownhall/beads"
#let backlog-url = "https://github.com/MrLesk/Backlog.md"

// MindTape's own version reads the real binary's report of itself — "mt X.Y.Z"
// captured by www/generate.sh into www/generated/version.txt, same pattern as
// every other page's `read("../generated/…")` — so it can never drift from what
// actually got tagged and shipped.
#let mt-version = read("../generated/version.txt").trim().split(" ").last()

// Neutral capability heatmap: one rubric applies to every column,
// including MindTape — green = has the capability / strong, amber = partial
// or indirect, grey = none. Only the objectively-scoreable capability rows in
// the matrix below get wrapped in `cap`; descriptive rows (storage
// granularity, on-disk format, identity, ...) stay plain, since there is no
// better or worse to signal there. Colour lives in theme-aware CSS custom
// properties (components.css), not a baked Typst `fill`, so the tone stays
// legible in both themes.
#let cap(level, body) = html.elem(
  "span",
  attrs: (class: "cap cap-" + level),
  [#body],
)

MindTape is not out to replace every task tracker. It sits in a narrow niche:
#strong[typed tasks, one per file, versioned with the code, validated by config,
  no daemon]. The neighbours below are the other local, CLI-first trackers.
Hosted services (GitHub Issues, Jira, Linear) are a different category:
daemon-plus-database SaaS with no in-repo file artifact, and out of scope here.

= The neighbourhood

- #strong[#link(u("/"))[MindTape]] — one typed Typst file per task, config-validated, in your repo.
- #strong[#link(dstask-url)[dstask]] — the closest cousin: one YAML file per task, synced through
  git from a single Go binary, auto-committed in its own repo rather than the code tree, with a
  light fixed schema and no setup.
- #strong[#link(taskwarrior-url)[Taskwarrior]] — the incumbent CLI engine for a central out-of-repo
  store: urgency scoring, recurrence, user-defined attributes, a deep filter language.
- #strong[#link(todotxt-url)[todo.txt]] — the minimalist standard: one portable plain-text file, no
  schema, supported by apps on every platform.
- #strong[#link(orgmode-url)[Org-mode]] — Emacs outline files that double as a live agenda and a
  typeset-exportable document.
- #strong[#link(git-bug-url)[git-bug]] — issues stored inside git objects, offline-first and
  distributed, bridging to GitHub or GitLab.
- #strong[#link(beads-url)[Beads]] — a coordination database for fleets of coding agents: Dolt
  SQL synced over its own git refs, atomic claims with leases, workflow molecules, MCP.
- #strong[#link(backlog-url)[Backlog.md]] — one Markdown file per task in the repo, built for a
  human steering agents: acceptance-criteria checklists, a terminal and web Kanban, MCP.

All of them run locally, and none needs a server to work: Beads can run one
(`bd serve`, `dolt sql-server`) but embeds its database by default. The table
below digs into how each one stores, types, links, and renders a task.

= Feature by feature

Cells are terse on purpose; the prose after the table carries the nuance. The
matrix scrolls sideways on narrow screens.

#html.elem("div", attrs: (class: "matrix"), table(
  columns: 9,
  align: left,
  table.header(
    [Dimension],
    [#link(u("/"))[MindTape]],
    [#link(dstask-url)[dstask]],
    [#link(taskwarrior-url)[Taskwarrior]],
    [#link(todotxt-url)[todo.txt]],
    [#link(orgmode-url)[Org-mode]],
    [#link(git-bug-url)[git-bug]],
    [#link(beads-url)[Beads]],
    [#link(backlog-url)[Backlog.md]],
  ),

  [#link(u("/concepts/"))[Storage granularity]],
  [One file / task],
  [One file / task],
  [Central store],
  [One shared file],
  [Many per `.org` file],
  [Git objects],
  [Dolt database],
  [One file / task],

  [#link(u("/dsl/"))[On-disk format]],
  [Typst `.typ`],
  [YAML `.yml`],
  [SQLite DB],
  [Plain-text line],
  [Outline text],
  [Git blobs on refs],
  [Dolt SQL tables],
  [Markdown + YAML `.md`],

  [#link(u("/concepts/"))[In the working tree, reviewed with your code]],
  [#cap("green")[Yes, by design]],
  [#cap("grey")[No — its own git repo]],
  [#cap("grey")[No — `~/.task` store]],
  [#cap("amber")[Possible]],
  [#cap("green")[Yes]],
  [#cap("grey")[In git, not the tree]],
  [#cap("grey")[No — its own Dolt refs]],
  [#cap("green")[Yes]],

  [#link(u("/config/"))[Typed schema + config validation]],
  [#cap("green")[Config-enforced, `mt check`]],
  [#cap("amber")[Light, fixed]],
  [#cap("amber")[Typed attrs + UDAs]],
  [#cap("grey")[None]],
  [#cap("amber")[Soft]],
  [#cap("amber")[Minimal]],
  [#cap("amber")[Fixed, checked on write]],
  [#cap("amber")[Checked on CLI writes only]],

  [#link(u("/config/"))[Status model]],
  [Config legend (enum + dates + transitions)],
  [Fixed set],
  [Fixed enum],
  [`x ` prefix],
  [User-defined keywords],
  [open / closed],
  [7 built-ins + custom categories],
  [Config list],

  [#link(u("/priority/"))[Priority model]],
  [Int scale or scored framework],
  [`P0`–`P3`],
  [H/M/L + urgency score],
  [`(A)`–`(Z)`],
  [`#A`/`#B`/`#C`],
  [None],
  [`P0`–`P4`],
  [Configured labels],

  [#link(u("/concepts/"))[Task identity]],
  [Filename stem],
  [UUID],
  [UUID],
  [Line position],
  [Headline / `:ID:`],
  [Content hash],
  [Hash ID (`bd-a1b2`)],
  [Sequential `TASK-12`],

  [#link(u("/filters/"))[Dependencies / blocking]],
  [#cap("green")[Yes — typed links]],
  [#cap("green")[Yes]],
  [#cap("green")[Yes — `depends`]],
  [#cap("grey")[No]],
  [#cap("green")[Yes — `ORDERED` / org-depend]],
  [#cap("grey")[No]],
  [#cap("green")[Yes — typed kinds]],
  [#cap("green")[Yes]],

  [#link(u("/links/"))[Hierarchy]],
  [#cap("amber")[`parent` links + folder tags]],
  [#cap("amber")[Projects]],
  [#cap("amber")[Dotted projects]],
  [#cap("amber")[`+project` tags]],
  [#cap("green")[Nested headlines]],
  [#cap("grey")[Labels only]],
  [#cap("green")[Epics, child IDs, molecules]],
  [#cap("green")[Subtasks + milestones]],

  [#link(u("/filters/"))[Query + machine output]],
  [#cap("green")[Filter DSL; JSON, CSV]],
  [#cap("amber")[CLI filters; text only]],
  [#cap("green")[Filter DSL; JSON]],
  [#cap("grey")[grep-style; text only]],
  [#cap("green")[agenda / org-ql; many exports]],
  [#cap("green")[filters; JSON]],
  [#cap("green")[`bd query`; JSON]],
  [#cap("amber")[Flags + fuzzy search; JSON]],

  [#link(u("/templates/"))[Templates / scaffolds]],
  [#cap("green")[Slot templates]],
  [#cap("amber")[Copy-a-task template]],
  [#cap("amber")[Recurrence, UDAs, reports]],
  [#cap("amber")[Addons only]],
  [#cap("green")[`org-capture` templates]],
  [#cap("grey")[No]],
  [#cap("green")[Formulas → molecules]],
  [#cap("amber")[Definition-of-Done defaults]],

  [#link(u("/dsl/"))[Recurring tasks]],
  [#cap("green")[Yes — `mt recur`]],
  [#cap("grey")[No]],
  [#cap("green")[Yes]],
  [#cap("amber")[Addons only]],
  [#cap("green")[Yes]],
  [#cap("grey")[No]],
  [#cap("amber")[Patrol molecules, no calendar]],
  [#cap("grey")[No]],

  [#link(u("/hooks/"))[Hooks / automation]],
  [#cap("green")[`[[on.flip]]` gates]],
  [#cap("grey")[No]],
  [#cap("green")[Hook scripts]],
  [#cap("amber")[Addon actions]],
  [#cap("green")[Elisp hooks]],
  [#cap("grey")[No]],
  [#cap("green")[Git hooks, gates]],
  [#cap("amber")[Status-change command]],

  [#link(u("/agents/"))[Agent integration]],
  [#cap("green")[Agent skill; JSON everywhere]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("amber")[GraphQL API]],
  [#cap("green")[MCP, `bd prime`, setup recipes]],
  [#cap("green")[MCP, agent instructions]],

  [#link(u("/config/"))[Multi-agent coordination]],
  [#cap("amber")[Assignee + WIP limits]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("green")[Atomic claim + leases]],
  [#cap("amber")[Assignee + file locks]],

  [#link(u("/cli/"))[Human UI]],
  [#cap("amber")[Table, `mt ls -i`]],
  [#cap("amber")[CLI table]],
  [#cap("amber")[CLI reports]],
  [#cap("green")[Apps on every platform]],
  [#cap("green")[Emacs agenda]],
  [#cap("green")[TUI + web UI]],
  [#cap("amber")[CLI; community UIs]],
  [#cap("green")[TUI + web Kanban]],

  [#link(REPO + "/blob/main/lib/prelude.typ")[Typeset-document face]],
  [#cap("green")[Yes — `task` show rule]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("green")[Yes — PDF today]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],

  // A single binary with no separate Typst: `mt` links the Typst compiler
  // crates and embeds the prelude, so nothing external is installed or shelled
  // out to (verified: `mt ls` evaluates a project with an empty environment
  // and no `typst` on `PATH`). The one-time `mt install` writes the prelude
  // into the user data directory; that is the whole footprint.
  [#link(u("/quickstart/"))[Toolchain footprint]],
  [Single binary + `mt install`],
  [Single Go binary + git],
  [Single binary],
  [Bash script only],
  [Emacs],
  [Go binary + git],
  [Go binary; CGO + ICU for embedded Dolt],
  [Bun binary (npm, brew)],

  // The table carries no GitHub star counts. Nothing in this repo can check
  // them, they change every week, and the maturity gap is argued in words
  // below. The license is stable, so it gets a row.
  [#link(REPO + "/blob/main/LICENSE")[License]],
  [MIT],
  [MIT],
  [MIT],
  [GPL-3],
  [GPL-3],
  [GPL-3],
  [MIT],
  [MIT],
))

= Where MindTape leads

- #strong[Tasks in the code tree.] A task is a file in your working tree,
  reviewed and merged in the same PR as the code it describes. Org-mode and
  Backlog.md also live in the tree, but neither checks what lands there: Org is a
  general editor format, and Backlog.md accepts a hand-edited status or a
  dangling dependency without complaint.
- #strong[Typed and validated.] One #link(u("/config/"))[config] enforces the
  status legend, bounded priority and difficulty scales, the `kind` enum, an
  estimate unit allow-list, and link resolution — all checked by `mt check` in
  CI and before every write, hand edits included. Taskwarrior's UDAs and Beads'
  checked writes come closest, and neither is configured per project.
- #strong[Scored priority frameworks.] ICE, RICE, WSJF, Eisenhower, and MoSCoW
  turn typed inputs into a priority — scored onto the scale or classified into
  named categories (#link(u("/priority/"))[details]). Nothing else in this table
  computes priority from structured inputs.
- #strong[Composable source.] The file is real Typst: `#import` a shared library,
  compute values, reuse templates. A todo.txt line or a YAML record cannot.

= Where the others win

- #strong[Maturity and ecosystem.] Every competitor is released and
  battle-tested. MindTape is at #mt-version, with far less mileage and a much
  smaller ecosystem; Taskwarrior and todo.txt carry decades of tooling,
  integrations, and mobile apps.
- #strong[Zero install.] todo.txt is one text file plus a shell script — no
  binary at all. MindTape needs the `mt` binary, plus one `mt install` per
  machine to place the prelude.
- #strong[Multi-agent coordination.] Beads lets an agent claim a task
  atomically, with a lease that expires when the agent dies, so a fleet
  picking from one queue never doubles up. MindTape has an `assignee` and WIP
  limits, but no claim.
- #strong[Agent and human front ends.] Beads and Backlog.md ship MCP servers,
  and Backlog.md adds a terminal and web Kanban board. MindTape offers a CLI with
  JSON output and an #link(u("/agents/"))[agent skill], nothing more.
- #strong[Rich document export.] Org-mode exports whole outlines to polished PDF,
  HTML, and LaTeX with decades of theming. MindTape's `task` show rule typesets a
  single task as an index card today — real, but one card, not a multi-task report
  or a multi-backend document pipeline.
