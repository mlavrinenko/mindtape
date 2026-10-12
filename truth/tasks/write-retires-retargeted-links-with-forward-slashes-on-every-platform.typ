#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write retire's retargeted links with forward slashes on every platform",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a link written on Windows reads the same on Linux],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ Moving `mt link` targets onto the shared resolver, which writes each target `/`-separated
+ 🔴 #raw("crates/mindtape-cli/src/cmd/rm/gitless/mod.rs"): `mt rm`'s inbound retarget writes `relative_target(...).display()`, which prints `\` between folders on Windows

```
let rel = mindtape::relative_target(from, dest);
edits.push(typst_edit::Edit::new(
    target.range.clone(),
    format!("\"{}\"", rel.display()),
));
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/rm/gitless/mod.rs").

== TO-BE

- 🟢 A retargeted link reads `../.archive/gone.typ` on every platform, as `mt link` writes it.


== Changes

- `crates/mindtape-cli/src/cmd/rm/gitless/mod.rs`: join the components with `/`, the way `Refs::target` does, or share one helper with it.


== Rollback

- Revert the line.


== Out of scope

- `mt link`, which already writes `/`.


== Dogfood

Not applicable, since this repo retires on Linux.
