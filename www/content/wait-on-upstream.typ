#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/recipes.typ": recipe-config

#let meta = (
  route: "/wait-on-upstream/",
  label: "Wait on upstream",
  title: "Wait on upstream",
  summary: "Hold a task until a command says the release it needs has landed, and free it the moment it has.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Imagine you have a workaround for a bug in an external library, and the fix
ships in the next release. The task that drops the workaround looks ready, so an agent picks it
up, finds the old version still locked and burns an hour proving it. Or nobody
looks again, and the workaround outlives the bug by a year.

= Config

#recipe-config("wait-on-upstream")

`scripts/locked` reads `Cargo.lock`. It exits 0 once the lockfile pins the
version an id names or a later one, and 1 while it pins an older one.

= Behaviour

The task names what it waits on in `waits-on`. In a fresh clone `mt` runs no
checker, so the task is blocked as unknown and `mt` asks you to allow the
config's commands. Once you do, `why` quotes the script while the lockfile pins 2.3.2,
and `mt wip` won't start the task. Merge the dependency bot's branch, and the
same listing lets it go:

#terminal("wait-on-upstream")

= Deep dive

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
letters, digits and `._@/#:+-`.

A checker runs with your full rights, like a `Justfile` recipe, and `mt ls`
would run it the moment you clone. So `mt` runs none until
#link(u("/cli/allow/"))[`mt allow`] records the config's commands for the
repo. Until then each command warns once and lists them, every wait reads
`unknown: not allowed`, and `mt wip --force` still starts a task. The approval
lives outside the repo and covers each of its worktrees. Change a `run` line
and `mt` asks again.

#link(u("/links/"))[Task links] covers `depends-on`, the blocker that waits on
another task instead.
