#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u

#show: page.with(
  title: "Comparison",
  active: "/comparison/",
  summary: "Where MindTape sits among file-based and CLI task trackers, and when another tool is the better fit.",
)

MindTape is not trying to replace every task tracker. It occupies a specific
niche: #strong[typed tasks, one per file, versioned with the code, validated by
config, with no daemon]. This page is an honest map of the neighbourhood.

= At a glance

#table(
  columns: (auto, auto, auto),
  table.header([Tool], [Storage], [Shape of a task]),
  [MindTape], [One `.typ` file per task, in your repo], [Typed Typst, config-validated],
  [Taskwarrior], [One central database], [Attributes + UDAs, rich reports],
  [todo.txt], [One shared text file], [A single formatted line],
  [Org-mode], [Emacs `.org` files], [Headline + properties, prose-first],
  [dstask], [One file per task, git-backed], [Markdown-ish front matter],
  [GitHub Issues], [Hosted service], [Web form, collaborative],
)

= When MindTape fits

- You want tasks to live #strong[in the repo], reviewed and versioned with the
  code they describe.
- You want tasks that are #strong[typed and validated] — enum statuses, bounded
  scales, resolvable links, checked in CI.
- You want to #strong[compose] tasks: `#import` shared Typst libraries, compute
  values, reuse templates.
- You want the same file to be a queryable row #strong[and] a typeset document.
- You want a #strong[local, offline, daemon-free] tool that AI agents can drive.

= When another tool fits better

- #strong[Taskwarrior] — you want a mature, central personal tracker with deep
  reports, recurrence, and a huge extension ecosystem, and you do not need tasks
  in-repo.
- #strong[todo.txt] — you want the absolute minimum: one line, one file, zero
  schema.
- #strong[Org-mode] — you live in Emacs and want tasks woven into prose, agendas,
  and literate documents.
- #strong[GitHub Issues / Jira] — you need hosted, multi-user collaboration with
  notifications and a web UI.

= The closest cousin

dstask shares the most DNA — one file per task, git as the store. MindTape differs
in that a task is #strong[Typst], not front matter: it is typed and composable, the
project #link(u("/config/"))[config] drives validation, and #link(u("/priority/"))[priority
frameworks] score typed inputs. The trade is a Typst toolchain versus plain
Markdown.

#callout("note")[
  Pick the tool that matches how you work. If tasks-as-code, in-repo, and typed is
  the shape you want, #link(u("/quickstart/"))[start here].
]
