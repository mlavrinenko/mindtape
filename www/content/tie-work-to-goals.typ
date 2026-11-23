#import "../utils/page.typ": page
#import "../utils/site.typ": REPO, u
#import "../utils/ui.typ": terminal
#import "../utils/recipes.typ": own-config, recipe-config
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/tie-work-to-goals/",
  label: "Tie work to your goals",
  title: "Tie work to your goals",
  summary: "Make each milestone say how it serves your goals, and refuse one that helps none of them.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Someone opens a milestone to rewrite the logging. The work is tidy, but the
milestone serves none of your goals, and a week passes before anyone asks why.

The core keeps the mechanism, not the policy. A task body can emit metadata
markers, and a `[[tasks.rules]] check` with `field = "markers"` hands them to a
Typst function your project writes. The goal DSL is project Typst. The blocks
below are `examples/goals`, a project whose tasks rate two goals and whose
check refuses a task that helps neither.

= Declare the goals

A goal value carries the infinitive a task writes after `to:`, the gerund it
writes after `from:`, and the sentence a render shows. The project keeps its
registry in `goals.typ`:

#let code = checked-slices(
  json("../generated/goals.json"),
  "goals",
  ("goals", "goal", "verdicts", "check", "lib", "ship-the-search-index"),
)

#raw(
  render-slices(code, (("goals", "the goals every task rates"),)),
  lang: "typ",
  block: true,
)

The helpers are project Typst too. `goal(...)` derives the name a task writes,
and each verdict emits one marker under a `goal` discriminator:

#raw(
  render-slices(code, (("goal", "one value, named by its infinitive"),)),
  lang: "typ",
  block: true,
)

#raw(
  render-slices(code, (("verdicts", "one marker per verdict"),)),
  lang: "typ",
  block: true,
)

= Check the markers

`check-goal-over(goals, markers)` reads the reserved `markers` input: every
metadata marker in the body, in document order. It returns one string per
problem, so `mt` names every missing goal at once:

#raw(
  render-slices(code, (("check", "one message per problem"),)),
  lang: "typ",
  block: true,
)

`lib.typ` wraps the check with this project's registry, and the config names
that file and function beside `field = "markers"`:

#raw(
  render-slices(code, (("lib", "the rule's entry point"),)),
  lang: "typ",
  block: true,
)

#recipe-config("tie-work-to-goals")

= Rate a task

A task imports the calls and writes `== Goals`, one verdict per goal. `Helps`
takes the goal it serves and a reason, and `Neutral` leaves a goal alone:

#raw(
  render-slices(code, (
    ("ship-the-search-index", "helps one goal and leaves the other alone"),
  )),
  lang: "typ",
  block: true,
)

The example's task template carries the whole section in one `goals` raw slot,
and rates every goal neutral unless you answer for it. Add a goal to the
registry, and the next `mt add` asks for its verdict.

= See it refuse

The template's default leaves both goals neutral, so `mt add` refuses a task
that helps none of them, and files it once the goals slot says how it helps:

#terminal("tie-work-to-goals")

= Check functions

`check` runs a Typst function you write on one input of each task the rule's
`when` matches. Each string the function returns is one error, so `mt` names
every problem at once. #link(u("/self-validating-tasks/"))[Self-validating
  tasks] covers a field your project adds. This recipe reads `markers`, the one
input the core reserves. The
#link(REPO + "/tree/main/examples/goals")[goals example] is the whole project:
the module, the registry, the wrapper, the template and two tasks.

This repo rates every open task against its product and process goals in
#own-config, with its own module in
#link(REPO + "/blob/main/truth/goal-dsl.typ")[`truth/goal-dsl.typ`].
