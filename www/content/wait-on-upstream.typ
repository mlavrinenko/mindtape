#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/problems.typ": problem-config

#let meta = (
  route: "/wait-on-upstream/",
  label: "Wait on upstream",
  title: "Work started before upstream ships the fix",
  summary: "Hold a task until a command says the release it needs has landed, and free it the moment it has.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Your code works around a bug in a library, and the fix ships in the next
release. The task that drops the workaround looks ready, so an agent picks it
up, finds the old version still locked and burns an hour proving it. Or nobody
looks again, and the workaround outlives the bug by a year.

= The config

#problem-config("wait-on-upstream")

`scripts/locked` reads `Cargo.lock`. It exits 0 once the lockfile pins the
version an id names or a later one, and 1 while it pins an older one.

= What `mt` does

The task names what it waits on in `waits-on`. While the lockfile pins 2.3.2,
the task is blocked, `why` quotes the script, and `mt wip` won't start it.
Merge the dependency bot's branch, and the same listing lets it go:

#terminal("wait-on-upstream")

= How it works

Each `[[tasks.waits]]` entry names a checker command. A task lists
`checker:id` entries in `waits-on`, and `mt` runs the checker's `run` line from
the project root with `{id}` filled in. Exit 0 means the wait is over. Exit 1
means the task still waits. Any other exit means `mt` can't tell, and the task
stays blocked. The first line the command prints lands in `why`. Add an entry
with `mt set <task> waits-on+=locked:acme-pay@2.4.0`.

Every command that reads readiness runs checkers, `mt ls` included. So a
checker answers from local state, and fast. `mt` runs each distinct command
line once per call and skips closed tasks. Fetching news over the network is
the tool's own job, on its own schedule, such as a timer, a recipe or the bot
that bumps your lockfile.

`{id}` lands in a shell line as is, so `mt check` refuses an id that a shell
could read as more than a word. An id starts with a letter or digit and holds
letters, digits and `._@/#:+-`. A checker runs with your full authority, like
a `Justfile` recipe, so read a cloned repo's `.mindtape/config.toml` before
you run `mt` in it.

#link(u("/links/"))[Links] covers `depends-on`, the blocker that waits on
another task instead.
