#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Fail the dry gate on a jscpd that isn't the one the baseline was minted with",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  links: (
    (
      task: "add-a-benchmark-harness-for-the-load-path.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "tooling", "gating", "stateless"),
  assignee: "agent",
  status: proposed(2026, 11, 25),
)

== AS-IS

The dry gate blames the tree for a tool the shell got wrong.

+ Running `just check` in the main checkout while the bench harness had no commit yet
+ 🔴 #raw(".just/lint.just"): the dry gate reports 161 new clones that the baseline holds, because the shell runs jscpd 5.4.1 while 5.4.3 minted the baseline

```
$ just lint::dry
1366 clones · 9.2% duplication
ERROR: jscpd found 161 new clones not in the baseline (allowed: 0)
error: Recipe `dry` failed on line 62 with exit code 1

$ which jscpd
/nix/store/2b80qmdnw9mbzkknhziiw4660nj39kp6-jscpd-5.4.1/bin/jscpd
$ jscpd --version
jscpd 5.4.1
```

The same tree and the same flags under the pinned 5.4.3:

```
$ /nix/store/8yxphvg10c7598vy6apwy8qrvj6fh16s-jscpd-5.4.3/bin/jscpd --no-tips -f rust -k 50 \
    --ignore-identifiers --ignore-literals --similarity=0.8 --ignore '**/tests/it/main.rs' \
    --baseline .config/jscpd-baseline.json --fail-on-new-clones 0 -r silent crates
Duplications detection: Found 1287 exact clones with 13954(8.68%) duplicated lines in 775 (1 formats) files.
```

`a698ca43` re-minted the baseline on 5.4.3, and its message says the version "drops 161 it found". That is exactly the count the gate reports. The `crates` tree is byte-identical in both runs: `find crates -name '*.rs' | sort | xargs sha256sum | sha256sum` gives `48a3c00b…` either way, and `--print-config` differs only in the usage line each binary prints.

== TO-BE

- 🟢 The dry gate fails naming the version it ran and the version that minted the baseline, before it reports a clone.
- 🟢 A shell on the pinned version passes, and each baseline carries the version that minted it.

```
$ just lint::dry
error: jscpd 5.4.1 is not the 5.4.3 that minted .config/jscpd-baseline.json
hint: re-enter the dev shell: `direnv reload`
```

== Changes

- `.config/jscpd-baseline.json`, `.config/jscpd-repeats-baseline.json`: a `jscpd` key beside `version`, holding the version that minted the fingerprints.
- `.just/lint.just`: `dry` compares `jscpd --version` with the baseline's key before it runs the detector, and `dry-baseline` stamps the running version into the file with `jq` after jscpd writes it.
- `.just/scripts/repeats-gate.sh`: the same check for the repeats baseline, which reads the same binary.
- `CHANGELOG.md`: an entry under `[Unreleased]`.

== Out of scope

- The stale `.direnv` cache in this checkout. `direnv reload` rebuilds it, and the gate only has to name the mismatch.
- Pinning jscpd by store path in the recipe. The dev shell pins it, and a hard-coded path breaks on another machine.

== Dogfood

Every checkout that predates a jscpd bump fails with the two versions, not with 161 clones it cannot explain.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a tool mismatch cannot pass for 161 new clones].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
