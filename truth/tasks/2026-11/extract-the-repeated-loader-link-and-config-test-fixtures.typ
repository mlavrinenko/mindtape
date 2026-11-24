#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Extract the repeated loader, link and config test fixtures",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tests", "repeats", "stateless"),
  assignee: "agent",
  status: proposed(2026, 11, 24),
)

== AS-IS

+ Running the repeats arm over `crates/` while the loader, link and config tests grow
+ 🔴 #raw("crates/mindtape/src/loader/tests.rs"): 26 copies of `let dir = setup(); write_file(dir.path(), "tasks/build.typ", TASK_BODY);`, 23 jscpd tokens, gain 435
+ 🔴 #raw("crates/mindtape/src/loader/tests.rs"): 19 copies of `let outcome = load_project(dir.path(), &config); assert_eq!(outcome.errors, Vec::new());`, 29 tokens, gain 417
+ 🔴 #raw("crates/mindtape/src/validate/links_tests.rs"): 10 copies of the two-task `check(&[a, b], &Config::default())` setup, 43 tokens, gain 327
+ 🔴 #raw("crates/mindtape/src/config/actors_tests.rs"): 14 copies of `parse_config(...).unwrap(); assert!(config.validate().is_ok());`, 42 tokens, gain 466

```text
$ jscpd --no-tips -f rust -k 15 -l 1 -r json -o /tmp/rep crates
$ jq -r -f <score_repeats> /tmp/rep/jscpd-report.json
crates/mindtape/src/config/actors_tests.rs	113	14	42	466	5
crates/mindtape/src/loader/tests.rs	69	26	23	435	6
crates/mindtape/src/loader/tests.rs	78	19	29	417	6
crates/mindtape/src/validate/links_tests.rs	36	10	43	327	1
```


== TO-BE

- 🟢 Each repeated shape lives in one helper. `test_support::project_with_file` builds a project plus its one file. `loader::tests::load_ok` loads a project and asserts its errors are empty. `loader::tests::one_error` asserts a load failed on one file. `loader::tests::load_one_task` returns the one task a load must find. `validate::links_tests::check_pair` checks a two-task link set. `test_support::assert_valid` parses a config and asserts it validates.
- 🟢 Every copy calls its helper. A fixture fix lands once.
- 🟢 The repeats arm doesn't report a snippet under `crates/mindtape/src/`.

```text
$ jscpd --no-tips -f rust -k 15 -l 1 -r json -o /tmp/rep crates
$ jq -r -f <score_repeats> /tmp/rep/jscpd-report.json | grep mindtape/src
(no output)
```


== Changes

- `crates/mindtape/src/test_support/fs.rs`: `project_with_file`, a tempdir with one file written.
- `crates/mindtape/src/test_support/mod.rs`: re-export it, and add `assert_valid`.
- `crates/mindtape/src/loader/tests.rs`: `load_ok`, `one_error` and `load_one_task`, and their call sites.
- `crates/mindtape/src/loader/tests_cache.rs`, `tests_one.rs`, `tests_visible.rs`, `tests_eval_line.rs`: call the loader helpers.
- `crates/mindtape/src/validate/candidate_tests.rs`, `candidate_tests_markers.rs`: call `project_with_file`.
- `crates/mindtape/src/validate/links_tests.rs`: `check_pair` and its ten call sites.
- `crates/mindtape/src/config/actors_tests.rs`, `tests.rs`, `tests_ls.rs`, `framework/tests.rs`, `framework/validate_tests.rs`: call `assert_valid`.


== Out of scope

- The 23 remaining repeats-arm findings, all under `crates/mindtape-cli/`. They are another task's area.
- The first-line convention for AS-IS, the repeats gate and the qahq bump. Each has its own task.
- `// jscpd:ignore` markers. This change fixes the copies rather than waiving them.


== Dogfood

The next edit to a loader fixture lands in one helper, and the repeats arm stays quiet on `crates/mindtape/src/`.


== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a fixture pasted across the loader, link and config tests lives in one helper. A fix lands once, and the repeats arm stays quiet].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).

