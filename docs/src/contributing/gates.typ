// docs/src/contributing/gates.typ — source for docs/contributing/gates.md.
// Rendered by docs/generate-md.sh via typlite, same as every other
// docs/src/*.typ source; see docs/src/lib.typ for the render path's
// constraints.
//
// Started as a content-preserving port of the hand-written gates.md — the
// MINDTAPE_GATE_SCOPE escape hatch and the commit-msg trailer it requires.
// The three sections above that came out of the Justfile under
// truth/tasks/relocate-justfile-commentary-to-scripts-and-docs.typ: the
// arguments that used to sit above `check`, `mmz`, `check-examples` and the
// JUST_FMT_PATHS assignment. Nothing here is derived — a memoization
// contract, a fixture that rotted, and a formatter behaviour verified against
// one just version are process and measurement, not values any of
// www/generate-facts.sh's artifacts (crate-map, dsl-fields, gates,
// linecop-caps, sweep-flags) carries.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/gates.md",
)) <mindtape-md>

#doc-title[Quality gates]

Deep dive behind CONTRIBUTING.md's Quality Gates section: how #just.check()
runs, what keeps the gates themselves honest, and the one sanctioned way to
commit without a fully fresh run.

= How check runs

#just.check() runs every gate recipe in parallel — formatting, clippy, tests,
unused deps, file size, drift, the doc gates and the fixture locks.
CONTRIBUTING.md's table lists them all, derived from the recipe's own
dependencies rather than retyped.

Each arm is #just.mmz("<subgate>"): mmz wraps the sub-gate, so a no-op re-run
skips every arm whose declared inputs are unchanged, and `mt flip` into a
terminal status asserts them all fresh via `mmz --is-fresh --tag gate` without
running any. Per-rule scopes and the `gate` tag live in `.mmz/config.yaml`.

The double hop makes mmz the cache identity — the rule name is
#raw("just <recipe>") in `.mmz/config.yaml` — uniform across every sub-gate.
#just.test("<filter>"), a developer query with no mmz around it, bypasses that
recipe and stays plain `cargo test` with `ARGS`, so the cached record reflects
only the bare, full-suite invocation #just.check() runs, never a filtered
subset.

There is no `chronic` here: a cache hit prints mmz's own `on_hit` note to
stderr (quiet by construction), a miss streams the underlying recipe's stdout
and stderr live, and a failure propagates its non-zero exit out of `parallel`
unchanged.

The arm that measures a build artifact is wrapped like the rest, and it is the
case that shows what the wrapping needs. #just.check-crap() reads
`target/coverage/lcov.info`, a path git ignores and no source scope covers, so
it is named by an `lcov` scope of its own, declared `gitignore: false` — the
per-scope opt-out that keeps a scope over ignored paths from resolving empty
under the manifest-wide filter (when that key is load-bearing for a given glob
and when it is a declaration is measured in `.mmz/config.yaml`, beside the
scope). Absent and present are therefore different input states: the arm skips
loudly while no lcov exists and goes stale the moment #just.cover() writes
one, instead of caching "I could not measure" as a pass that outlives the
artifact appearing. That arm ran unwrapped for as long as nothing named the
artifact it reads; nothing else ever did.

= Why the script scopes are enumerated

`.mmz/config.yaml` names one scope per file under `.just/scripts/`, listed
path by path. It used to name them with a single `.just/scripts/**` glob inside
`gates-meta`, the scope every rule shares, so editing one recipe body re-ran
every rule — clippy, the full test suite, the docs build — for a change to a
gate that never opens the file.

The glob was not an oversight, and the enumeration gives up something real. The
directory is recipe bodies by construction, so a body extracted out of the
Justfile was covered by arriving, rather than by someone remembering to wire it
up. An enumeration only covers what somebody listed, and an unlisted script is
invisible: the rule that runs it serves a stale hit against a body it never
hashed.

That is paid for by #just.script-scope-check(), not by care. It set-differences
the files on disk against the literal paths the scopes name, and separately
reads each rule's recipe body back out of the just dump to require that a rule
invoking a script declares a scope covering it — the second arm because a
script listed in a scope no rule reads passes the first while the stale hit
survives. A wildcard never counts as naming a script, so answering a failure by
putting the glob back satisfies nothing.

