#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Show the release version in the docs site chrome",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("www", "docs", "stateless", "no-repro"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the site chrome shows the release version the binary reports],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 25),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`.
+ Open an article page, such as `/mindtape/cli/`.
+ 🔴 The sidebar footer reads `v0.4.0` and the rail reads `Version 0.4.0`, while `mt --version` prints `mt 0.6.0`.


== Environment

- The docs site at `8a14e3b9` on `docs-redesign`, built with tola 0.7.1 on Linux.


== TO-BE

- 🟢 The sidebar footer reads `v0.6.0` and the article rail reads `Version 0.6.0`, both from `[workspace.package]`, the version `mt --version` prints.
- 🟢 `www/post-build.sh` fails the docs build when either slot differs from that version.

```
<span>v0.6.0</span><span>MIT</span>
<span>Version 0.6.0</span>
```


== Changes

- `www/utils/site.typ`: `mt-version` reads `workspace-package.version` from the generated crate map.
- `www/templates/layout.typ`: the sidebar footer and the article rail render `mt-version` instead of the prelude's `pkg-version`.
- `www/post-build.sh`: the regression arm fails the build when either slot differs from `[workspace.package]`.


== Out of scope

- The prelude version in prose, such as `@local/mindtape:0.4.0`. It references the package version task files import.
- A link from the chrome version to the release notes or the changelog. The chrome only shows the version.


== Dogfood

Every docs build runs the regression arm: the Pages workflow runs `just docs::check`, and both `docs::check` and `docs::build` run `www/post-build.sh`.

