// docs/src/contributing/command-semantics.typ — source for
// docs/contributing/command-semantics.md. Rendered by docs/generate-md.sh via
// typlite, same as every other docs/src/*.typ source; see docs/src/lib.typ
// for the render path's constraints.
//
// The verb list is derived: `fact("cli")` reflects `mt _cli --json`, the tree
// `command_class.rs` classifies. This page keeps the full read/write split out
// of the always-loaded CONTRIBUTING.md (per the .linecop.yaml note), so only a
// reader who needs it opens it.
#import "../lib.typ": doc-title, fact

#metadata((
  output: "docs/contributing/command-semantics.md",
)) <mindtape-md>

#doc-title[Command semantics (read/write separation)]

Deep dive behind CONTRIBUTING.md's read/write rule: every verb's side of the
split, and why it is declared rather than remembered.

MindTape follows a strict read/write split (CQRS). Read commands never mutate
the working tree; only explicit write commands do. The split is declared once,
in `crates/mindtape-cli/src/command_class.rs`, checked for exhaustiveness
against `mt _cli --json`'s visible command list in `cli_dump.rs`'s own tests,
and read here from `fact("cli")` rather than retyped.

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

- Read (pure): #names("read").join(", ") — each leaves the tree
  byte-identical, asserted per command in
  `crates/mindtape-cli/tests/read_no_write_e2e.rs`. Add your command's test
  there, not to this list.
- Write (explicit): #names("write").join(", "). Every legend value is also a
  top-level verb over `flip` — `mt done` is `mt flip done`, generated from
  the legend rather than declared as a subcommand.
#for e in exempt [
  - #raw(e.name) is neither: #e.class.reason.
]

Recurrence never materializes as a side effect of a read: an instance is
spawned or refreshed only by an explicit `mt recur`, and `mt done` closes a
task without creating or advancing anything.

"Byte-identical" means the _authored_ tree — task files and config. A read may
still populate the git-ignored `.mindtape/cache/` scratch, derived state the
way `target/` is.
