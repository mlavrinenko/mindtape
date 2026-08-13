// docs/src/readme.typ — source for the repo-root README.md. Rendered to
// Markdown by `docs/generate-md.sh` via typlite; see docs/src/lib.typ for
// what that render path can and cannot do.
//
// Content-preserving port of the hand-written README.md, except for the
// `@local/mindtape` version in the Task DSL snippet and the inline `#import`
// mention under "Set up, once per project", both of which now read
// `www/generated/crate-map.json`'s `typst-lib.version` (`www/generate-facts.sh`)
// instead of hand-typing "0.2.0" — see outdatty.yaml's `typst-pkg-version`
// group, which no longer lists README.md for the same reason it never listed
// docs/src/agents.typ.
//
// The centered header is a deliberate rebuild, not a literal port:
// `<p align="center">` is how README.md centers its logo/title/tagline/doc
// links today, but typlite silently strips every `<p>` (docs/src/lib.typ).
// `<h1 align="center">`/`<h4 align="center">` survive both typlite and
// GitHub's HTML sanitizer (`<center>` survives typlite but is stripped by
// GitHub's sanitizer), so the header below uses those instead — closest
// achievable to the original centered look, not a byte-for-byte match. This
// document therefore has no separate `doc-title` ATX `#`: the centered
// `<h1>` IS the title, exactly as in the hand-written original.
#import "lib.typ": fact

#metadata((
  output: "README.md",
)) <mindtape-md>

#let typst-lib-version = fact("crate-map").at("typst-lib").at("version")

#html.elem("h1", attrs: (align: "center"))[
  #html.elem("img", attrs: (
    src: "https://raw.githubusercontent.com/mlavrinenko/mindtape/main/www/assets/images/logo.svg",
    alt: "MindTape",
    width: "120",
  ))
  #html.elem("br")
  MindTape
]

#html.elem("h4", attrs: (align: "center"))[
  A #link("LICENSE")[MIT]-licensed, daemon-free file-based task tracker powered by #link("https://typst.app/")[Typst].
]

#html.elem("h4", attrs: (align: "center"))[
  #link("https://mlavrinenko.github.io/mindtape/")[Documentation] · #link("https://mlavrinenko.github.io/mindtape/quickstart/")[Quickstart] · #link("https://mlavrinenko.github.io/mindtape/comparison/")[Comparison]
]

\-\-\-

MindTape tracks tasks as one-task-per-file Typst artifacts. Each file declares a
task with `#show: task.with(...)`; MindTape live-evaluates the configured roots on
every command — no daemon, no database, no index you maintain. A task's `id`
identifies it in listings, filters and links-by-hand, and defaults to the
filename stem. Links target files, so renaming a task without updating its
dependents fails `mt check` instead of dangling silently.

#let example-lines = (
  "#import \"@local/mindtape:" + typst-lib-version + "\": *",
  "",
  "#show: task.with(",
  "  title: \"Implement authentication\",",
  "  status: wip(2026, 6, 14),",
  "  priority: ice(impact: 8, confidence: 0.9, ease: 7),",
  "  estimate: (days: 1,),",
  "  tags: (\"backend\",),",
  "  links: related(\"write-docs.typ\")[same feature],",
  ")",
  "",
  "== Summary",
  "Wire up the login flow.",
)
#raw(example-lines.join("\n"), lang: "typ", block: true)

Everything scalar or enum is a generic core field constrained by
`.mindtape/config.toml`; tools built on MindTape add their own data through
the `x-<ns>` extension seam, carried through opaquely. See
#link("https://mlavrinenko.github.io/mindtape/concepts/")[Concepts] for the full
model and #link("https://mlavrinenko.github.io/mindtape/dsl/")[Task DSL] for every
field a task can declare.

= Installation

From crates.io:

```bash
cargo install mindtape-cli
```

Or grab a prebuilt binary for Linux, macOS or Windows from the
#link("https://github.com/mlavrinenko/mindtape/releases")[Releases page], or run it
without installing:

```bash
nix run github:mlavrinenko/mindtape -- --help
```

Building from source needs only a Rust toolchain (`cargo build --release`
produces `target/release/mt`); the `nix develop` shell adds the full test and
lint toolchain used by `just check`.

Linux, macOS and Windows each get a prebuilt binary per release;
#link("https://mlavrinenko.github.io/mindtape/internals/")[Internals] covers the two
Windows seams and what CI verifies where.

== Set up, once per machine

```bash
mt install
```

Task files begin with #raw("#import \"@local/mindtape:" + typst-lib-version + "\""), and Typst resolves
`@local` packages from a directory on your machine, not from the repository. So
until `mt install` puts the prelude there, every task file fails to evaluate —
including in a repository someone else set up. `mt install` also seeds the
global defaults and the JSON Schema that gives editors config completion. Any
command that hits an uninstalled prelude says so and names this command.

== Set up, once per project

```bash
mt init
```

Writes `.mindtape/config.toml`. With a terminal and no arguments it asks a short
questionnaire; `--preset <name>` skips it and writes a named starter shape
(`mt init --list-presets`). See
#link("https://mlavrinenko.github.io/mindtape/quickstart/")[Quickstart] for what each
preset is for and
#link("https://mlavrinenko.github.io/mindtape/config/")[Configuration] for the
four-layer merge both commands write into.

= Usage

One task, start to finish — file it, list it, close it, validate the project:

```bash
mt add --set title="Wire OAuth" --set priority=7   # file a task from the default template
mt ls 'status != "done" && priority >= 7'          # query it with a filter expression
mt flip done wire-oauth                            # set its status, dated today
mt check                                           # validate the whole project
```

That is a sliver of the surface. The docs site has the rest:

- #link("https://mlavrinenko.github.io/mindtape/cli/")[CLI reference] — every `mt`
  command and global flag (`-w`/`--worktree`, `@name` worktree aliases,
  `--color`, `--hyperlinks`, `-v`), grouped. Includes `mt find TERM…` (sugar
  over `contains()`) and `mt ls --format path`/`path0` (a bare path list for
  piping into another program, no `jq` needed).
- #link("https://mlavrinenko.github.io/mindtape/filters/")[Filter language] — the
  fields, operators, and functions `mt ls` / `mt rm` accept.
- #link("https://mlavrinenko.github.io/mindtape/templates/")[Templates & slots] —
  every way `mt add` can fill a task from a Typst template.
- #link("https://mlavrinenko.github.io/mindtape/config/")[Configuration] — the
  four-layer config merge, every `.mindtape/config.toml` section, and stem
  naming for minted filenames.
- #link("https://mlavrinenko.github.io/mindtape/import-export/")[Import & export] —
  `mt import` plus `jq` adapters for Taskwarrior and GitHub Issues. There is no
  `mt export` command; the tasks are already plain files, and the page shows
  how to reshape them into another tool's format.
- #link("https://mlavrinenko.github.io/mindtape/agents/")[For AI agents] —
  structured JSON, slot schemas, and driving `mt` from outside the project
  tree.
- #link("https://mlavrinenko.github.io/mindtape/internals/")[Internals] — the crate
  layout, the content-hash cache that keeps live evaluation cheap (a warm
  `mt ls` over 2000 tasks lands at 0.20 s), and what differs per platform.

= Contributing

See #link("CONTRIBUTING.md")[CONTRIBUTING.md].

= Acknowledgements

The logo uses a brain icon from #link("https://lucide.dev")[Lucide] (ISC license).
