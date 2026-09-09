#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "re-mint the stale CRAP baseline",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("tooling",),
  estimate: (files: 1),
  assignee: "agent",
  status: done(2026, 9, 9),
)

== Summary

`just crap` is red and `just crap-baseline` cannot refresh it. The committed
`crap-baseline.json` was minted at 1a57fad (2026-09-01); 65 commits have
landed since, including whole features whose functions the baseline has never
seen — `mt link` alone contributes `shape_of`, `links_edit`, `link_cmd::run_with`
and five more, every one reported NEW. The gate now names 17 regressions on a
tree where nothing was reviewed as a regression, which is the state that
teaches a reader to skim it.

== Scope

Re-mint the baseline, deliberately, in a commit of its own — the recovery
`docs/contributing/coverage-crap.md` describes under "When the baseline itself
is the thing that is wrong", since `just crap-baseline` depends on `just crap`
and a red gate aborts the mint.

Before minting, carry the two facts that make it a review rather than a
bypass: `cargo crap --lcov ... --threshold 30 --min 30 --fail-above` is green
(nothing is absolutely bad), and each ▲ entry is either explained by a
deliberate complexity landed since 1a57fad or fixed first. The `▲` rows to
account for, from the 2026-09-09 run: `Cli::run` (+1.0), `add::run` (-4.0 but
still flagged), `ask_axis` (+0.4), `shape_of` (NEW, 50.0% covered),
`eval_bool` (+0.9), `links_edit` (NEW), `link_cmd::run_with` (NEW),
`resolve_function` (+2.0), `revalidate` (+6.8, 35.3% covered) and
`Hooks::list`.

`shape_of` and `revalidate` are the two worth looking at rather than
absorbing: both are new-ish and under 55% covered, which is exactly the
"complex and undertested but scoring under 30" case the ratchet exists to
catch and the absolute threshold is blind to.

Not in scope: relaxing the ratchet, or minting past a red arm. The recovery is
manual on purpose.
