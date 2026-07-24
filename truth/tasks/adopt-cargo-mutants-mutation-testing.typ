#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "adopt cargo-mutants mutation testing",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("tooling", "tests"),
  status: done(
    2026,
    7,
    21,
  )[every file swept clean, the 4 parked files resolved, 6 exclusions dropped as killable; re-sweep 542 mutants → 522 caught / 17 unviable / 0 MISSED],
)

== Summary

Add mutation testing (cargo-mutants) as an on-demand quality signal for the test
suite. Line coverage and the CRAP gate prove a line ran; a mutant proves a test
would fail if the line's behaviour broke. Surviving (`MISSED`) mutants are the
assertion gaps that a refactor could silently walk through. Use it to vet the
modules touched by a refactor before starting.

== Why

967 tests, all green, well structured (sibling `_tests.rs`, real `assert_cmd`
e2e, CQRS read-no-write invariant enforced). Strong, but count and coverage do
not measure whether assertions pin behaviour. A scoped trial on `numeric.rs`
(85 mutants) scored 66 caught / 6 missed / 9 unviable / 4 timeout; the misses
were real edge-branch gaps, not noise.

== Operational model

A dev-aid loop, not a gate — kept out of `just check`. `just mutants` runs one
bounded step: the next suitable file whose source or `_tests.rs` sibling changed
since it was last swept clean, then reports how many remain. Repeat across
sessions until every file is clean.

- Engine: `cargo mutants --iterate`, which skips mutants caught in prior runs so
  re-runs stay cheap. It is a heuristic (matches file/line/col/desc), so a final
  result is only trusted after a run without `--iterate`.
- File set: auto-discovered via `cargo mutants --list-files` — no hand-list to
  drift from `./src`. "Suitable" is a predicate, set by `.cargo/mutants.toml`
  excludes, not an enumeration.
- Next-file + remaining count: a git-ignored `.mutants-ledger` maps each
  swept-clean file to the git-blob hash of it plus its `_tests.rs` sibling
  (tight per-file invalidation — editing either re-stales just that file). No
  mmz: cargo-mutants owns invalidation.
- Not every survivor is a bug: `numeric.rs:256` (`<` vs `<=` after an
  exact-match short-circuit) is equivalent — 100% is not the target.

Why not mmz: mmz gives file-level content-hash skip, but a per-file rule per
target hand-listed in `.mmz/config.yaml` drifts from the tree. `--iterate` plus
`--list-files` covers the incremental need natively with zero per-file wiring.
The general mmz gap (glob-fanned per-file rules) is filed in mmz's own backlog
(`per-file-command-cache`), which now runs on MindTape.

== Done in this pass

- `.cargo/mutants.toml`: excludes `**/main.rs` (mirrors the `just crap`
  exclusion) and the `numeric.rs` `<`→`<=` equivalent mutant via `exclude_re`;
  runs tests with `--features rrule` (matches `just check`); `minimum_test_timeout
  = 60`. Tracked despite `.gitignore` ignoring the rest of `.cargo/`.
- `just mutants` rewritten into the `--iterate` + ledger loop above; `-f <file>`
  and arbitrary passthrough retained.
- `.mutants-ledger` and the existing `mutants.out` git-ignored.
- `cargo-mutants` wired into `flake.nix` devShell (nixpkgs unstable 26.2.0);
  `just mutants` resolves project-locally, no global install.
- Documented in CONTRIBUTING.md (Testing); not a `just check` gate.
- Earlier: closed the 4 real gaps the trial found in `numeric.rs` (band `min >
  max`, band `min == max` boundary, anchor out-of-range, point-below-lowest
  derivation); only survivor is the excluded equivalent at `numeric.rs:256`.

== CI story

No workspace-wide nightly sweep (multi-hour, low ROI, CLI files carry
`unviable`/timeout noise). The loop is the primary discipline. If CI is ever
added, a per-PR `cargo mutants --in-diff` advisory (non-blocking) check is the
candidate — fast, scoped to touched lines — promoted to a gate only once the
equivalent-mutant floor in `.cargo/mutants.toml` is stable. CRAP stays
complementary: `just crap` is the cheap proxy for *where* a pass pays off;
mutants proves whether tests there actually pin behaviour.

== Burn-down complete

Every file `cargo mutants --list-files` reports has been swept clean: 0 surviving
(`MISSED`) mutants across the whole tree. The `exclude_re` catalogue in
`.cargo/mutants.toml` peaked at 16 provably-equivalent entries and is now back
down to 10, each paired with its unkillable reason.

Final full sweeps (no `--iterate`, so trustworthy): `mindtape` 308 mutants →
262 caught / 45 unviable / 0 missed; `mindtape-cli` (bar the parked files) all
caught or unviable / 0 missed. The last real gap closed in that pass was
`task_edit::verify_written` (3 survivors — whole-body `Ok(())` and both `==`→`!=`
flips — killed by `task_edit_tests.rs`).

