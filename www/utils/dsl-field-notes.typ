// What each core task field means, keyed by field name. The /dsl/ page reads
// the field list and each default from www/generated/dsl-fields.json, which
// www/generate-facts.sh reads off lib/prelude.typ's `task` signature, so this
// dictionary holds only the meaning. `just docs::facts` fails on a field
// in that list with no entry here.

#import "site.typ": u

#let dsl-field-notes = (
  "title": [The task's title. The built-in template asks for this field
    alone.],
  "status": [A status, the date it was set and an optional note:
    `wip(2026, 7, 6)[behind a flag]`. See "Writing a status" below.],
  "priority": [A number on the project's priority scale, or a framework call
    such as `ice(...)`. See #link(u("/priority/"))[Priority].],
  "difficulty": [A number on the difficulty scale, or a framework call, as for
    `priority`.],
  "kind": [One of the kinds the project's config lists, such as
    `"feature"`.],
  "assignee": [Who is working on the task, as a string. The project can limit
    the names.],
  "estimate": [A dictionary of units to amounts: `(days: 1, hours: 2)`. `mt`
    checks the units against the project's list and never converts between
    them. See #link(u("/estimates/"))[Estimates].],
  "links": [Typed edges to other tasks, joined with `+`:
    `depends-on("a.typ") + related("b.typ")[why]`. See
    #link(u("/links/"))[Task links].],
  "waits-on": [What the task waits on outside the repo, as `checker:id`
    strings: `("locked:acme-pay@2.4.0",)`. A configured command answers each
    one. See #link(u("/wait-on-upstream/"))[Wait on upstream].],
  "tags": [An array of strings: `("backend", "auth")`. One tag needs a
    trailing comma, `("backend",)`, because `("backend")` is a string.],
  "due": [The task's deadline, a date.],
  "start": [The date work on the task can begin. The default
    #link(u("/named-sets/#views"))[view] hides the task until then.],
  "reviewed": [The date someone last reviewed the task. `mt review` sets it to
    today.],
  "recur": [A rule that makes the task repeat: `every(weeks: 1)`. See
    #link(u("/recurring/"))[Recurring tasks].],
  "id": [The task's id, when it should differ from the filename stem. See
    #link(u("/concepts/"))[Concepts].],
)
