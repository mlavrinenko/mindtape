<!-- Generated from docs/src/contributing/command-semantics.typ by `just docs md`. Do not edit; edit the source. -->

# Command semantics (read/write separation)

Deep dive behind CONTRIBUTING.md’s read/write rule: every verb’s side of the split, and why it is declared rather than remembered.

MindTape follows a strict read/write split (CQRS). Read commands never mutate the working tree; only explicit write commands do. The split is declared once, in `crates/mindtape-cli/src/command_class.rs`, checked for exhaustiveness against `mt _cli --json`’s visible command list in `cli_dump.rs`’s own tests, and read here from `fact("cli")` rather than retyped.

- Read (pure): `ls`, `find`, `graph`, `views`, `worktrees`, `check`, `config show`, `config schema`, `preset ls`, `eval` — each leaves the tree byte-identical, asserted per command in `crates/mindtape-cli/tests/read_no_write_e2e.rs`. Add your command’s test there, not to this list.
- Write (explicit): `add`, `import`, `preset save`, `init`, `install`, `rm`, `mv`, `set`, `review`, `flip`, `recur`, `link`. Every legend value is also a top-level verb over `flip` — `mt done` is `mt flip done`, generated from the legend rather than declared as a subcommand.
- `go` is neither: hands the file to the configured editor; mutates nothing itself.

Recurrence never materializes as a side effect of a read: an instance is spawned or refreshed only by an explicit `mt recur`, and `mt done` closes a task without creating or advancing anything.

“Byte-identical” means the _authored_ tree — task files and config. A read may still populate the git-ignored `.mindtape/cache/` scratch, derived state the way `target/` is.
