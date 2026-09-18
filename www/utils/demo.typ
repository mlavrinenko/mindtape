// www/utils/demo.typ — the opening demonstration the site root and README.md
// share: one real, closed task quoted as Markdown+YAML and as Typst, then what
// only the Typst side does.
//
// It lives here because the two front doors were hand-copied and `just
// dup-prose` reported six pairs between them, every one a sentence that would
// have drifted silently. The fact it reads, www/generated/readme-task.json, is
// written by www/generate-facts.sh from a task in this repo's own backlog;
// reading it relative to this file resolves under both render paths — tola's
// `--root www` and typlite's `--root .` — so neither page passes a path or
// keeps its own copy of the stem.
//
// The link style is the one thing the two paths cannot share: docs/src/readme.typ
// resolves a route through its manifest-checking `www-link`, while a tola page
// prefixes it with `u()`. So the caller passes that one function, `(route,
// body) -> content`, and nothing else.
#let doc = json("../generated/readme-task.json")

#let demo(link) = [
  Say the same task lived as a Markdown file with YAML frontmatter instead,
  checked by a script. Here is one real, closed task from this repo's own
  backlog, both ways.

  #raw(doc.yaml, lang: "yaml", block: true)

  Nothing above is checked until a script reads the file and does it by hand:
  parses `status_date`, keeps `priority`'s three numbers in bounds, resolves
  `depends_on` against another file on disk, and confirms every `x_goals` key
  is present with a reason.

  The same task as Typst — #raw("truth/tasks/" + doc.stem + ".typ") (trimmed
  here: the file continues with `== Problem`, `== TO-BE`, `== Changes`,
  `== Rollback`, `== Out of scope` and `== Dogfood`):

  #raw(doc.marker, lang: "typ", block: true)

  What only the Typst side does:

  - #strong[Typed values with their own shape.] `status: wip(2026, 9, 14)[note]`,
    `priority: ice(impact: 8, confidence: 0.7, ease: 6)` — validated on every
    command, never parsed as strings.
  - #strong[Typed links.] `depends-on("other.typ")` — resolved and checked on
    every command, feeding `ready`/`blocked` and the graph; a renamed target
    fails `mt check` instead of dangling.
  - #strong[A repo-invented field.] Here `x-goals`, checked by the repo's own
    Typst function named in config, and refused before `mt add` writes:

  ```toml
  [[tasks.rules]]
  check = { typst = "truth/lib.typ", function = "check-goals", field = "x-goals" }
  ```

  The full model is on #link("/concepts/")[Concepts]; every field a task file
  can declare is on #link("/dsl/")[Task DSL].
]
