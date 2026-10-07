#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Package the prelude inside mindtape-cli so the crate can publish",
  kind: "dev",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[`cargo install mindtape-cli` works once the crate reaches crates.io],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
)

== Problem

+ Running \`cargo publish --dry-run --locked --workspace\`, the new release preflight, on the release-pipeline branch
+ 🔴 #raw("crates/mindtape-cli/src/cmd/init/mod.rs"): mindtape-cli's packaged tarball does not compile, because \`include\_str!\` reaches \`lib/prelude.typ\` outside the crate directory, so the next release would fail at \`cargo publish -p mindtape-cli\`

```
error: couldn't read `src/cmd/init/../../../../../lib/prelude.typ`: No such file or directory (os error 2)
  --> src/cmd/init/mod.rs:38:23
error: could not compile `mindtape-cli` (bin "mt") due to 1 previous error
error: failed to verify package tarball
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/init/mod.rs").

== TO-BE

- 🟢 `cargo publish --dry-run --locked --workspace` passes, so `just release-dry-run` can pass.
- 🟢 The prelude `mt init` installs is the same bytes as `lib/prelude.typ`, checked by a gate.


== Changes

- `crates/mindtape-cli/src/cmd/init/mod.rs`: read the prelude from a file inside the crate (or from the `mindtape` crate, which could carry it).
- `crates/mindtape-cli/Cargo.toml` or a build script: keep that copy in step with `lib/prelude.typ`.


== Rollback

- Revert the commit; the crate stays unpublishable as before.


== Out of scope

- Other `include_str!`/`include_bytes!` that reach outside a crate: grep for them in the same pass, but each crate's fix is its own.


== Dogfood

This repo's next `just release-dry-run` passes the packaging step.

