// Renders www/generated/config-schema.json, the JSON Schema `mt config schema`
// prints, as the Configuration page's "Sections" table. This file sets the row
// order and the TOML labels; the schema supplies the descriptions.

// ── resolve a dotted schema path to its property-level description ─────────
// Walks `$ref` indirection through `$defs` one segment at a time. A
// property's own "description" (sibling of its "$ref", not the target def's)
// is the text schemars attaches from the field's doc comment, so that is what
// each step reads. Panics on an unknown segment so a typo in config.typ's own
// section list fails the build instead of rendering a blank cell.
#let _prop-desc(schema, path) = {
  let node = schema
  let desc = none
  for seg in path.split(".") {
    if "properties" not in node or seg not in node.properties {
      panic("config-ref: unknown schema path: " + path)
    }
    let prop = node.properties.at(seg)
    desc = prop.at("description", default: none)
    node = if "$ref" in prop {
      schema.at("$defs").at(prop.at("$ref").split("/").last())
    } else {
      prop
    }
  }
  desc
}

// A description is a doc comment, wrapped at a fixed column with blank lines
// between paragraphs. Each paragraph is evaluated as Typst markup, so a
// backtick span renders as code. Its lines are joined first, because a wrap
// inside a backtick span would print as a line break in the code.
#let _fmt-desc(desc) = {
  if desc == none { return [] }
  desc
    .split("\n\n")
    .map(p => eval(p.split("\n").join(" "), mode: "markup"))
    .join(parbreak())
}

// ── public: the Sections table and its completeness gate ───────────────────

// `sections` is an ordered list of `(path, label)` pairs: `path` resolves the
// purpose text from the schema, `label` is the TOML-syntax heading to show
// (e.g. `"[[tasks.status]]"`). One row per pair.
#let config-sections-table(schema, sections) = table(
  columns: 2,
  table.header([Section], [Purpose]),
  ..sections
    .map(s => (raw(s.at(1)), _fmt-desc(_prop-desc(schema, s.at(0)))))
    .flatten(),
)

// Is `prefix` covered by some rendered path — an exact match, or the ancestor
// of a deeper path actually rendered (e.g. `"tasks.body.headers"` covers both
// `"tasks.body"` and `"tasks"`)?
#let _covered(paths, prefix) = {
  for p in paths {
    if p == prefix or p.starts-with(prefix + ".") { return true }
  }
  false
}

// Two-way completeness check, like cli-gate in cli-ref.typ. Every top-level
// schema property and every `[tasks.*]` property must be covered by a
// rendered path, and every rendered path must resolve against the schema.
// Call once per page. A new config key fails the docs build until config.typ
// lists it.
#let config-gate(schema, sections) = {
  let paths = sections.map(s => s.at(0))
  for p in paths {
    let _ = _prop-desc(schema, p) // unknown path panics; value unused
    if paths.filter(x => x == p).len() > 1 {
      panic("config-ref: path listed more than once in config.typ: " + p)
    }
  }
  for name in schema.properties.keys() {
    if not _covered(paths, name) {
      panic("config-ref: schema property not covered in config.typ: " + name)
    }
  }
  for key in schema.at("$defs").at("TasksConfig").properties.keys() {
    let full = "tasks." + key
    if not _covered(paths, full) {
      panic("config-ref: " + full + " not covered in config.typ")
    }
  }
}
