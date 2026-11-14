#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Source the Vale rules from Nix",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs", "gating", "stateless", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[one lock pins the prose rules],
      cheap-adoption: helps[a checkout builds the rules with the shell],
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the docs state the build that supplies the rules],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 14),
)

== AS-IS

+ Run `git show --stat a4cb45cc`. It adds 41 rule files under `.config/vale/styles/Prose/`, about 4k lines of upstream YAML.
+ Run `ls .config/vale/styles/Prose`. It holds 40 fetched rules, five local rules, `words.txt` and a notice, and the upstream revision is a comment.
+ 🔴 A rule update is a manual copy, and the repo carries every upstream file in git.

```
$ git show --stat a4cb45cc | tail -1
184 files changed, 4749 insertions(+), 1127 deletions(-)
```

== TO-BE

- 🟢 `nix/dev/flake.nix` pins the four upstream sources, and `nix/dev/flake.lock` records their revisions.
- 🟢 `nix/dev/vale-styles.nix` builds the style, strips each upstream `level`, and applies the four local edits.
- 🟢 `.config/vale/local/` holds only the five repo rules, `words.txt` and the vocabulary. `.config/vale.ini` reads `target/vale/styles`.
- 🟢 `just docs vale-prose` and `just docs vale-tasks` pass with the built style.

```ini
StylesPath = ../target/vale/styles
```

== Changes

- `nix/dev/flake.nix`: the four rule inputs.
- `nix/dev/flake.lock`: their pinned revisions.
- `nix/dev/vale-styles.nix`: the build and the local edits.
- `nix/dev/shell.nix`: link the output at `target/vale/styles`.
- `.config/vale.ini`: point `StylesPath` at the link and drop the copied rules.
- `.config/vale/local/`: the five local rules, `words.txt` and the vocabulary.
- `.config/outdatty.yaml`, `.config/outdatty.lock`: the new sources.
- `.mmz/config.yaml`: the local style scope.
- `docs/src/contributing/documentation.typ`: the build description.
- `.just/scripts/check-doc-prose.sh`: the new path.

== Alternatives

- Keep the copied rules. That keeps every rule reviewable in one diff and pins nothing.
- Use the `valeStyles.google` and `valeStyles.microsoft` packages. The pinned nixpkgs carries older revisions, so the backlog would change.

== Out of scope

- Changes to the rules themselves. The build applies the same four local edits and the same upstream revisions.

== Dogfood

This repo's `just docs vale-prose` and `just docs vale-tasks` read the Nix-built style.
