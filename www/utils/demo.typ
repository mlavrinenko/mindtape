// www/utils/demo.typ — the opening README.md and the site root share.
//
// `showcase` quotes a task from examples/api-service. `recipe-table` lists the
// Recipes nav group, one row per page: its title and summary from the page
// manifest, and the first error line its capture printed, so no row quotes a
// refusal that did not happen. Paths are relative to this file, so they
// resolve under tola's `--root www` and typlite's `--root .`. The caller
// passes its link function, `(route, body) => content`, because the two
// render paths build site links differently.

#import "site.typ": NAV, REPO, SITE, page-meta, render-summary
#import "slices.typ": checked-slices

#let tagline = SITE.tagline

#let intro = [
  Your tasks live in the repo, one #link("https://typst.app/")[Typst] file
  each. Edit them in any editor, or let `mt` commands handle the fiddly parts,
  like status dates and link paths. Rules go in `.mindtape/config.toml`, like
  "one task in progress at a time" or "every commit names a task", and `mt`
  refuses any edit or commit that breaks them. There's no server or database.
  It just reads the files in your project.
]

#let _fact = json("../generated/api-service.json")
#let _task = checked-slices(_fact, "api-service", ("showcase",)).showcase

#let showcase = {
  let path = _fact.root + "/" + _task.file
  par(link(REPO + "/blob/main/" + path, raw(path)))
  raw(_task.code, lang: "typ", block: true)
}

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
  heading(level: 1)[Flexible remedy]
  [
    Here are some examples of what a few lines of config can do. These checks
    aren't built in. Mindtape is flexible enough to allow them, and it can do
    much, much more.
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
