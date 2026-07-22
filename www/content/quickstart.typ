#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal, terminal-ansi, callout
#import "../utils/site.typ": u, REPO

#show: page.with(
  title: "Quickstart",
  active: "/quickstart/",
  summary: "Install MindTape, scaffold a project, and file your first task.",
)

From nothing to a validated task in a few commands.

= Install

Run it straight from the flake, or build from source:

```sh
# Run without installing
nix run github:mlavrinenko/mindtape -- --help

# Or build from source
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape && cargo build --release   # produces target/release/mt
```

= Set up the environment

Once per machine, install the Typst prelude (so `#import` resolves) and seed the
global defaults:

```sh
mt install        # installs @local/mindtape + ~/.config/mindtape/defaults.toml
```

Then scaffold a project, once per repository:

```sh
mt init                  # writes .mindtape/config.toml in the current directory
mt init --list-presets   # the other starter shapes
```

A preset is a starter shape, not a straitjacket — the written file is yours to
edit. Four ship:

/ #raw("todone"): open or finished, nothing else constrained.
/ #raw("flow"): a lifecycle with enforced transitions (the default). Suits one
  person or a small team.
/ #raw("personal"): life admin — capture to an inbox, hide-until dates, a weekly
  review view, folders as contexts.
/ #raw("gated"): for work handed to an AI agent — required spec sections, and a
  review state that `done` cannot skip.

`--annotate` keeps the preset's explanatory comments in the written file. User
presets live in `~/.config/mindtape/presets` and shadow the built-ins by name.
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

Empty columns hide themselves. The full #link(u("/filters/"))[filter language]
covers tags, dates, dependency state, and full-text search.

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
