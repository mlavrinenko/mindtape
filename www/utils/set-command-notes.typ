// What each command that runs on a set does there, keyed by the command's
// path in cli.json. Which commands run on a set comes from
// www/generated/set-commands.json, which www/generate-facts.sh reads off the
// `runs_on_set` mark that `CLASSIFICATION` in
// crates/mindtape-cli/src/cli/command_class.rs gives each command, the same
// mark `main` refuses a set by. So this dictionary holds only what a command
// does on a set. `just docs::facts` fails on a marked command with no
// entry here, and `set-commands` below fails the build on the same gap. Both
// readers below are functions, so importing the dictionary alone reads no
// generated file.

#let set-command-notes = (
  "ls": [`mt @work ls` prints one table for the whole set, a `PROJECT` column first.],
  "find": [`mt @work find` searches every project and prints one table, a `PROJECT` column first.],
  "board": [A bare `mt @work` runs it, one row per project.],
  "check": [`mt @work check` checks each project under its own rules and ends with one line for the set.],
)

// The commands that run on a set, in the order `mt --help` lists them.
#let set-commands() = {
  let commands = json("../generated/set-commands.json").commands
  for name in commands {
    assert(
      name in set-command-notes,
      message: "www/utils/set-command-notes.typ has no note for the set command "
        + name,
    )
  }
  commands
}

// The same commands as a sentence's list: `ls`, `find`, `board` and `check`.
#let set-command-list() = set-commands().map(raw).join([, ], last: [ and ])
