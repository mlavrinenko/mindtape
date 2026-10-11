// docs/src/contributing/command-semantics.typ renders
// docs/contributing/command-semantics.md through docs/generate-md.sh (typlite;
// docs/src/lib.typ lists the constraints).
//
// The command lists read `fact("cli")`, the `mt _cli --json` tree that
// `cli/command_class.rs` classifies.
#import "../lib.typ": doc-title, fact

#metadata((
  output: "docs/contributing/command-semantics.md",
)) <mindtape-md>

#doc-title[Command semantics (read/write separation)]

Which `mt` commands read and which write. CONTRIBUTING.md's Command Semantics
section states the rule.

Each command declares its class in
`crates/mindtape-cli/src/cli/command_class.rs`. The tests in
`cli/tree_json.rs` fail when a visible command in `mt _cli --json` has no
class, and the lists below read the same data.

#let cli-leaves(commands, prefix: none) = (
  commands
    .map(c => {
      let name = if prefix == none { c.name } else { prefix + " " + c.name }
      if c.subcommands.len() == 0 {
        ((name: name, class: c.class),)
      } else {
        cli-leaves(c.subcommands, prefix: name)
      }
    })
    .flatten()
)
#let cli-classes = cli-leaves(fact("cli").commands)
#let names(kind) = (
  cli-classes.filter(l => l.class.kind == kind).map(l => raw(l.name))
)
#let exempt = cli-classes.filter(l => l.class.kind == "exempt")

- Read: #names("read").join(", "). Each leaves the tree byte-identical, and
  `crates/mindtape-cli/tests/it/read_no_write_e2e.rs` checks each one. When you
  add a read command, add its test there. Each also takes `--json`, and
  `every_read_command_takes_json` in `cli/tree_json_tests.rs` fails naming a
  read command without it. `config schema` is exempt, since it prints JSON
  already.
- Write: #names("write").join(", "). Every status in the legend is also a
  top-level verb for `flip`, so `mt done` runs `mt flip done`. `mt` builds
  those verbs from the legend.
#for e in exempt [
  - #raw(e.name) is neither: #e.class.reason.
]

A read never creates a recurring instance. Only `mt recur` creates or
refreshes one, and `mt done` closes a task without creating or advancing
anything.

The authored tree is the task files and the config. A read can still write
the git-ignored `.mindtape/cache/`, which holds derived state the way
`target/` does.

A read runs no `[[on.flip]]` action. The one config command a read runs is a
`[[tasks.waits]]` checker, for an open task that names it in `waits-on`,
because `ready` and `blocked` depend on its answer. A checker must answer from
local state and write nothing in the authored tree. It runs only once
`mt allow` has recorded the config's commands for the repository, so a fresh
clone runs nothing it ships.
