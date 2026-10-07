<!-- Generated from docs/src/contributing/generated-docs.typ by `just docs md`. Do not edit; edit the source. -->

# Generating the root Markdown from Typst

How `docs/src/*.typ` becomes README.md, AGENTS.md, CONTRIBUTING.md, `skills/mindtape/SKILL.md` and `docs/contributing/*.md`, how you add a source, and what typlite can’t render.

### Why the sources are Typst

The root docs repeat facts that have a machine-readable source: the crate map, the DSL fields, the gate table and the linecop caps, all written to `www/generated/*.json` by `www/generate-facts.sh`. A Markdown file can’t read JSON, and a Typst source can. So each fact is stated once, and every doc reads it from the same file.

`docs/generate-md.sh` runs `mmz just docs::gen` before it renders, so a source never depends on another recipe having run first. On a changed tree that costs a debug build of `mt`, because `www/generate.sh` captures real `mt` output. `www/generate-facts.sh` alone isn’t enough, because `www/generate-site-pages.sh` compiles every site page to build the page manifest, and the pages read those captures. Inside `just check` the four gates that read `www/generated/` share one run: the first to take the lock regenerates, and mmz finds the record fresh for the rest. Each source is rendered with typlite, from nixpkgs’ `tinymist` package.

The site pages under `www/content/` are a separate render path. They open with `#show: page.with(...)`, import `@tola` and target HTML. A `docs/src` source has neither, so typlite can render it alone. Why the site always rebuilds clean, and how typlite renders each site page’s Markdown copy, is in [docs/contributing/docs-site-build.md](docs/contributing/docs-site-build.md).

### Adding a source

To add a source, put a `<mindtape-md>` metadata block in it naming its output path, relative to the repo root:

```typ
#metadata((
  output: "AGENTS.md",
)) <mindtape-md>
```

`docs/generate-md.sh` globs `docs/src/**/*.typ` and skips any file without that block, so a helper module such as `docs/src/lib.typ` needs no exclude list. For each source it reads `output`, runs typlite and writes the result there. Adding a source changes no other file.

`skills/mindtape/SKILL.md` needs YAML frontmatter, which typlite can’t emit, so its block carries a `frontmatter` dict beside `output`:

```typ
#metadata((
  output: "skills/mindtape/SKILL.md",
  frontmatter: (
    name: "mindtape",
    description: "...",
  ),
)) <mindtape-md>
```

The generator writes that frontmatter as a `---` block, before the banner that opens every generated file. The banner is an HTML comment naming the source, so GitHub doesn’t show it:

```
<!-- Generated from docs/src/agents.typ by `just docs md`. Do not edit; edit the source. -->
```

### The freshness gate renders into a temp directory

`just docs md-check` renders every source into a fresh temp directory and diffs each file against the committed copy. If it rendered in place and then read `git status`, it would overwrite a hand-edited or stale committed file before the diff ran, and the gate would pass. A temp render leaves the committed file as it is, so a wrong file fails by name.

`just docs md-check` also lints each render outside fenced code. It fails naming file and line on a `|` inside a code span in a table row, a backtick with no closing backtick on its line, an escaped backtick, and `-\>` outside a code span. It also scans `docs/src` for a `;` right after a `#link(...)[...]`. Each is a trap listed below.

### What typlite can and can’t do

Checked against the tinymist this flake pins and the typst 0.14.2 inside it. That typst is separate from the `typst` and `typst-eval` crates mindtape links (0.15.0, in `www/generated/crate-map.json`), so recheck this table when a nixpkgs bump moves tinymist.

| Construct | Result |
| --- | --- |
| `=` / `==` heading | Shifts down one level: `##` / `###`. `set heading(offset: -1)` errors with “number must be at least zero”. Use `doc-title` (`docs/src/lib.typ`) for the one `#` a document needs |
| `set document(...)` | Errors, because typlite wraps the input in its own container |
| `#footnote[...]` | Fails inside typlite’s own `md-link` with “expected string, found location”. Footnotes are unusable here |
| `#html.elem("p"/"div"/"span")` | Dropped with its attributes, with no error |
| `#html.elem` with `h1`..`h6`, `img`, `br`, `center`, `blockquote` | Kept as literal HTML tags, attributes included |
| `#link("a/b.md")[x]` | Becomes `[x](a/b.md)`, or a raw `<a href>` inside an `html.elem` |
| `json(...)`, `raw(read(...))` | Both work, so a source can read `www/generated/*.json` or another file |
| `#metadata(v) <label>` | Emits nothing into the Markdown. `typst query` reads it back, which is how `<mindtape-md>` works |
| `\@foo` | Literal `@foo`. An unescaped `@foo` can be read as a citation |
| `->` | Escaped as `-\>`. Use backticks or `→` |
| `"straight quotes"`, an apostrophe | Turned into curly quotes outside a raw span |
| Tables, ordered and nested lists, fenced code, blockquotes, definition lists, `**bold**`, `_em_` | Render correctly |
| A raw span containing a literal backtick | Emitted with a single backtick as the delimiter, so the span breaks. Split the text into two spans instead |
| Bare URL in text | Linked as `[url](url)`, by Typst’s own markup |

A footnote fails with a message about typlite’s internals, not your source. A dropped `<p>`, `<div>` or `<span>` leaves the content missing with no message at all. So before you use a new construct in a source, run `typlite --root . <src> /tmp/out.md` and read the output.

### Authoring traps

- Typst’s `*x*` is bold. Markdown’s single-asterisk italic is Typst’s `_x_`, which typlite renders back as `_x_`.
- A wrapped line that ends in a hyphen joins the next line with a space, as `wrong- edited`. Keep a hyphenated word on one line.
- A raw span that wraps across a source line keeps the newline. In a table row that ends the row inside a cell. Wrap the prose around a code span, never through one.
- A `;`, or any character Typst can read as continuing an expression, right after the closing `]` of a `#link(...)[...]` disappears. Put a space before it or escape it with a backslash.
- A `|` inside a code span in a table row splits the row on GitHub. Write `\|` there, which GitHub shows as a pipe.
- A string interpolated as text gets its backticks escaped, so GitHub shows backticks instead of a code span. Interpolate each span through `raw`.
- `doc-title` with a raw span inside renders the span as `<code>`, so the heading demotion in `docs/generate-md.sh` skips it and the title stays a raw `<h1>`. For a literal backtick in a title, interpolate the character from a string.
- A wrapped line that starts with a number and a period is a list item to Typst. The paragraph splits around a stray list marker with no error. Rewrap so no line starts with the number, or escape the period (`15\.`).

typlite emits one line per paragraph, so a generated file’s line count measures paragraphs, list items and table rows. `.linecop.yaml` caps each `docs/src` source, and also caps the rendered CONTRIBUTING.md, which every agent session loads.
