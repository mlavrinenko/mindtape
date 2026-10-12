#import "../utils/page.typ": page
#import "../utils/site.typ": REPO, mindtape, u
#import "../utils/ui.typ": el

#let meta = (
  route: "/comparison/",
  label: "Comparison",
  title: "Comparison",
  summary: "Let's see how "
    + mindtape
    + " compares to other local task trackers and which one may fit you better.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Each tool's home page, used in the list and the table header.
#let dstask-url = "https://github.com/naggie/dstask"
#let taskwarrior-url = "https://taskwarrior.org"
#let todotxt-url = "http://todotxt.org"
#let orgmode-url = "https://orgmode.org"
#let git-bug-url = "https://github.com/git-bug/git-bug"
#let beads-url = "https://github.com/gastownhall/beads"
#let backlog-url = "https://github.com/MrLesk/Backlog.md"

// The version `mt --version` printed when the docs were built.
#let mt-version = read("../generated/version.txt").trim().split(" ").last()

// A table cell coloured by how fully a tool has the feature: green for yes,
// amber for partly, grey for no. Rows that describe rather than rate use no
// colour. The colours come from CSS variables in components.css, so they work
// in both themes.
#let cap(level, body) = el(
  "span",
  attrs: (class: "cap cap-" + level),
  [#body],
)

= The tools

- #strong[#link(u("/"))[#mindtape]]: one Typst file per task in your repo,
  checked against rules you set.
- #strong[#link(dstask-url)[dstask]]: one YAML file per task in a git
  repository of its own, `~/.dstask`, synced with git. It's a single Go binary
  for personal tasks.
- #strong[#link(taskwarrior-url)[Taskwarrior]]: a command-line tracker with its
  own database, an urgency score, recurring tasks, user-defined attributes and
  hooks.
- #strong[#link(todotxt-url)[todo.txt]]: one plain-text file with a task per
  line, read by apps on many platforms.
- #strong[#link(orgmode-url)[Org-mode]]: Emacs outlines whose headlines are
  tasks, with an agenda and export to HTML, LaTeX and PDF.
- #strong[#link(git-bug-url)[git-bug]]: issues stored as git objects, with a
  terminal and web UI and bridges to GitHub, GitLab, Jira and Launchpad.
- #strong[#link(beads-url)[Beads]]: an issue tracker for coding agents, kept in
  a Dolt database, with atomic claims and an MCP server.
- #strong[#link(backlog-url)[Backlog.md]]: one Markdown file per task in the
  repo, with a terminal and web Kanban board and an MCP server.

= Feature by feature

