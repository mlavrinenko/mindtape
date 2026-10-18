set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[private]
list:
    @just --list mutants

[doc("Run mutation testing over this branch's diff against main")]
[positional-arguments]
diff *ARGS:
    bash .just/scripts/mutants-diff.sh "$@"

[doc("Run a full-file mutation testing sweep; -f <file> for one file")]
[positional-arguments]
sweep *ARGS:
    bash .just/scripts/mutants.sh "$@"

[doc("Report progress and ETA of a mutants sweep")]
[positional-arguments]
status DIR='mutants.out':
    bash .just/scripts/mutants-status.sh "$@"
