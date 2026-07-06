#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal, callout, mt-import
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
mt init           # writes .mindtape/config.toml in the current directory
```

`mt init --full` writes a fully-expanded config with every section spelled out;
the bare `mt init` writes a slim scaffold that inherits the shipped defaults.
See #link(u("/config/"))[Configuration] for the layering.

= File your first task

`mt add` fills a Typst template, injects the status and id, and validates the
result:

```sh
mt add --set title="Wire OAuth login" --set priority=8
```

#terminal("$ mt add --set title=\"Wire OAuth login\" --set priority=8
wrote tasks/wire-oauth-login.typ")

The written file is plain Typst — no template machinery survives:

#mt-import("#show: task.with(
  title: \"Wire OAuth login\",
  priority: 8,
  status: proposed(2026, 7, 6),
)

== Summary
")

More ways to file — positional titles, editor mode, named templates — are on
#link(u("/templates/"))[Templates & slots].

= List and query

```sh
mt ls
```

#terminal("$ mt ls
#   STATUS    READY    PRIORITY  DUE         TITLE
1e  wip       ready    high      2026-07-20  Wire OAuth login
2g  proposed  blocked  normal                Write onboarding docs")

Pass a filter expression to narrow it down:

#terminal("$ mt ls 'status != \"done\" && priority >= 7'
#   STATUS  PRIORITY  DUE         TITLE
1e  wip     high      2026-07-20  Wire OAuth login")

Empty columns hide themselves. The full #link(u("/filters/"))[filter language]
covers tags, dates, dependency state, and full-text search.

= Change status

Every status in the legend is also a verb. Set one, dated today, without opening
the file:

#terminal("$ mt done wire-oauth-login
flipped wire-oauth-login (Wire OAuth login): wip → done")

`mt done x` is shorthand for `mt flip done x`. See #link(u("/cli/"))[the CLI
reference] for the full command set.

= Validate

```sh
mt check
```

#terminal("$ mt check
ok: 3 task(s) valid")

`mt check` verifies every task against the project config: enum values, required
fields, numeric bounds, link resolution, and dependency cycles. Wire it into a
pre-commit hook or CI.

#callout("note")[
  Want the model behind all this? #link(u("/concepts/"))[Concepts] explains why a
  task is a file and what "live evaluation" buys you.
]
