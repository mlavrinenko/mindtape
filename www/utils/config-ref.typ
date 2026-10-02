// Renders www/generated/config-schema.json, the JSON Schema `mt config schema`
// prints, as the Configuration page's "Sections" table and as the key and
// value tables of the pages that explain one section. This file sets the row
// order and the TOML labels; the schema supplies the descriptions and
// defaults.

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

// ── public: one section's keys ──────────────────────────────────────────────

// Step through what a property points at to the object that holds its keys:
// a `$ref` to its def, an array to its items, an optional (`anyOf` with
// `null`) to its one non-null branch.
#let _target(schema, node) = {
  let n = node
  for _ in range(8) {
    if "$ref" in n {
      n = schema.at("$defs").at(n.at("$ref").split("/").last())
    } else if "items" in n {
      n = n.items
    } else if "anyOf" in n {
      n = n.anyOf.find(b => b.at("type", default: none) != "null")
    } else { break }
  }
  n
}

// A schema default as it would be written in TOML; none when the schema
// carries no default or the default is null.
#let _toml-value(v) = {
  if v == none { none } else if type(v) == str { "\"" + v + "\"" } else if (
    type(v) == bool
  ) { if v { "true" } else { "false" } } else if type(v) == array {
    "[" + v.map(_toml-value).join(", ") + "]"
  } else { str(v) }
}

// The keys one section takes, read from the schema: key, default, and the
// key's own doc comment. `path` is dotted from the top of the config, as in
// config-sections (`"tasks.links.kinds"` is the `[[tasks.links.kinds]]`
// entry). An unknown segment panics, so a renamed key fails the docs build.
#let config-keys-table(schema, path) = {
  let node = schema
  for seg in path.split(".") {
    node = _target(schema, node)
    if "properties" not in node or seg not in node.properties {
      panic("config-ref: unknown schema path: " + path)
    }
    node = node.properties.at(seg)
  }
  let obj = _target(schema, node)
  if "properties" not in obj {
    panic("config-ref: " + path + " has no keys of its own")
  }
  table(
    columns: 3,
    table.header([Key], [Default], [Purpose]),
    ..obj
      .properties
      .pairs()
      .map(((key, prop)) => {
        let d = _toml-value(prop.at("default", default: none))
        let desc = prop.at(
          "description",
          default: _target(schema, prop).at("description", default: none),
        )
        (raw(key), if d == none { [] } else { raw(d) }, _fmt-desc(desc))
      })
      .flatten(),
  )
}

// The values one enum-valued key takes, read from the schema def `name`:
// each value with its doc comment.
#let config-values-table(schema, name) = {
  let def = schema.at("$defs").at(name, default: none)
  if def == none or "oneOf" not in def {
    panic("config-ref: " + name + " is not a documented enum in the schema")
  }
  table(
    columns: 2,
    table.header([Value], [Meaning]),
    ..def
      .oneOf
      .map(v => (raw(v.const), _fmt-desc(v.at("description", default: none))))
      .flatten(),
  )
}
