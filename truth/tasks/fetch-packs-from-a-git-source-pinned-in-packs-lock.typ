#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fetch packs from a git source pinned in packs.lock",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 3,
  ),
  difficulty: 7,
  tags: ("mt", "gating", "decision"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a team adopts its own packs from one repository, pinned, without a submodule or a copy per project],
      typst-over-markdown: neutral,
      plain-files: hurts[a fetched pack lives outside the repo, though `mt packs install` restores it from the committed lock],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    26,
  )[agreed: \[pack-sources\] in the project layer, a committed packs.lock, a sha256-addressed cache, mt packs install and update, and no fetch in read commands],
  links: parent("share-config-between-projects-through-packs.typ")
    + related(
      "let-a-config-extend-rule-packs.typ",
    )[reverses its Out of scope line on packs fetched over the network],
)

== Problem

+ Keep a team's packs in their own git repository and adopt them in several projects.
+ 🔴 An `extends` entry is a built-in name or a path inside the project, so each project carries the packs as a submodule or a copy, and every `just fork` worktree must initialise the submodule again.
+ 🔴 Nothing records which revision a project adopted, or notices a hand-edited copy.


== TO-BE

```toml
extends = ["acme:commits"]

[pack-sources.acme]
git = "https://github.com/acme/mindtape-packs"
ref = "v1"   # a tag, branch or commit
```

- 🟢 `.mindtape/packs.lock`, written by `mt` and committed, pins each source's commit and the sha256 of its files.
- 🟢 A source is fetched once per machine into `<cache dir>/mindtape/packs/<sha256>/`, shared by every project and worktree, and re-hashed on each load.
- 🟢 Read commands never fetch. `mt packs install` fetches what the lock pins, as `npm ci` does, and `mt packs update` moves sources along their `ref` and prints what changed.
- 🟢 Fetching runs `git`, so private repositories use the user's own credentials.


== Changes

- `truth/tasks/*.typ`: the leaves that build it, linked with `parent` to `share-config-between-projects-through-packs.typ`. This decision changes no file itself.


== Alternatives

- Committed copies in each project: every repo carries every pack, and an edit to a copy goes unnoticed.
- A gitignored `.mindtape/packs/`, like `node_modules/`: every checkout and worktree fetches its own copy.
- Fetch over HTTP: a TLS stack inside `mt`, and a token scheme for private repositories.
- An ancestor `.mindtape/defaults.toml`: no file in the repo names it, so a clone or CI can't restore it.


== Out of scope

- A pack registry, search or signing: a git URL and a pinned hash cover sharing between known projects.
- Fetching in read commands: `mt check` runs in editors and commit hooks, where network access is a surprise.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs. The integration tests fetch from a local git repository.

