#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, pkg, terminal, terminal-ansi
#import "../utils/site.typ": REPO, u

#let meta = (
  route: "/quickstart/",
  label: "Quickstart",
  title: "Quickstart",
  summary: "Install MindTape, scaffold a project, and file your first task.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

= Install

#let release = json("../generated/release-targets.json")
// The installable CLI is whichever published crate declares its own
// `[[bin]]` — asserted here so a workflow edit that drops or renames it
// fails the docs build loudly instead of quietly showing a stale crate.
#let cli-crates = release.publish.filter(p => p.bin)
#if cli-crates.len() != 1 {
  panic(
    "quickstart: release-targets.json must name exactly one published crate "
      + "with a [[bin]] (the CLI); found "
      + str(cli-crates.len()),
  )
}
#let cli-crate = cli-crates.at(0).crate
// Each entry carries the artifact name the release workflow actually uploads
// for that target — extension included, so the Windows build reads `.exe`
// here rather than being silently listed as an extensionless file nobody
// could run.
#let targets = release.targets

Install the published CLI crate from crates.io:

#raw("cargo install " + cli-crate, lang: "sh", block: true)

Every tagged release (`.github/workflows/release.yml`) also builds a binary
for #targets.len() platforms, attached to the #link(REPO + "/releases")[Releases
  page] — download whichever matches yours:

#list(..targets.map(t => raw(t.artifact)))

Or run it straight from the flake, without installing anything:

```sh
nix run github:mlavrinenko/mindtape -- --help
```

Or build from source — a Rust toolchain is all it needs:

```sh
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape && cargo build --release   # produces target/release/mt
```

= Set up the environment

The binary needs a prelude to read. Install it once per machine, not once per
project:

```sh
mt install
```

Every task file opens by importing #pkg, and Typst resolves an `@local` package
from a directory on your machine — never from the repository holding the tasks.
Until the prelude is installed there, every task file fails to evaluate and
`mt ls` shows an empty table; any command that meets an uninstalled prelude
prints a note naming `mt install`, so a forgotten setup reports itself rather
than looking like an empty backlog.

`mt install` also seeds `~/.config/mindtape/defaults.toml` (the global config
layer) and `config.schema.json` (which gives editors completion and validation
over `.mindtape/config.toml`). It performs all three installs; `--no-lib`,
`--no-defaults` and `--no-schema` skip any of them.

Then scaffold a project, once per repository:

```sh
mt init                  # interactive: a short questionnaire, then writes .mindtape/config.toml
mt init --preset flow    # skip the questions, write a named starter shape
mt init --no-input       # non-interactive: write the default (flow) without asking
mt init --list-presets   # the other starter shapes
```

With no `--preset` and a terminal, `mt init` walks the configuration axes —
workflow, priority, estimate, required fields, organisation, body headers,
recurrence — asking a few core questions, then offering to accept defaults for
the rest, and records the answers as a comment block in the written file. A
piped (non-terminal) stdin skips the questionnaire the way `--no-input` does;
every question also has a flag that suppresses just that question.

A build offers only the answers it can act on. The `rrule` recurrence syntax
needs an expansion engine behind an off-by-default cargo feature, so an `mt`
built without it neither lists that choice nor accepts `--recurrence rrule` —
the flag fails naming the feature, rather than writing a config whose first
`rrule(...)` task would fail `mt check`. An axis left with one answer is not
asked at all.

A preset is a starter shape; the written file is yours to edit. Four ship:

/ #raw("todone"): open or finished, nothing else constrained.
/ #raw("flow"): a lifecycle with enforced transitions (the default). Suits one
  person or a small team.
/ #raw("personal"): life admin — capture to an inbox, hide-until dates, a weekly
  review view, folders as contexts.
/ #raw("gated"): for work handed to an AI agent — required spec sections, a
  review state that `done` cannot skip, a `[[on.flip]]` gate command that must
  pass before a task may close, no leftover review comments, and a difficulty
  rubric for routing a task to a model tier.

`--annotate` keeps the preset's explanatory comments in the written file. User
presets live in `~/.config/mindtape/presets` and shadow the built-ins by name.
`mt preset save <name>` captures the current project's config and templates as
such a preset (`mt preset ls` lists what exists); a preset may carry an
`[overlay]` block for config the axes cannot express, merged last over the base.
See #link(u("/config/"))[Configuration] for the layering.

= File your first task

`mt add` fills a Typst template, injects the status and id, and validates the
result. The example's template scores priority through `ice(…)`, so each leaf is
its own `--set` — no hand-set priority int:

```sh
mt add --set title="Practice pedaling technique" \
  --set summary="Half-pedal control on sustained chords." \
  --set impact=6 --set ease=5
```

#terminal(read("../generated/add.txt"))

The written file is plain Typst — no template machinery survives:

#raw(read("../generated/add-written.typ"), lang: "typ", block: true)

More ways to file — positional titles, editor mode, named templates — are on
#link(u("/templates/"))[Templates & slots].

= List and query

```sh
mt ls
```

#terminal-ansi(read("../generated/ls.ansi"))

Pass a filter expression to narrow it down — here, everything not yet terminal:

#terminal-ansi(read("../generated/ls-open.ansi"))

Empty columns hide themselves, and an `assignee` column appears only when the
rows disagree about who works them. An opt-in `file` column (`--columns
'#,title,file'`) shows each task's source path, clickable in a standalone
terminal with `--hyperlinks always` — see #link(u("/cli/"))[the CLI
  reference]. The full #link(u("/filters/"))[filter language] covers tags,
dates, dependency state, and full-text search.

`mt find login bug` is a shortcut for the last of those —
`mt ls 'contains("login") && contains("bug")'` — and takes every `ls` flag.
`mt ls --format path` prints a bare list of task paths (`path0` NUL-delimits
it for `xargs -0`), for composing with another program:

```sh
rg TODO $(mt ls ready --format path)
```

= Change status

Every status in the legend is also a verb. Set one, dated today, without opening
the file:

#terminal(read("../generated/done.txt"))

`mt done x` is shorthand for `mt flip done x`. See #link(u("/cli/"))[the CLI
  reference] for the full command set.

= Validate

```sh
mt check
```

#terminal(read("../generated/check.txt"))

`mt check` verifies every task against the project config: enum values, required
fields, numeric bounds, link resolution, and dependency cycles. Wire it into a
pre-commit hook or CI.

#callout("note")[
  #link(u("/concepts/"))[Concepts] has the model behind all this: why a task is a
  file, and what live evaluation means.
]
