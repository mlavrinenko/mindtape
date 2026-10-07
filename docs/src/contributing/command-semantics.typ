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

Which `mt` commands read and which write, for the rule in CONTRIBUTING.md's
Command Semantics section.

`crates/mindtape-cli/src/cli/command_class.rs` declares each command's class.
The tests in `cli/tree_json.rs` fail when a visible command in `mt _cli --json`
has no class, and the lists below read the same data.

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
  `crates/mindtape-cli/tests/read_no_write_e2e.rs` checks each one. Add a new
  read command's test there.
- Write: #names("write").join(", "). Every status in the legend is also a
  top-level verb for `flip`, so `mt done` runs `mt flip done`. `mt` builds
  those verbs from the legend.
#for e in exempt [
  - #raw(e.name) is neither: #e.class.reason.
]

A read never creates a recurring instance. Only `mt recur` creates or
refreshes one, and `mt done` closes a task without creating or advancing
anything.

The authored tree is the task files and the config. A read may still write
the git-ignored `.mindtape/cache/`, which holds derived state the way
`target/` does.