Three rules keep a directory-wide glob, and their subject is the directory
rather than one body in it: #just.check-file-size() (linecop scans every file
under it), #just.outdatty-check() (every script is a `dev-docs` source) and
#just.script-scope-check() itself, which has to notice a script ARRIVING and so
cannot take an enumeration as its input.

== What the split did and did not buy

Measured on the maintainer's machine with a hot `target/`, against the
twenty-one rules in the manifest (the twenty #just.check() arms plus
#just.cover()). Each figure is one #just.check() run after the edit named,
with the `dev-docs` group re-recorded the way any such edit requires:

- A no-op re-run skips 20 of 20 arms: 0.2s wall, 0.4s CPU. The floor.
- A one-script edit re-runs 4 of 20 — the rule whose body it is, plus the three
  directory-wide ones above: 0.8s wall, 1.4s CPU. Before the split it re-ran
  all of them, which is the Justfile figure below.
- A Justfile-only edit re-runs 20 of 20: 39s wall, 85s CPU. Unchanged by this
  work, and 50x the one-script case.

So the split moved script edits from the Justfile figure to the no-op figure,
and moved the Justfile itself not at all. Every rule names `gates-meta` and the
Justfile is one file, so hashing part of it needs mmz's command-driven input
hashes; that is filed separately and deliberately not adopted here, where a
wrong probe would make a gate lie. The scripts were the half that split with no
new mmz feature at all, and this measurement is what the adoption decision
waits on.

One caveat the averages hide: what a one-script edit costs after the split is
the edited arm's own runtime, not the fan-out. Editing
`.just/scripts/check-doc-recipes.sh` still takes 38s, because that arm alone
takes 38s — it is also the arm that sets the whole gate's parallel wall-clock
floor. Fan-out is what the scopes control; a slow arm is its own problem.

= Why the example projects are gated

#just.check-examples() validates every example project against itself, so a
fixture some other tool runs `mt` against can never rot: `learn-piano` feeds
the docs generator and the skill's triage evals, `minimal` feeds the skill's
trap evals.

Gating them is what earns the right to depend on them. The eval fixture that
preceded these sat in untracked scratch and silently outlived two config
migrations — the retired `[ls] default-filter`, the `depends-on:` → `links:`
move — so no eval could have run against it, and nothing said so.

= Why the module files are formatted one by one

A bare `--fmt` pass formats the root `Justfile` only and silently skips every
`mod` file. That is verified rather than assumed: on just 1.46, against a
synthetic module, `--fmt` left the mis-formatted module byte-identical and
`--fmt --check` reported only the root file's diff.

It had already cost something. The docs module sat outside #just.fmt() and
#just.fmt-check() from the day it was split out
(`truth/tasks/split-the-justfile-into-modules.typ`) and had drifted by one
blank line before anything looked at it
(`truth/tasks/gate-formatting-of-just-module-files.typ`). So each module file
gets its own `--justfile <file> --fmt` pass, over one `JUST_FMT_PATHS` list
shared by both recipes; a module file that moves without its entry moving
drops back out of both and reopens exactly that hole.

= What the pre-commit hook asserts

The hook installed by `nix develop` asserts the gate is fresh
(`mmz --is-fresh --tag gate`) and checks task files. A commit whose gate run is
stale is refused.

= The escape hatch

`MINDTAPE_GATE_SCOPE=<reason> git commit` skips only the freshness assertion.
Everything else still runs: `typstyle --check truth/tasks` and `mt check` are
unaffected.

It is not a silent skip. `.githooks/commit-msg` refuses the commit unless the
message carries a matching `Gate-Scope: <reason>` trailer, so a scoped commit
never reads like a fully gated one in `git log`. The record of what was not
verified travels with the commit that did not verify it.

= When it is legitimate

One case: a delegated worktree whose full aggregate runs at integration. The
gate is not being skipped, it is being deferred to a run that provably covers
the merged result — which is the only run that proves the integration anyway.
A branch that passes its gate in isolation can still break the integrated
tree.

It is never legitimate to dodge a gate that is actually red. If the gate
fails, the commit is wrong, and a trailer saying so does not make it right.
