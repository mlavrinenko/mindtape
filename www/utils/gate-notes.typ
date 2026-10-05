// What breaks without each `just check` gate, one clause per gate, keyed by
// gate name. docs/src/contributing.typ renders the clause beside the gate's
// name in the gate table. How a gate works belongs in its recipe comment or
// script header. `just check-doc-facts` fails naming any gate in gates.json
// with no entry here.
//
// A recipe mention goes through `#just.<name>()`, because
// `just check-doc-recipes` fails on a hand-typed one. That import makes this
// file a reader of www/generated/, which www/generate.sh deletes and
// recreates. So any program that evaluates this file must hold the
// target/www-generated.lock flock for its whole run, as check-doc-facts does.
//
// The import path is relative because two Typst roots read this file: the
// repo (docs/src/contributing.typ) and www/ (the docs site).
#import "../generated/just.typ" as just

#let gate-notes = (
  "fmt-check": [a file that #just.fmt() would rewrite reaches review],
  "clippy": [code that breaks a workspace lint lands],
  "test": [a change that breaks a test lands],
  "machete": [an unused dependency stays in a `Cargo.toml`],
  "check-file-size": [a file grows past its cap in `.linecop.yaml`],
  "check-dry": [a copied Rust block stays where a shared helper belongs],
  "check-secrets": [a credential lands in the tree],
  "outdatty-check": [a source changes and the docs that restate it go unreviewed],
  "check-examples": [an example project the docs and evals read stops passing `mt check`],
  "check-doc-coverage": [an `mt` subcommand ships with no CLI doc entry],
  "check-doc-facts": [a derived crate, field, framework, gate, sweep flag, filter function or completable argument ships with no doc entry],
  "check-doc-recipes": [a doc names a recipe by hand, and a rename leaves it stale],
  "check-doc-spans": [a doc code span names an `mt` command, flag or config key that does not exist],
  "check-doc-prose": [a doc source, a config comment or a schema description tells history or shouts],
  "check-doc-blocks": [a doc shows a hand-typed code block that nothing validates],
  "docs::check": [the docs site fails to build or links to a missing page],
  "docs::md-check": [a committed Markdown file differs from its source or renders broken on GitHub],
  "fmt-fixture-check": [a formatter check passes a misformatted file],
  "arg-pattern-check": [a recipe argument pattern accepts bad input or refuses its own default],
  "positional-args-check": [a passthrough recipe splits an argument that contains a space],
  "script-attr-check": [a recipe keeps running after a command in a pipeline fails],
  "size-fixture-check": [the size gate stops scanning `.just/scripts/` and still reports a clean tree],
  "script-scope-check": [a gate script changes and its arm reports a cached pass],
  "site-pages-fixture-check": [the docs manifest gate stops catching a duplicate route, a page without `<page-meta>`, or a page missing from the nav],
  "check-crap": [a CRAP failure that a fresh local lcov already shows first appears in CI],
  "crap-baseline-check": [the CRAP baseline carries machine-local paths or scores nothing],
  "check-gate-drift": [a TOML file goes unformatted, or a gate build relinks `target/debug/mt` under running tests],
)
