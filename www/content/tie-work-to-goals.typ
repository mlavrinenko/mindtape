#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": REPO, u
#import "../utils/demo.typ": fixture-slice
#import "../utils/recipes.typ": own-config, recipe-config

#let meta = (
  route: "/tie-work-to-goals/",
  label: "Tie work to your goals",
  title: "Tie work to your goals",
  summary: "Make each milestone say how it serves your goals, and refuse one that helps none of them.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A teammate opens a milestone to rewrite the logging. It's tidy work, but it
serves none of your goals, and nobody asks why until a week has gone into it.

= Config

#recipe-config("tie-work-to-goals")

The rule hands each open milestone's `x-goals` field to `check-goals`, a Typst
function next to the goals it checks:

#fixture-slice("goals")

= Behavior

The milestone template rates each goal `neutral` unless you answer for it. So
`mt add` refuses a milestone that helps no goal, and files it once one goal
says how it helps:

#terminal("tie-work-to-goals")

= Deep dive

`check` runs a Typst function you write on one field of each task the rule's
`when` matches. Each string the function returns is one error, so `mt` names
every problem at once. #link(u("/self-validating-tasks/"))[Self-validating
  tasks] covers check functions and `x-` fields.

The #link(REPO + "/blob/main/examples/api-service/.mindtape/templates/milestone.typ")[milestone template] imports `helps`, `hurts` and `neutral` and asks for one
rating per goal. Add a goal, and `mt check` names every open milestone that
doesn't rate it yet.

This repo rates every open task against its product and process goals in
#own-config.
