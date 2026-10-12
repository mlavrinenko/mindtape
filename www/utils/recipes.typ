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

// The slice of recipe `id`, under a link to the file it comes from.
#let recipe-config(id) = {
  let slice = _slices.at(id)
  par(link(REPO + "/blob/main/" + slice.file, raw(slice.file)))
  raw(slice.code, lang: "toml", block: true)
}

// This repo's own config, which runs the same kind of rule on its backlog.
#let own-config = link(
  REPO + "/blob/main/.mindtape/config.toml",
  raw(".mindtape/config.toml"),
)
