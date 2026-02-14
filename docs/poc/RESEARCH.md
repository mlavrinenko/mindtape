# PoC Research Findings

## Critical Discovery: No Native Checkbox Syntax

Typst does NOT have built-in `- [ ]` / `- [x]` checklist syntax. In Typst:
- `- item` is a bullet list (`ListElem` containing `ListItem`)
- `[ ]` and `[x]` inside a list item are just text content
- The `ListItem` struct only has a `body: Content` field, no `checked` state

**Implication**: We must parse the checkbox pattern from `ListItem.body.plain_text()`.
Pattern: starts with `[ ] ` (unchecked) or `[x] ` (checked), then task title follows.

## Typst Eval API (v0.14)

### Entry Point
```rust
typst_eval::eval(
    routines: &Routines,         // typst::ROUTINES static
    world: Tracked<dyn World>,   // world.track()
    traced: Tracked<Traced>,     // Traced::default().track()
    sink: TrackedMut<Sink>,      // sink.track_mut()
    route: Tracked<Route>,       // Route::default().track()
    source: &Source,
) -> SourceResult<Module>
```

### Module (eval result)
- `module.scope() -> &Scope` — all `#let` bindings
- `module.content() -> Content` — document content tree

### Content Traversal
```rust
content.traverse(&mut |node: Content| -> ControlFlow<B> {
    if node.is::<ListElem>() { ... }
    if node.is::<HeadingElem>() { ... }
    ControlFlow::Continue(())
});
```

Key methods:
- `content.is::<T>()` — check element type
- `content.to_packed::<T>()` — downcast to typed element
- `content.plain_text()` — extract text
- `content.get_by_name(field)` — get field value

### Element Types
- `ListElem` — container with `children: Vec<Packed<ListItem>>`
- `ListItem` — has `body: Content`
- `HeadingElem` — has `body: Content`, `level`

### Value Enum
```rust
pub enum Value {
    None, Auto, Bool(bool), Int(i64), Float(f64),
    Str(Str), Datetime(Datetime), Content(Content),
    Array(Array), Dict(Dict), Func(Func), Module(Module),
    // ... more variants
}
```

## World Trait Implementation

```rust
pub trait World: Send + Sync {
    fn library(&self) -> &LazyHash<Library>;
    fn book(&self) -> &LazyHash<FontBook>;
    fn main(&self) -> FileId;
    fn source(&self, id: FileId) -> FileResult<Source>;
    fn file(&self, id: FileId) -> FileResult<Bytes>;
    fn font(&self, index: usize) -> Option<Font>;
    fn today(&self, offset: Option<i64>) -> Option<Datetime>;
}
```

### Minimal World (eval-only, no render)
- `library()`: `Library::default()` wrapped in `LazyHash`
- `book()`: Empty `FontBook::new()` (no fonts needed for eval-only)
- `font()`: Return `None` always
- `main()`: `FileId` for the input file
- `source(id)`: Read file from disk, handle `@mind-tape` package
- `file(id)`: Read raw bytes from disk
- `today()`: Return current date

### FileId and VirtualPath
- `FileId::new(package: Option<PackageSpec>, path: VirtualPath)`
- `VirtualPath::new(path)` — root-anchored path
- `vpath.resolve(root) -> Option<PathBuf>` — resolve to real FS path
- `id.join(relative_path)` — resolve relative imports

### Package Resolution for @mind-tape
In `source()` and `file()`, check `id.package()`:
- If `namespace == "mind-tape"`, resolve from our lib/ directory
- Otherwise return error (no external packages in PoC)

## lib/prelude.typ Design

The `due()` and `id()` functions need to produce Content that we can identify
during traversal. Best approach: use `metadata()` to emit invisible structured data.

```typ
// lib/prelude.typ
#let due(date) = metadata(("due", date))
#let id(uuid) = metadata(("id", uuid))
```

`metadata()` produces a `MetadataElem` with a `value: Value` field.
We can find these during content traversal with `content.is::<MetadataElem>()`.

The metadata value will be a Typst `Array` containing:
- Index 0: `Str` — the kind ("due", "id")
- Index 1: `Datetime` or `Str` — the actual value

## Cargo Dependencies (actual, working)

```toml
[dependencies]
typst = "0.14"
typst-eval = "0.14"
typst-library = "0.14"
typst-syntax = "0.14"
comemo = "0.5"   # must be 0.5 to match typst 0.14's comemo version
```

Note: `typst-kit` is NOT needed for eval-only (it's for fonts/packages).
`clap` is NOT needed — manual arg parsing is simpler for the `-N` shorthand.

## Content Tree Structure (ACTUAL from debugging)

**Critical finding**: There is NO `ListElem` wrapper in the content tree!
List items appear directly as `ListItem` ("item") nodes in a flat `SequenceElem`.

Given:
```typ
= Piano
- [ ] Finish learning Lilium #due(datetime(year: 2026, month: 5, day: 1))
```

Actual content tree after eval:
```
SequenceElem [
  HeadingElem { body: Text(Piano) },
  ParbreakElem,
  ListItem { body: SequenceElem [
    Text([), SpaceElem, Text(]),   // "[ ]" - checkbox is split into parts!
    SpaceElem,
    Text(Finish learning Lilium),
    SpaceElem,
    MetadataElem { value: ["due", Date(2026-05-01)] },
    SpaceElem,                      // trailing spaces from formatting
  ]},
  SpaceElem,
  ...
]
```

Key observations:
- Checkbox `[ ]` becomes `Text([) + SpaceElem + Text(])` (3 separate nodes)
- Checkbox `[x]` becomes `Text([) + Text(x) + Text(])` (3 separate nodes)
- `plain_text()` on the body gives `"[ ] Finish learning Lilium  "` (with trailing spaces)
- `MetadataElem` is in `typst_library::introspection`, NOT `typst_library::model`
- Must `trim()` the title after stripping the checkbox prefix
- Traverse for `ListItem` directly, NOT `ListElem`

## Extraction Algorithm (corrected)

For each `ListItem` found via `content.traverse()`:
1. Get `body.plain_text()` — e.g. `"[ ] Finish learning Lilium  "`
2. Parse checkbox: `strip_prefix("[x] ")` or `strip_prefix("[ ] ")`
3. **Trim** the remaining text to remove trailing whitespace from SpaceElem nodes
4. Traverse body children with `body.traverse()` for `MetadataElem` nodes
5. For each MetadataElem, value is `Value::Array` — check `slice[0]` for tag string
6. If tag is `"due"`, extract `slice[1]` as `Value::Datetime`

## World::track() Gotcha

The `comemo::Track` trait method `.track()` is on the `dyn World` trait object,
not on concrete types. Must cast: `(world as &dyn World).track()`.

## CLI Design (PoC)

```
mindtape <file.typ> [--due] [-N]
```

- Positional: `.typ` file path
- `--due`: filter to tasks with due dates + sort by due date ascending
- `-N` (e.g. `-2`): limit output to N results

### Output Format
```
- (due YYYY-MM-DD) Task title
```
Tasks without due date (when not filtered by --due):
```
- Task title
```

Default: exclude completed tasks (no `--status=any` flag in PoC).