Re-sweep after the four parked files were resolved and 6 exclusions dropped, over
every file this pass touched (`date.rs`, `config_views.rs`, `naming.rs`,
`recur.rs`, `recur_dates.rs`, `render/{color,table,field}.rs`,
`ls_interactive/mod.rs`, `add.rs`, `import.rs`, `cli/recur.rs`): 542 mutants →
522 caught / 17 unviable / 0 missed, plus 3 `TIMEOUT`s re-classified at `-j1`.

== Operational finding: a TIMEOUT is not a verdict, in either direction

A `-j6` sweep on the CLI subprocess-heavy files produced 9 `TIMEOUT` verdicts with
build times inflated 5–8x (26s → 200s+). Every one was a false positive: re-run
single-threaded / `-j2` with an extended `--timeout`, all 9 came back `caught`
(and `select`'s was hand-confirmed caught in 0.11s).

The follow-up sweep sharpened this. Of 3 fresh `TIMEOUT`s in `add.rs`, re-running
at `-j1` resolved one to `caught` (contention, as before) and one to `MISSED` — a
real gap the timeout had been hiding: `add`'s `stdin && stdout` TTY check, the
same shape as `ls --interactive`'s `guard`, now pinned by a pty case in
`tests/tty_e2e.rs`. So a `TIMEOUT` is never a result: it can mask either a pass or
a genuine survivor. Keep final-verdict sweeps at `-j2` or lower with an extended
`--timeout`; reserve `-j6` for a fast first pass that only flags candidates.

== The four parked files, resolved

`mutants-needs-human.txt` (`config_views.rs`, `date.rs`, `naming.rs`,
`recur_dates.rs`) held 5 `MISSED` + 4 `TIMEOUT` survivors. Every one turned out
to be one of three shapes, and only one was a plain assertion gap:

- Real gap (1): `date.rs` `today_epoch_days`, `secs / 86_400` → `%`. The clock
  fallback had no test at all — the only pin was via the `MINDTAPE_NOW` override
  path. Killed by asserting the returned day brackets the system clock
  (`days * 86_400 <= now < (days + 1) * 86_400`).
- Dead code (3): `epoch_days_to_ymd`'s negative-`shifted` branch (input is
  `u64`, so `shifted >= 719_468` always) and the `#[cfg(not(feature =
  "rrule"))]` twin of `advance_rrule`. Both deleted rather than excluded — see
  below.
- Unbounded loops (4, all `TIMEOUT`): `naming.rs` `disambiguate` and
  `recur_dates.rs`'s due-anchor catch-up. Both bounded — see below.
- Provably equivalent (1): `config_views.rs` `dfs`, `delete match arm DONE`.
  The black-node check is pure memoisation: a node reaches `DONE` only after its
  whole subtree came back cycle-free, so re-walking it can only return `None`
  again. Deleting the arm costs exponential time, never correctness.

== Three lessons that changed the code, not the exclude list

+ A `TIMEOUT` survivor is a hang report. `n += 1` → `n *= 1` in `disambiguate`
  froze the suffix at `-2` and spun forever; the same shape in `next_date`'s
  catch-up loop (3 mutants) meant a rule that never advances hangs `mt recur`.
  Both loops are now bounded (`MAX_DISAMBIGUATE_ATTEMPTS`, `MAX_CATCH_UP_STEPS`,
  10_000 each) with a typed error on exhaustion. Bound by a `for` over a fixed
  range, never by the counter the mutant can freeze — a `while n <= CEILING`
  guard is still infinite once the mutant stops incrementing `n`.
+ A survivor inside `#[cfg(not(feature = …))]` code is unreachable under the
  sweep's own feature set, and its name is indistinguishable from the live
  twin's, so `exclude_re` cannot target it. Fixed structurally: the off-state of
  the `rrule` feature moved into one stub module (`recur_rrule_off.rs`, selected
  by `#[path]`), so `recur.rs` and `recur_dates.rs` are one always-compiled
  function each and the whole stub file is one `exclude_globs` entry.
+ "Untestable environment boundary" was wrong five times over. `no_color_set`,
  `terminal_width`, `apply_flex`, `guard`'s `||` and `event_loop`'s `!=` were
  excluded as needing a real terminal — but the repo already had a PTY harness
  (`script(1)`, util-linux in the devShell) from the interactive e2e test.
  `tests/tty_e2e.rs` runs `mt ls` under a pty with `stty cols 40` vs `200`
  (pinning the flex column) and with `NO_COLOR` set vs unset (pinning the env
  read, no `set_var` and no `unsafe`); `interactive_e2e.rs` gained a case that
  redirects only stdin from `/dev/null` — the single shape that tells
  `!stdin || !stdout` from `&&`. `event_loop`'s exclusion was simply stale: the
  bounded `wait_with_timeout` added last session turns its hang into a fast test
  failure. Reach for the harness, and re-check old exclusions, before writing a
  new one.

== Follow-up

The one deferred question — whether a CI `--in-diff` advisory earns its keep now
the equivalent-mutant floor is stable — is filed as its own task,
`ci-in-diff-mutation-advisory.typ`.
