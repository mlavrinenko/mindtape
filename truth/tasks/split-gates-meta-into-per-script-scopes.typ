#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "split gates-meta into per-script scopes",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling", "gating"),
  estimate: (files: 2),
  links: related(
    "give-check-crap-an-lcov-input-scope-and-wrap-it-in-mmz.typ",
  )[the other mmz-scope change to this manifest],
  status: wip(2026, 8, 16),
)

== Summary

`gates-meta` is one scope shared by all eighteen rules, and it carries
`.just/scripts/**` as a glob. So editing `check-crap.sh` busts `clippy`,
`test`, `docs::check` and fifteen others that never read it. Split the script
glob into per-script scopes, each referenced only by the rule whose body it is.

== Why it is a glob today

Deliberately, and the reasoning still holds for what it was written against:
the directory is recipe bodies by construction, so a body extracted from the
Justfile is covered by arriving rather than by someone remembering to add it.
Per-script scopes give that up, and the price must be paid on purpose: a new
script that nobody wires into a scope is invisible, and the rule serves a stale
hit against a body it never hashed.

Pay it with a check, not with care. `.just/scripts/**` files and the scopes
naming them are both enumerable, so a set difference is a few lines in the same
shape as `check-doc-recipes` — an unreferenced script fails by name. Without
that check this split is a net loss and should not land.

== Why bother

Every Justfile or script edit currently re-runs the whole gate, test suite
included, for a change to one recipe. The Justfile itself stays shared (it is
one file, and hashing part of a file needs mmz's probe work, which is filed
separately and deliberately not started yet). The scripts are the half that
splits with no new mmz feature at all.

== Scope

- `.mmz/config.yaml`: one scope per script under `.just/scripts/`, each
  referenced by its own rule; `gates-meta` keeps `Justfile`, `.just/docs.just`,
  `flake.nix`, `flake.lock`. Keep the comment discipline the file already has,
  and record why the glob became an enumeration.
- A gate arm asserting every `.just/scripts/*.sh` is named by some scope.
  Wire it into `check` the way the other self-tests are, with a `chronic`-free
  failure that names the unreferenced script.
- `Justfile` and `docs/contributing/gates.md`: the new arm, its group tag, and
  the `dev-docs` outdatty reconcile the Justfile edit forces.
- `www/utils/gate-notes.typ`: an entry for the new arm, like every other.

== Measure before adopting probes here

The point of this task is also to find out how much churn is left once the
scripts are out of the shared scope. Record what a Justfile-only edit still
costs afterwards. That number decides whether THIS repo should move its
Justfile rules onto mmz's command-driven input hashes
(`tasks/mmz-command-driven-input-hashes.typ` in the mmz repo) — the feature
exists either way; what waits on the number is adopting it here, where a wrong
probe could make a gate lie.
