#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep declaration lines out of the clone gate",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "tooling", "dry"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a gate fails only on a clone someone can extract],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ"),
)

== Problem

+ Adding a pack and its test module, as five agents did today for the packs milestone
+ 🔴 #raw(".just/scripts/lint-dry.sh"): `just lint dry` reads a run of `mod`, `#[path] mod tests_*` and `include_str!` registry lines as a renamed clone of itself, so each new module or pack fails the gate until the baseline is re-minted. A declaration list has no helper to extract.

```
crates/mindtape/src/config/pack_registry.rs:23: renamed clone of crates/mindtape/src/config/pack_registry.rs:24
crates/mindtape/src/config/packs.rs:384: renamed clone of crates/mindtape/src/config/packs.rs:388
```
One new `EMBEDDED_PACKS` entry and one new `mod tests_scratch;` caused that. Commits `3d46f7d9`, `613f8856`, `e677a9b8` and `0be3eac0` each re-minted the baseline for it.

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/lint-dry.sh").

== TO-BE

- 🟢 Adding a `mod` line, a `#[path] mod tests_*` line or an `include_str!` registry entry leaves `just lint dry` green with no baseline change.
- 🟢 A duplicated function block still fails `just lint dry`.

== Changes

- `.just/lint.just` or `.just/scripts/lint-dry.sh`: stop declaration lines from forming a clone, then re-mint the baseline once so it drops the declaration-only entries.
- `docs/contributing/gates.md` or the script header: say what the gate skips.

== Rollback

- Revert the gate change and the baseline.

== Out of scope

- Clones of real code in the baseline. Each is a real duplicate with its own fix.

== Dogfood

The next module or pack this repo adds lands without a baseline re-mint.
