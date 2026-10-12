#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Extract the repeated short snippets and lock each helper with clippy",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (src-files: 12),
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
      sound-codebase: helps[a snippet too short for any clone gate still lives in one place, and clippy refuses copy number eight],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
)

== Problem

+ Run `rg -l -U 'util::open\(\)\?;\n\s*util::warn_errors' crates`.
+ 🔴 Seven commands (`go`, `link`, `mv`, `recur`, `review`, `rm`, `set`) open with the same two lines: `util::open()?` and `util::warn_errors(&project.outcome)`.
+ 🔴 Nine files repeat one `match`: `suggest::nearest(name, &known)`, then `Some(near) => diag.hint(Hint::did_you_mean(near))` and `None => diag`.
+ Each snippet sits under jscpd's 50-token floor, so no clone gate sees it, and nothing stops the next copy.


== TO-BE

- 🟢 Each snippet is one helper, such as `util::open_warned()` and a `Diag` method that adds the nearest known name as a did-you-mean hint.
- 🟢 `clippy.toml`'s `disallowed-methods` bans the raw call outside its helper, the way it bans `std::fs::write` for `write_atomic`. A new copy fails clippy, not a review.


== Changes

- `crates/mindtape-cli/src/util.rs`: `open_warned`, the one caller of `open` and `warn_errors` left.
- `crates/mindtape-cli/src/diag/hint.rs`: the did-you-mean helper.
- `crates/mindtape-cli/src/cmd/**`: the callers use the helpers.
- `clippy.toml`: one `disallowed-methods` entry per helper, with the helper as its reason.


== Rollback

- Revert the commit. The helpers inline back and the clippy entries go.


== Out of scope

- Longer clones, which the jscpd baseline lists and its own task works down.


== Dogfood

This repo's clippy gate refuses a re-inlined copy.
