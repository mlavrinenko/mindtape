#import "../utils/page.typ": page
#import "../utils/ui.typ": hero
#import "../utils/site.typ": u

// The same real, closed backlog task README.md's opening quotes, derived once
// by www/generate-facts.sh (`readme-task.json`) so the two pages never
// disagree about which task or what its marker says — that script's own
// section argues the trim and the drift check on the hand-authored YAML
// twin. Read directly, not through docs/src/lib.typ's `fact`: this is a
// different render path (tola, not typlite) with its root at www/, and a
// leading-slash path there would look for truth/tasks/ under www/ itself.
#let readme-task = json("../generated/readme-task.json")

#let meta = (
  route: "/",
  label: "Overview",
  title: "MindTape",
  summary: "A daemon-free task tracker whose tasks are Typst files, live-evaluated on every command.",
  home: true,
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

#hero()

= What it is

A task is a single Typst file. You write it, MindTape reads it: every command
re-evaluates the files under your configured roots with the Typst compiler, then
applies your config. A task's identity is its filename stem. Core fields are
constrained by one config file, and tools built on MindTape add their own through
an `x-<ns>` extension seam — #link(u("/concepts/"))[Concepts] has the model.

= One task, two ways

Say the same task lived as a Markdown file with YAML frontmatter instead,
checked by a script. Here is one real, closed task from this repo's own
backlog, both ways.

#raw(readme-task.yaml, lang: "yaml", block: true)

Nothing above is checked until a script reads the file and does it by hand:
parses `status_date`, keeps `priority`'s three numbers in bounds, resolves
`depends_on` against another file on disk, and confirms every `x_goals` key
is present with a reason.

The same task as Typst — #raw("truth/tasks/" + readme-task.stem + ".typ")
(trimmed here: the file continues with `== Problem`, `== TO-BE`,
`== Changes`, `== Rollback`, `== Out of scope` and `== Dogfood`):

#raw(readme-task.marker, lang: "typ", block: true)

What only the Typst side does:

- #strong[Typed values with their own shape.] `status: wip(2026, 9, 14)[note]`,
  `priority: ice(impact: 8, confidence: 0.7, ease: 6)` — validated on every
  command, never parsed as strings.
- #strong[Typed links.] `depends-on("other.typ")` — resolved and checked on
  every command, feeding `ready`/`blocked` and the graph; a renamed target
  fails `mt check` instead of dangling.
- #strong[A repo-invented field.] Here `x-goals`, checked by the repo's own
  Typst function named in config, and refused before `mt add` writes:

```toml
[[tasks.rules]]
check = { typst = "truth/lib.typ", function = "check-goals", field = "x-goals" }
```

= Why Typst

A task file is real code.

- #strong[Plain text.] Read it, diff it, grep it, version it with the code it tracks.
- #strong[No lock-in.] No proprietary store, no cloud, no binary blobs.
- #strong[Composable.] `#import` a shared library or your own helpers, compute
  values, reuse definitions across tasks.

= Built for humans and agents

The CLI is scriptable and quiet by default: table, JSON, CSV, or a bare path
list for piping into another program, honest exit codes, and a filter language
for querying (`mt find` is a shortcut over it). Structured JSON and slot
schemas make it equally legible to AI coding agents — see
#link(u("/agents/"))[For AI agents].
