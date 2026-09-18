#import "../utils/page.typ": page
#import "../utils/site.typ": REPO, u

#let meta = (
  route: "/comparison/",
  label: "Comparison",
  title: "Comparison",
  summary: "How MindTape compares, feature by feature, to the other local file-and-CLI task trackers, and when one of them fits better.",
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

// MindTape's own version reads the real binary's report of itself — "mt X.Y.Z"
// captured by www/generate.sh into www/generated/version.txt, same pattern as
// every other page's `read("../generated/…")` — so it can never drift from what
// actually got tagged and shipped.
#let mt-version = read("../generated/version.txt").trim().split(" ").last()

// Neutral capability heatmap: the SAME rubric applies to every column,
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
  no daemon]. The neighbours below are the other local file-and-CLI trackers.
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

All of them are daemon-free and run locally, so that alone separates none of
them. The table below digs into how each one stores, types, links, and renders a
task.

= Feature by feature

Cells are terse on purpose; the prose after the table carries the nuance. The
matrix scrolls sideways on narrow screens.

#html.elem("div", attrs: (class: "matrix"), table(
  columns: 7,
  align: left,
  table.header(
    [Dimension],
    [#link(u("/"))[MindTape]],
    [#link(dstask-url)[dstask]],
    [#link(taskwarrior-url)[Taskwarrior]],
    [#link(todotxt-url)[todo.txt]],
    [#link(orgmode-url)[Org-mode]],
    [#link(git-bug-url)[git-bug]],
  ),

  [#link(u("/concepts/"))[Storage granularity]],
  [One file / task],
  [One file / task],
  [Central store],
  [One shared file],
  [Many per `.org` file],
  [Git objects],

  [#link(u("/dsl/"))[On-disk format]],
  [Typst `.typ`],
  [YAML `.yml`],
  [SQLite DB],
  [Plain-text line],
  [Outline text],
  [Git blobs on refs],

  [#link(u("/concepts/"))[In the working tree, reviewed with your code]],
  [#cap("green")[Yes, by design]],
  [#cap("grey")[No — its own git repo]],
  [#cap("grey")[No — `~/.task` store]],
  [#cap("amber")[Possible]],
  [#cap("green")[Yes]],
  [#cap("grey")[In git, not the tree]],

  [#link(u("/config/"))[Typed schema + config validation]],
  [#cap("green")[Config-enforced, `mt check`]],
  [#cap("amber")[Light, fixed]],
  [#cap("amber")[Typed attrs + UDAs]],
  [#cap("grey")[None]],
  [#cap("amber")[Soft]],
  [#cap("amber")[Minimal]],

  [#link(u("/config/"))[Status model]],
  [Config legend (enum + dates + transitions)],
  [Fixed set],
  [Fixed enum],
  [`x ` prefix],
  [User-defined keywords],
  [open / closed],

  [#link(u("/priority/"))[Priority model]],
  [Int scale or scored framework],
  [`P0`–`P3`],
  [H/M/L + urgency score],
  [`(A)`–`(Z)`],
  [`#A`/`#B`/`#C`],
  [None],

  [#link(u("/concepts/"))[Task identity]],
  [Filename stem],
  [UUID],
  [UUID],
  [Line position],
  [Headline / `:ID:`],
  [Content hash],

  [#link(u("/filters/"))[Dependencies / blocking]],
  [#cap("green")[Yes — typed links]],
  [#cap("green")[Yes]],
  [#cap("green")[Yes — `depends`]],
  [#cap("grey")[No]],
  [#cap("green")[Yes — `ORDERED` / org-depend]],
  [#cap("grey")[No]],

  [#link(u("/filters/"))[Query + machine output]],
  [#cap("green")[Filter DSL; JSON, CSV]],
  [#cap("amber")[CLI filters; text only]],
  [#cap("green")[Filter DSL; JSON]],
  [#cap("grey")[grep-style; text only]],
  [#cap("green")[agenda / org-ql; many exports]],
  [#cap("green")[filters; JSON]],

  [#link(u("/templates/"))[Templates / scaffolds]],
  [#cap("green")[Slot templates]],
  [#cap("amber")[Copy-a-task template]],
  [#cap("amber")[Recurrence, UDAs, reports]],
  [#cap("amber")[Addons only]],
  [#cap("green")[`org-capture` templates]],
  [#cap("grey")[No]],

  [#link(u("/dsl/"))[Recurring tasks]],
  [#cap("green")[Yes — `mt recur`]],
  [#cap("grey")[No]],
  [#cap("green")[Yes]],
  [#cap("amber")[Addons only]],
  [#cap("green")[Yes]],
  [#cap("grey")[No]],

  [#link(REPO + "/blob/main/lib/prelude.typ")[Typeset-document face]],
  [#cap("green")[Yes — `task` show rule]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("grey")[No]],
  [#cap("green")[Yes — PDF today]],
  [#cap("grey")[No]],

  // NOT "mt binary + Typst": `mt` links the Typst compiler crates and embeds
  // the prelude, so nothing external is installed or shelled out to (verified —
  // `mt ls` evaluates a project with an empty environment and no `typst` on
  // PATH). The one-time `mt install` writes the prelude into the user data
  // directory; that is the whole footprint.
  [#link(u("/quickstart/"))[Toolchain footprint]],
  [Single binary + `mt install`],
  [Single Go binary + git],
  [Single binary],
  [Bash script only],
  [Emacs],
  [Go binary + git],

  // This row used to carry each neighbour's GitHub star count under a
  // "mid-2026" stamp. Nothing in this repo can check those numbers, and they
  // move every week — a stamped snapshot only dates the page, and the maturity
  // gap is already argued in words below, which is where a judgement belongs.
  // The license is the part of the old cell that holds still.
  [#link(REPO + "/blob/main/LICENSE")[License]],
  [MIT],
  [MIT],
  [MIT],
  [GPL-3],
  [GPL-3],
  [GPL-3],
))

= Where MindTape leads

- #strong[Tasks in the code tree.] A task is a file in your working tree,
  reviewed and merged in the same PR as the code it describes. Only Org-mode also
  lives in the tree comfortably — and it is a general editor format, not a
  per-project tracker with a schema.
- #strong[Typed and validated.] One #link(u("/config/"))[config] enforces the
  status legend, bounded priority and difficulty scales, the `kind` enum, an
  estimate unit allow-list, and link resolution — all checked by `mt check` in
  CI. Only Taskwarrior's UDAs come close, and without the validation gate.
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
- #strong[Rich document export.] Org-mode exports whole outlines to polished PDF,
  HTML, and LaTeX with decades of theming. MindTape's `task` show rule typesets a
  single task as an index card today — real, but one card, not a multi-task report
  or a multi-backend document pipeline.
