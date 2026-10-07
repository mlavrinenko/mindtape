<!-- Generated from docs/src/contributing/command-semantics.typ by `just docs md`. Do not edit; edit the source. -->

# Command semantics (read/write separation)

Which `mt` commands read and which write, for the rule in CONTRIBUTING.md’s Command Semantics section.

`crates/mindtape-cli/src/cli/command_class.rs` declares each command’s class. The tests in `cli/tree_json.rs` fail when a visible command in `mt _cli --json` has no class, and the lists below read the same data.

- Read: `ls`, `find`, `graph`, `views`, `worktrees`, `check`, `config show`, `config schema`, `preset ls`, `eval`. Each leaves the tree byte-identical, and `crates/mindtape-cli/tests/read_no_write_e2e.rs` checks each one. Add a new read command’s test there.
- Write: `add`, `import`, `preset save`, `init`, `install`, `rm`, `mv`, `set`, `review`, `flip`, `recur`, `link`. Every status in the legend is also a top-level verb for `flip`, so `mt done` runs `mt flip done`. `mt` builds those verbs from the legend.
- `go` is neither: hands the file to the configured editor; mutates nothing itself.

A read never creates a recurring instance. Only `mt recur` creates or refreshes one, and `mt done` closes a task without creating or advancing anything.

The authored tree is the task files and the config. A read may still write the git-ignored `.mindtape/cache/`, which holds derived state the way `target/` does.
