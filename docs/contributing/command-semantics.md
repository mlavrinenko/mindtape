<!-- Generated from docs/src/contributing/command-semantics.typ by `just docs md`. Do not edit; edit the source. -->

# Command semantics (read/write separation)

Which `mt` commands read and which write. CONTRIBUTING.md’s Command Semantics section states the rule.

Each command declares its class in `crates/mindtape-cli/src/cli/command_class.rs`. The tests in `cli/tree_json.rs` fail when a visible command in `mt _cli --json` has no class, and the lists below read the same data.

- Read: `ls`, `find`, `check`, `eval`, `graph`, `views`, `dicts`, `config show`, `config schema`, `rules`, `preset ls`, `at`. Each leaves the tree byte-identical, and `every_classified_read_command_leaves_tree_untouched` in `crates/mindtape-cli/tests/it/read_no_write_e2e.rs` runs each command its class marks read. A new one that takes a required argument gets it from that file’s `extra_args`. Each also takes `--json`, and `every_read_command_takes_json` in `cli/tree_json_tests.rs` fails naming a read command without it. `config schema` is exempt, since it prints JSON already.
- Write: `init`, `add`, `flip`, `set`, `link`, `mv`, `rm`, `review`, `recur`, `import`, `install`, `allow`, `preset save`. Every status in the legend is also a top-level verb for `flip`, so `mt done` runs `mt flip done`. `mt` builds those verbs from the legend.
- `go` is neither: hands the file to the configured editor; mutates nothing itself.

A read never creates a recurring instance. Only `mt recur` creates or refreshes one, and `mt done` closes a task without creating or advancing anything.

The authored tree is the task files and the config. A read can still write the git-ignored `.mindtape/cache/`, which holds derived state the way `target/` does.

A read runs no `[[on.flip]]` action. The one config command a read runs is a `[[tasks.waits]]` checker, for an open task that names it in `waits-on`, because `ready` and `blocked` depend on its answer. A checker must answer from local state and write nothing in the authored tree. It runs, as a flip’s `run` action does, only once `mt allow` has recorded the config’s commands for the repository, so a fresh clone runs nothing it ships.
