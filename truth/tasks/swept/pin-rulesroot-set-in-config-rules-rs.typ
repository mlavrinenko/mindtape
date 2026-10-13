#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "pin RulesRoot::set in crates/mindtape/src/config_rules.rs",
  kind: "test",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 9,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  assignee: "agent",
  tags: ("swept", "mutants"),
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
      sound-codebase: helps[a line that decides something gets a test that fails
        when it breaks],
      docs-match-code: neutral,
    ),
  ),
  status: done(
    2026,
    10,
    1,
  )[pinned by \\\`a\_typst\_check\_rule\_resolves\_against\_the\_project\_root\\\`: the follow-up \\\`just mutants --iterate\\\` re-ran this mutant and caught it],
)

== Problem

+ Run `cargo mutants` over the file below.
+ 🔴 #raw("RulesRoot::set") in #raw("crates/mindtape/src/config_rules.rs") survived a mutation at line 103: the suite still passed after #raw("replace RulesRoot::set with ()").

The line is covered, so something executes it, but nothing asserts on what it
decides. The 2026-09-30 full workspace sweep found this; it was filed by hand
because the follow-up `--iterate` run consumed the sweep record.

That sweep ran 4240 mutants across the workspace and left 1 unpinned, with 0 timeouts.

== TO-BE

- 🟢 `just mutants -f` on this file reports the mutant as caught, or
  `.cargo/mutants.toml` declares it uninteresting with a reason.

== Changes

- Pick the remedy, then do it: write the assertion that kills the mutant, or
  declare it uninteresting in `.cargo/mutants.toml` with a reason. The
  assertion has to be one a reader would want anyway.
- If you assert, assert on the behaviour the mutated line decides, not on the
  literal it was replaced with.
- If you exclude, write the reason next to the entry in `.cargo/mutants.toml`.
  A written reason is the only thing that suppresses this finding on the next
  sweep; there is no ignore list anywhere else, deliberately.

== Out of scope

- A test that pins the replaced constant and nothing else: it moves the
  survivor without buying a regression net.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
