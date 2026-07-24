// Render www/generated/config-schema.json — the JSON Schema `mt config
// schema` prints (facts only, generated from the config structs — the exact
// bytes `mt install` also writes) — as the Configuration page's "Sections"
// table. This file owns presentation (row order, TOML-syntax labels); the
// JSON owns the descriptions.
//
// Refs: truth/tasks/generate-the-config-reference-from-the-schema.typ

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

// schemars/rustdoc wraps prose at a fixed column with a single "\n"; a real
// paragraph break is a blank line ("\n\n"). Typst renders an embedded "\n" in
// a string as a hard line break, so a bare wrap would show mid-sentence —
// reflow: join wrapped lines with a space, keep blank-line paragraph breaks.
#let _fmt-desc(desc) = {
  if desc == none { return [] }
  desc.split("\n\n").map(p => [#(p.split("\n").join(" "))]).join(parbreak())
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

// Two-way completeness gate, mirroring cli-gate in cli-ref.typ: every
// top-level schema property, and every `[tasks.*]` nested one — where the
// hand-written table had actually drifted — must be covered by a rendered
// path, and every rendered path must resolve against the schema. Call once
// per page; a newly added config key fails the docs build until it is
// surfaced here.
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
