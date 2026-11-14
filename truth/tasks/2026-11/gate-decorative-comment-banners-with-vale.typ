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
  status: done(
    2026,
    11,
    14,
  )[banners gated, prose stays on 3.26 until the backfill],
)

== AS-IS

+ Run `just check` with a new `# arm 1 ----------------------------------------------------------------------` in a script.
+ 🔴 It stays green: no arm reads comment text.
+ Run `vale --version`. The repo built 3.26, and nixpkgs-unstable still carries it. The filed note blamed that pin, but this repo builds Vale from source, so 3.27 was buildable already.

```
$ printf '# arm 1 ----------------------------------------------------------------------\n' >> .just/scripts/_gate-drift-lib.sh
$ just check
# green
```


== TO-BE

- 🟢 `just lint::banners` fails on a comment whose text ends in five or more of `- = * _ ~ #`.
- 🟢 The gate reads Rust, Shell and CSS comments through Vale's tree-sitter grammars.
- 🟢 The grammars come from the nix store. The gate then runs offline.
- 🟢 `vale` is 3.27.0 from source. The prose gates keep `vale-prose` 3.26 until a backfill task cleans the wrapped-line alerts.

```
$ printf '# arm 1 ----------------------------------------------------------------------\n' >> .just/scripts/_gate-drift-lib.sh
$ just lint::banners
.just/scripts/_gate-drift-lib.sh:100:9:Sep.Banner:Delete the decorative run; keep any label it frames.
```


== Changes

- `.config/vale.ini`: the `[formats] sh = r` mapping and the `[*.{rs,sh,css}]` section.
- `.config/vale/local/Sep/Banner.yml`: the rule for a trailing decorative run.
- `.config/canibump.toml`: drop the stale nixpkgs 3.27 blocker.
- `.config/outdatty.yaml`: the new Nix sources in the two Vale groups.
- `.just/lint.just`: the `banners` recipe over `git ls-files --cached --others --exclude-standard`.
- `.just/docs.just`: the two prose arms call `vale-prose`.
- `.mmz/config.yaml`: the `lint::banners` rule and its inputs.
- `Justfile`: `lint::banners` in `_check-fast`.
- `.githooks/pre-commit`: task prose calls `vale-prose`.
- `nix/dev/vale.nix`: 3.27.0 on Go 1.27.3.
- `nix/dev/vale-prose.nix`: the 3.26 pin under `bin/vale-prose`.
- `nix/dev/vale-grammars.nix`: the three `grammar-*.zip` assets, unpacked to WebAssembly modules.
- `nix/dev/vale-styles.nix`: copy the `Sep` style.
- `nix/dev/shell.nix`: build both binaries and export `VALE_GRAMMARS_PATH`.
- `nix/dev/flake.lock`: the nixpkgs-tombi bump for Go 1.27.3.


== Alternatives

- A repo-local ast-grep ruleset needs one more binary and a rule per language, where Vale is the prose tool we want anyway.
- linecop: it caps comment lines and carries no comment text.
- `commentment`: it reads Typst comments, but Vale's rule engine and editor integration win for prose.
- A single Vale version: 3.27's sentence reader joins a wrapped line into one sentence, which turns 147 existing prose alerts on. A backfill task cleans them, and then the `vale-prose` pin goes.


== Out of scope

- Typst comments: Vale's Typst reader drops them, and the only banners are in generated `tola.typ`.
- Markdown tables and `---` rules: they are syntax, not decoration.
- Prose rules on code comments: the `lint-code-comments-with-vale` task.
- The 3.27 prose backfill: the 147 alerts need their own task.
- Comments in other languages until the grammar list grows.


== Dogfood

This repo runs `just lint::banners` in `_check-fast`. The tree carries no banners after the strip task, and a new one fails before review.

