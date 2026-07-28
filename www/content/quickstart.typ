#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, pkg, terminal, terminal-ansi
#import "../utils/site.typ": REPO, u

#show: page.with(
  title: "Quickstart",
  active: "/quickstart/",
  summary: "Install MindTape, scaffold a project, and file your first task.",
)

From nothing to a validated task in a few commands.

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

#raw("cargo install " + cli-crate, lang: "sh") gave you the binary. One more
command gives it something to read:

```sh
mt install
```

This is not optional, and it is easy to skip. Every task file opens by importing
#pkg, and Typst resolves an `@local` package from a
directory on your machine — never from the repository holding the tasks. Until
the prelude is installed there, every task file fails to evaluate, `mt ls` shows
an empty table, and the warnings name a path rather than a fix. It bites twice:
on a fresh machine, and on a repository a colleague set up, which otherwise
looks like it should just work after a clone.

#callout("note")[
  Run it once per machine, not once per project. Any command that meets an
  uninstalled prelude prints a note naming `mt install`, so a forgotten setup
  reports itself rather than looking like an empty backlog.
]

`mt install` also seeds `~/.config/mindtape/defaults.toml` (the global config
layer) and `config.schema.json` (which gives editors completion and validation
over `.mindtape/config.toml`). `--no-defaults` and `--no-schema` skip either.

Then scaffold a project, once per repository:

```sh
mt init                  # interactive: a short questionnaire, then writes .mindtape/config.toml
mt init --preset flow    # skip the questions, write a named starter shape
mt init --no-input       # non-interactive: write the default (flow) without asking
mt init --list-presets   # the other starter shapes
```

With no `--preset` and a terminal, `mt init` walks a short questionnaire over the
configuration axes — workflow, priority, estimate, required fields, organisation,
body headers, recurrence — asking a few core questions, then offering to accept
defaults for the rest, and records the answers as a comment block in the written
file. `--preset X`, `--no-input`, and a piped (non-terminal) stdin all skip it;
every question also has a flag that suppresses just that question.

A preset is a starter shape, not a straitjacket — the written file is yours to
edit. Four ship:

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
`mt preset save <name>` captures the current project's config and templates as one
such reusable preset (and `mt preset ls` lists what exists); a preset may carry an
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

Empty columns hide themselves. An opt-in `file` column (`--columns
'#,title,file'`) shows each task's source path, clickable in a standalone
terminal with `--hyperlinks always` — see #link(u("/cli/"))[the CLI
  reference]. The full #link(u("/filters/"))[filter language] covers tags,
dates, dependency state, and full-text search.

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
  Want the model behind all this? #link(u("/concepts/"))[Concepts] explains why a
  task is a file and what "live evaluation" buys you.
]
