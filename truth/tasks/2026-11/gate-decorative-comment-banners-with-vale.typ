#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Gate decorative comment banners with Vale",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling", "gating", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[a comment says what it means without a ruler to scroll past],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 13)[waiting on vale 3.27 in nixpkgs-unstable],
)

== AS-IS

+ Run `just check` with a new `# arm 1 ----------------------------------------------------------------------` in a script.
+ 🔴 It stays green: no arm reads comment text.
+ Run `vale --version`; nixpkgs-unstable has 3.26, and `Grammars` needs 3.27.

```
$ printf '# arm 1 ----------------------------------------------------------------------\n' >> .just/scripts/_gate-drift-lib.sh
$ just check
# green
```


== TO-BE

- 🟢 `just lint::banners` fails on a comment whose text ends in five or more of `- = * _ ~ #`.
- 🟢 The gate reads Rust, Shell and CSS comments through Vale's tree-sitter grammars.
- 🟢 The grammars come from the nix store, so the gate runs offline.

```
$ just lint::banners
crates/foo/tests.rs:12:4: Sep.Banner: Delete the decorative run; keep any label it frames.
```


== Changes

- `.config/vale.ini`: `StylesPath = vale/styles`, `Grammars = bash, rust, css`, `Sep` on `*.{rs,sh,css}`.
- `.config/vale/styles/Sep/Banner.yml`: the `existence` rule for a trailing run of five.
- `nix/dev/vale-grammars.nix`: fetch the three `grammar-*.zip` assets for `vale.version`, unpack the wasm.
- `nix/dev/shell.nix`: take `vale` from `pkgs-tombi`, call the grammar file, export `VALE_GRAMMARS_PATH`.
- `.just/lint.just`: the `banners` recipe over `git ls-files --cached --others --exclude-standard`.
- `.mmz/config.yaml`: the `lint::banners` rule.


== Alternatives

- A repo-local ast-grep ruleset: one more binary and a rule per language, where Vale is the prose tool we want anyway.
- linecop: it caps comment lines and carries no comment text.
- commentment: it reads Typst comments, but Vale's rule engine and editor integration win for prose.


== Out of scope

- Typst comments: Vale's Typst reader drops them, and the only banners are in generated `tola.typ`.
- Markdown tables and `---` rules: they are syntax, not decoration.
- Prose styles: a later task moves the doc prose onto Vale.


== Dogfood

This repo runs `just lint::banners` in `_check-fast`, so a new run fails before review.

