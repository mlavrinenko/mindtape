// www/utils/demo.typ — the opening README.md and the site root share.
//
// `showcase` quotes a task from examples/api-service, through
// `fixture-slice`. `recipe-table` lists the Recipes nav group, one row per
// page: its title and summary from the page manifest, and the first error
// line its capture printed, so no row quotes a refusal that did not happen.
// `typst-section` says why tasks are Typst files. Paths are relative to this
// file, so they resolve under tola's `--root www` and typlite's `--root .`.
// The caller passes its link function, `(route, body) => content`, because
// the two render paths build site links differently.

#import "site.typ": NAV, REPO, SITE, mindtape, page-meta, render-summary
#import "slices.typ": checked-slices

#let tagline = SITE.tagline

// The tagline and the pitch, which open README and the site root.
#let lede = render-summary(SITE.tagline + " " + SITE.pitch)

// The fixture's Typst slices: the task README opens with, and the goals
// module the tie-work-to-goals recipe quotes.
#let _fact = json("../generated/api-service.json")
#let _slices = checked-slices(_fact, "api-service", ("showcase", "goals"))

// Slice `id` under a link to the file it comes from.
#let fixture-slice(id) = {
  let path = _fact.root + "/" + _slices.at(id).file
  par(link(REPO + "/blob/main/" + path, raw(path)))
  raw(_slices.at(id).code, lang: "typ", block: true)
}

// The line that introduces the showcase task, in README and on the site root.
#let showcase-intro = "Here's a real task."

// typlite joins two `par` calls into one line, so the paragraphs are markup.
#let showcase = [
  #showcase-intro

  #fixture-slice("showcase")
]

// The first error line of the first command that failed in capture `id`.
#let _refusal(id) = {
  let steps = json("../generated/" + id + ".json").steps
  let step = steps.find(s => s.code != 0)
  assert(step != none, message: id + ".json records no failing command")
  let lines = step.out.replace(regex("\u{1b}\[[0-9;]*m"), "").split("\n")
  let line = lines.find(l => lower(l).starts-with("error"))
  assert(
    line != none,
    message: id + ".json: the failing command printed no error line",
  )
  line
}

// A line as code spans split at its backticks, a space apart. typlite writes a
// raw span that holds a backtick between single backticks, which breaks the
// span, and it glues two spans that touch into one.
#let _code(line) = (
  line.split("`").map(str.trim).filter(part => part != "").map(raw).join[ ]
)

// The Recipes nav group's routes, in nav order.
#let recipe-routes = NAV.find(group => group.title == "Recipes").items

#let recipe-table(link) = {
  heading(level: 1)[Bring your own rules]
  [
    None of these checks ship with #mindtape. Each one is a few lines of
    config. Here's what `mt` says when a task breaks one.
  ]
  let rows = ()
  for route in recipe-routes {
    // The manifest is empty while www/generate-site-pages.sh builds it.
    let meta = page-meta(route)
    if meta == none { continue }
    rows.push(link(route, meta.title))
    rows.push(render-summary(meta.summary))
    rows.push(_code(_refusal(route.trim("/"))))
  }
  table(
    columns: 3,
    table.header([Recipe], [What it does], [What `mt` prints]),
    ..rows,
  )
}

#let typst-section(link) = {
  heading(level: 1)[The elephant in the room: Typst vs Markdown + frontmatter]
  [
    Why pick #std.link("https://typst.app/")[Typst] over Markdown to power a
    local task tracker? Here are a few reasons.

    + #strong[Language server autocompletion.] `task` declares every field as
      a named parameter, so your editor completes field names and flags a
      misspelled one.
    + #strong[Shared helpers.] `#import` your own Typst module to fill tasks
      the way you like.
    + #strong[Computed fields.] Fill a field with an expression, like
      `due: release - duration(days: 14)`. Change `release` in your shared
      module, and every deadline moves.
    + #strong[Custom validators.] Bring your own checks by writing a Typst
      function. See #link("/self-validating-tasks/")[Self-validating tasks].
  ]
}
