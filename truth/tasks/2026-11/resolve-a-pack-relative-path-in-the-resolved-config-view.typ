#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Resolve a pack-relative path in the resolved config view",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  links: (
    (
      task: "show-the-resolved-config-in-mt-config-show.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "cli", "stateless"),
  assignee: "agent",
  status: proposed(2026, 11, 21),
)

== AS-IS

+ Reading `mt config show --resolved` to check that its output parses back as a config
+ 🔴 #raw("crates/mindtape-cli/src/cmd/config.rs"): a path a pack declares resolves against the pack directory, and the resolved view prints it verbatim, so writing the output back re-points it at the project root

A `check.typst`, a collector `query.typst`, a `dicts.path` and a `[[typst.local-packages]] path` declared by a pack all load relative to that pack's directory: `crates/mindtape/src/config/path_resolve.rs`. `mt config show --resolved` prints the declared text, and the round-trip test passes because its fixture declares none of them.

== TO-BE

- 🟢 `mt config show --resolved` prints each pack-declared path as it resolves. The document then reads at the project root.
- 🟢 The round-trip test's fixture declares one pack-relative path. Writing the output back then keeps it.
- 🟢 A project sees the same file in the resolved view and in `mt explain`'s replacement, `essentials.jq`.

```
$ mt config show --resolved | grep typst
check = { typst = "crates/mindtape/src/packs/rules.typ", function = "check" }
```

== Changes

- `crates/mindtape/src/config/`: record each resolved path beside the value, so a serializer can print the absolute or root-relative form.
- `crates/mindtape-cli/src/cmd/config.rs`: emit the resolved form in the `--resolved` view.
- `crates/mindtape-cli/src/cmd/config_resolved_tests.rs`: a fixture whose pack declares one pack-relative path, and a round trip over it.
- `www/content/cli/config.typ`: what the resolved view does with a path a pack declares.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Fixed`.

== Out of scope

- The collected view. It prints the layer text, which is what a config file holds.
- `[params]`. It stays out of both views.

== Dogfood

This repo declares `[[typst.local-packages]] path` in its own config, so a project that copies a resolved view from a pack would break on the first check.

== Goals

+ #Helps(maintainer, to: match-docs-to-code)[the resolved view states the path a project actually loads].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe).
