set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[private]
list:
    @just --list advisory

[doc("Count tests across the workspace")]
[extension('.sh')]
[group("advisory")]
[script]
count-tests:
    cargo nextest list --workspace --message-format json \
        | jq -r '[.["rust-suites"][].testcases[] | select(.ignored == false)] | "\(length) tests"'

# awk, not head: an early exit by head kills sort, which pipefail reports.

[doc("Show the top 20 files by line count")]
[extension('.sh')]
[group("advisory")]
[script]
file-sizes:
    find . -type f \( -name '*.rs' -o -name '*.md' \) ! -path './target/*' -exec wc -l {} + | sort -rn | awk 'NR <= 20'

# A non-zero exit is the answer (9: a workaround can go), not a failure.

[doc("Say which upstream change in .config/canibump.toml landed, so its workaround can go")]
[group("advisory")]
[no-exit-message]
canibump *ARGS:
    canibump --manifest .config/canibump.toml --refresh {{ ARGS }}

# Refuses while `mt ls @ask-refactoring` still lists an open sweep task.

[arg('WRITE', long='write', pattern='[01]', value='1', help='file the tasks instead of printing what would be filed')]
[arg('BASELINE', long='baseline', pattern='(100|[1-9]?[0-9])', help='size arm: report files at or above PCT% of their linecop limit')]
[arg('CC_FLOOR', long='cc-floor', pattern='\d+', help='undertested arm: floor on cyclomatic complexity')]
[arg('FN_COVERAGE_FLOOR', long='fn-coverage-floor', pattern='(100|[1-9]?[0-9])', help='undertested arm: report functions below PCT% line coverage')]
[doc("File backlog tasks from advisory quality signals")]
[group("advisory")]
ask-refactoring $WRITE='0' $BASELINE='90' $CC_FLOOR='5' $FN_COVERAGE_FLOOR='80':
    bash contrib/ask-refactoring.sh --write "$WRITE" --baseline "$BASELINE" \
        --cc-floor "$CC_FLOOR" --fn-coverage-floor "$FN_COVERAGE_FLOOR"
