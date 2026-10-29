#!/usr/bin/env bash
# Body of `just check-examples`: each fixture another tool runs `mt` against,
# checked against its own project. Each fixture's `[[typst.local-packages]]`
# points `@local/mindtape` at this checkout's lib/. An empty
# MINDTAPE_DATA_HOME removes the machine-global package, so a fixture that lost
# its entry reads the prelude `mt` embeds, which a test keeps equal to lib/,
# and never another checkout's. XDG_DATA_HOME would not do: `dirs` reads it on
# Unix only.
#
# One `--tests` build, then the binary itself: a plain `cargo run` relinks
# target/debug/mt without the features the test arm links it with.
set -eo pipefail
examples=(
    minimal links estimates dsl statuses rules api-service derived-tags
    round-trip named-sets priority recurring task-files
    across-projects/projects/shop across-projects/projects/blog
)
cargo build -p mindtape-cli --tests -q
for ex in "${examples[@]}"; do
    MINDTAPE_DATA_HOME='' ./target/debug/mt -w "examples/$ex" check -q
done
