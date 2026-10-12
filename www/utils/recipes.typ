// www/utils/recipes.typ — the config block each page in the Recipes nav group
// shows.
//
// Each `# docs:slice api-service/<id>` block in examples/api-service's config
// is one recipe, and the page at /<id>/ shows it. checked-slices fails the
// build when the slice ids and the group's routes differ, so a slice cannot
// ship without its page, nor a page without its slice.

#import "site.typ": NAV, REPO
#import "slices.typ": checked-slices

#let _ids = (
  NAV
    .find(group => group.title == "Recipes")
    .at("items")
    .map(route => route.trim("/"))
)

#let _slices = checked-slices(
  json("../generated/api-service-config.json"),
  "api-service-config",
  _ids,
)

// The slice of recipe `id`, under a link to the file it comes from. `after`
// is TOML text the block shows below the slice.
#let recipe-config(id, after: none) = {
  let slice = _slices.at(id)
  let code = (slice.code, after).filter(part => part != none).join("\n")
  par(link(REPO + "/blob/main/" + slice.file, raw(slice.file)))
  raw(code, lang: "toml", block: true)
}

// www/generate-rule-packs.sh copies the mindtape-tasks pack here verbatim.
#let _pack-text = read("../generated/mindtape-tasks-pack.toml")
#let _pack = toml(bytes(_pack-text))

// The pack's rules named in `names`, as written in the pack and commented
// out, under a line that counts the rest. A rule runs from the `[[...]]`
// header above its `name` to the first blank line.
#let pack-excerpt(names) = {
  let lines = _pack-text.split("\n")
  let rule = name => {
    let at = lines.position(line => line == "name = \"" + name + "\"")
    assert(at != none, message: "mindtape-tasks has no rule " + name)
    let start = at
    while not lines.at(start).starts-with("[[") { start -= 1 }
    let end = at
    while end < lines.len() and lines.at(end) != "" { end += 1 }
    lines.slice(start, end)
  }
  let total = (
    _pack.tasks.rules.len()
      + _pack.tasks.body.headers.rules.len()
      + _pack.tasks.body.content.rules.len()
      + _pack.commits.rules.len()
  )
  let lines = (
    "As if you wrote these rules yourself, plus "
      + str(total - names.len())
      + " more:",
    ..names.map(name => ("", ..rule(name))).flatten(),
  )
  lines.map(line => if line == "" { "#" } else { "# " + line }).join("\n")
}

// This repo's own config, which runs the same kind of rule on its backlog.
#let own-config = link(
  REPO + "/blob/main/.mindtape/config.toml",
  raw(".mindtape/config.toml"),
)
