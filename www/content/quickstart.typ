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

Every task file opens by importing #pkg, which Typst resolves from a
machine-global `@local` directory rather than the checkout —
#link(u("/config/"))[Configuration] owns that resolution and how to pin a
project to an in-tree prelude instead, and #link(u("/cli/"))[the CLI reference]
lists what `mt install` places.

It also drops a shell completion hook per shell under
`~/.config/mindtape/completions/`; source the one your shell uses from its
startup file and tab completes this project's own stems, statuses, views,
templates and link kinds. The hook asks `mt` on every press rather than
carrying a generated script, so it is never stale and never wrong about the
project you happen to be standing in.

Then scaffold a project, once per repository:

```sh
mt init                  # interactive: a short questionnaire, then writes .mindtape/config.toml
mt init --preset flow    # skip the questions, write a named starter shape
mt init --no-input       # non-interactive: write the default (flow) without asking
mt init --list-presets   # the other starter shapes
```

The questionnaire, the starter shapes and the user presets are
#link(u("/cli/"))[the CLI reference]'s; what the written file may carry is
#link(u("/config/"))[Configuration]'s. The recurrence syntax is on
#link(u("/recurring/"))[Recurring tasks].

= File your first task

`mt add` files from a slot template — #link(u("/templates/"))[Templates &
  slots] owns the mechanics — and this project's template scores priority
through `ice(…)`:

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

#link(u("/filters/"))[Filters] owns the language — tags, dates, dependency
state, full-text search — and #link(u("/cli/"))[the CLI reference] owns the
listing surface: column auto-hide, the `file` column, hyperlinks, `mt find`,
and the machine-readable `--format`s.

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

Wire `mt check` into a pre-commit hook or CI — #link(u("/cli/"))[the CLI
  reference] lists what it verifies, and #link(u("/rules/"))[Rules] the
rules behind it.

#callout("note")[
  #link(u("/concepts/"))[Concepts] has the model behind all this: why a task is a
  file, and what live evaluation means.
]