#el("div", attrs: (class: "matrix-frame"), el(
  "div",
  attrs: (class: "matrix"),
  table(
    columns: 9,
    align: left,
    table.header(
      [Feature],
      [#link(u("/"))[#mindtape]],
      [#link(dstask-url)[dstask]],
      [#link(taskwarrior-url)[Taskwarrior]],
      [#link(todotxt-url)[todo.txt]],
      [#link(orgmode-url)[Org-mode]],
      [#link(git-bug-url)[git-bug]],
      [#link(beads-url)[Beads]],
      [#link(backlog-url)[Backlog.md]],
    ),

    [#link(u("/concepts/"))[Where tasks live]],
    [One `.typ` file per task],
    [One YAML file per task in `~/.dstask`],
    [SQLite database],
    [One text file, a line per task],
    [Headlines in `.org` files],
    [Git objects],
    [Dolt database in `.beads/`],
    [One Markdown file per task],

    [#link(u("/concepts/"))[Changed in the same pull request as the code]],
    [#cap("green")[Yes]],
    [#cap("grey")[No, its own repo]],
    [#cap("grey")[No]],
    [#cap("amber")[If you keep the file in the repo]],
    [#cap("amber")[If you keep the files in the repo]],
    [#cap("grey")[No, its own git refs]],
    [#cap("grey")[No, synced over its own git ref]],
    [#cap("green")[Yes]],

    [#link(u("/rules/"))[Rules the project sets]],
    [#cap("green")[Checked by `mt check` and before every write]],
    [#cap("amber")[Fixed fields]],
    [#cap("amber")[Fixed attributes and typed UDAs]],
    [#cap("grey")[None]],
    [#cap("grey")[None]],
    [#cap("amber")[Fixed fields]],
    [#cap("amber")[Fixed schema]],
    [#cap("amber")[Checked when its own commands write]],

    [#link(u("/links/"))[Dependencies]],
    [#cap("green")[Typed links: `depends-on`, `parent` and more]],
    [#cap("grey")[No]],
    [#cap("green")[`depends`]],
    [#cap("grey")[No]],
    [#cap("amber")[Child entries and `ORDERED`]],
    [#cap("grey")[No]],
    [#cap("green")[Typed: `blocks`, `parent-child` and more]],
    [#cap("green")[Yes]],

    [#link(u("/priority/"))[Priority]],
    [Integer scale, or scored from inputs by ICE, RICE, WSJF and others],
    [`P0`–`P3`],
    [`H`/`M`/`L`, and an urgency score computed from many attributes],
    [`(A)`–`(Z)`],
    [`[#A]`–`[#C]` by default],
    [None],
    [`P0`–`P4`],
    [Labels you configure],

    [#link(u("/filters/"))[Query and JSON]],
    [#cap("green")[Filter language; JSON and CSV]],
    [#cap("amber")[Tag, project and text filters; JSON when piped]],
    [#cap("green")[Filter language; `task export` JSON]],
    [#cap("grey")[Text search; no JSON]],
    [#cap("amber")[Agenda searches; no JSON]],
    [#cap("green")[Query language; JSON]],
    [#cap("green")[`bd query`; JSON]],
    [#cap("green")[Filters and fuzzy search; JSON]],

    [#link(u("/templates/"))[Templates]],
    [#cap("green")[Templates with typed slots]],
    [#cap("amber")[New tasks copy a template task]],
    [#cap("grey")[No]],
    [#cap("grey")[No]],
    [#cap("green")[`org-capture` templates]],
    [#cap("grey")[No]],
    [#cap("green")[Formulas, workflow templates]],
    [#cap("amber")[Default Definition of Done items]],

    [#link(u("/recurring/"))[Recurring tasks]],
    [#cap("green")[`recur:` field and `mt recur`]],
    [#cap("grey")[No]],
    [#cap("green")[Yes]],
    [#cap("amber")[Add-ons only]],
    [#cap("green")[Repeaters such as `+1w`]],
    [#cap("grey")[No]],
    [#cap("amber")[Patrol molecules, with no schedule]],
    [#cap("grey")[No]],

    [#link(u("/hooks/"))[Hooks]],
    [#cap("green")[Commands on a status change, which can refuse it]],
    [#cap("grey")[No]],
    [#cap("green")[Scripts on add and modify, which can refuse it]],
    [#cap("grey")[No]],
    [#cap("green")[Emacs Lisp hooks]],
    [#cap("grey")[No]],
    [#cap("amber")[Gates that wait on CI, a PR or a timer]],
    [#cap("amber")[A command on a status change]],

    [#link(u("/agents/"))[Agent support]],
    [#cap("green")[Agent skill; JSON from every read command]],
    [#cap("grey")[No]],
    [#cap("grey")[No]],
    [#cap("grey")[No]],
    [#cap("grey")[No]],
    [#cap("grey")[No]],
    [#cap("green")[MCP server, `bd prime`, setup for agent tools]],
    [#cap("green")[MCP server, agent instructions]],

    [#link(u("/rules/"))[Several agents on one backlog]],
    [#cap("amber")[Assignee and WIP rules; no claim]],
    [#cap("grey")[No]],
    [#cap("grey")[No]],
    [#cap("grey")[No]],
    [#cap("grey")[No]],
    [#cap("grey")[No]],
    [#cap("green")[Atomic claim]],
    [#cap("amber")[Assignee]],

    [#link(u("/cli/"))[Interface]],
    [#cap("amber")[Terminal table; `mt ls -i` picker]],
    [#cap("amber")[Terminal table]],
    [#cap("amber")[Terminal reports]],
    [#cap("green")[Apps on many platforms]],
    [#cap("green")[Emacs agenda]],
    [#cap("green")[Terminal and web UI]],
    [#cap("amber")[CLI; community UIs]],
    [#cap("green")[Terminal and web Kanban board]],

    [#link(REPO + "/blob/main/LICENSE")[License]],
    [MIT],
    [MIT],
    [MIT],
    [GPL-3],
    [GPL-3],
    [GPL-3],
    [MIT],
    [MIT],
  ),
))

= Where #mindtape fits

- #strong[Tasks change with the code.] A task is a file in your working tree,
  so the commit or pull request that does the work also closes the task, and
  whoever reviews it sees both. Backlog.md, Org-mode and todo.txt can keep
  tasks in the repo too, but none of them checks a hand-edited task against
  your rules.
- #strong[You set the rules.] One #link(u("/config/"))[config] sets the
  statuses, the priority and difficulty scales, the allowed tags and estimate
  units, and #link(u("/rules/"))[rules] you write as filter expressions or
  Typst functions. `mt check` applies them in CI, and every command that writes
  a task applies them first.
- #strong[Priority from inputs.] ICE, RICE, WSJF, Eisenhower and MoSCoW turn a
  task's inputs into a priority (#link(u("/priority/"))[Priority]).
  Taskwarrior's urgency is also computed, from fixed attributes such as the due
  date, tags and age, with coefficients you tune.
- #strong[A task is Typst.] A task can import a shared module and call your
  own functions. See #link(u("/self-validating-tasks/"))[Self-validating
    tasks].

= Where another tool fits better

- #strong[Your own tasks across every project.] Taskwarrior and dstask keep one
  list outside any repo. dstask syncs it between machines through git with no
  server.
- #strong[The least tooling.] todo.txt is one text file that any editor opens
  and many apps read.
- #strong[You work in Emacs.] Org-mode gives you an agenda across all your
  files and exports outlines to HTML, LaTeX and PDF.
- #strong[Issues that sync with a hosted tracker.] git-bug imports from and
  exports to GitHub, GitLab, Jira and Launchpad.
- #strong[Many agents taking work from one queue.] Beads lets an agent claim a
  task atomically, so two agents never take the same one. #mindtape has an
  `assignee` field and WIP rules, but no claim.
- #strong[You need a browser board, or an MCP server.] Backlog.md has both,
  and Beads has an MCP server.
- #strong[Maturity.] #mindtape is at version #mt-version. It has fewer users and
  integrations than the tools above. Taskwarrior and todo.txt have years of
  add-ons, integrations and mobile apps.
