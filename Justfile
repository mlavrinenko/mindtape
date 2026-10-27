# Each module file repeats these settings, since a module inherits none, and
# adds `working-directory := '..'`, since a module recipe runs in its own
# file's directory.

set quiet := true
set unstable := true
set script-interpreter := ["bash", "-eo", "pipefail"]

# The docs site, its Markdown renders and the doc gates
[group: "areas"]
mod docs '.just/docs.just'
# Static gates over the source tree
[group: "areas"]
mod lint '.just/lint.just'
# Gates that lock the justfiles, the gate scripts and their fixtures
[group: "areas"]
mod meta '.just/meta.just'
# The CRAP gate over `just cover`'s lcov, and its baseline
[group: "areas"]
mod crap '.just/crap.just'
# Mutation testing sweeps
[group: "areas"]
mod mutants '.just/mutants.just'
# The doc drift lock: check it, re-record it
[group: "areas"]
mod outdatty '.just/outdatty.just'
# The large-backlog benchmark
[group: "areas"]
mod bench '.just/bench.just'
# Install mt, the Typst lib, schema and skill on this machine
[group: "areas"]
mod install '.just/install.just'

[doc("List available recipes")]
[group("workflow")]
default:
    @just --list

[doc("Run fixes, then run the full check gate")]
[group("workflow")]
fix-check: eject fmt clippy-fix check

[arg('STAGES', pattern='fast|build|cover', help='stages to run (fast, build, cover); all when none')]
[doc("Run every check gate, fast stage first; print only failures and a verdict")]
[group("workflow")]
[no-exit-message]
check *STAGES:
    bash .just/scripts/check.sh {{ STAGES }}

# www/gates.jq reads the gate table, rows in this order, from these stages.

[private]
_check-stages: _check-fast _check-build _check-cover

[parallel]
[private]
_check-fast: (mmz "fmt-check") (mmz "clippy") (mmz "lint::machete") (mmz "lint::workflows") (mmz "lint::shellcheck") (mmz "lint::size") (mmz "lint::dry") (mmz "lint::secrets") (mmz "outdatty::check") (mmz "docs::recipes") (mmz "docs::blocks") (mmz "meta::fmt-fixture") (mmz "outdatty::trailer-fixture") (mmz "meta::arg-pattern") (mmz "meta::positional-args") (mmz "meta::script-attr") (mmz "lint::path-attr") (mmz "lint::plural") (mmz "meta::size-fixture") (mmz "meta::script-scope") (mmz "crap::baseline-check") (mmz "meta::gate-drift")

[parallel]
[private]
_check-build: (mmz "test") (mmz "check-examples") (mmz "docs::coverage") (mmz "docs::facts") (mmz "docs::spans") (mmz "docs::prose") (mmz "docs::check") (mmz "docs::md-check") (mmz "docs::pages-fixture")

[private]
_check-cover: (mmz "cover") (mmz "crap::check")

[doc("Run a just recipe through mmz memoization")]
[group("workflow")]
[no-exit-message]
mmz *ARGS:
    bash .just/scripts/mmz-arm.sh {{ ARGS }}

# worktrunk takes no merge defaults from project config, so they live here.
# `--no-squash` keeps each commit's `Refs:` footer and `--no-commit` refuses a
# dirty tree. A recipe cannot move its caller's shell, so neither one cds. The
# cd directive goes to a throwaway file, so wt has a real file to write it to
# and prints no "shell integration not installed" warning.

[doc("Fork a sibling worktree on a new branch, with its dev mt built")]
[group("workflow")]
[positional-arguments]
fork slug *args:
    wt switch --create --yes --no-cd "$@"

[doc("Gate, fast-forward the branch onto main and remove its worktree")]
[extension('.sh')]
[group("workflow")]
[positional-arguments]
[script]
land *args:
    cd_file=$(mktemp)
    trap 'rm -f "$cd_file"' EXIT
    WORKTRUNK_DIRECTIVE_CD_FILE="$cd_file" wt merge --no-squash --no-commit --yes "$@"

[doc("Build the project")]
[group("dev")]
[positional-arguments]
build *args:
    cargo build --workspace -q "$@"

# Its own target dir keeps it off the lock `just check` holds on target/, and
# off target/debug/mt, which the test arm links with test features.

[doc("Build the dev shell's mt")]
[group("dev")]
build-mt:
    cargo build -q -p mindtape-cli --bin mt --target-dir target/mt-dev

# nextest runs no doctests: the first doctest needs a `cargo test --doc` pass.
# A second `--cargo-quiet` would hide build errors too.

[doc("Run tests only")]
[group("dev")]
[group("gate")]
[positional-arguments]
test *ARGS:
    cargo nextest run --workspace --status-level fail --cargo-quiet "$@"

[doc("Run coverage with cargo-llvm-cov")]
[group("gate")]
cover:
    bash .just/scripts/cover.sh

[doc("Run clippy only")]
[group("dev")]
[group("gate")]
clippy:
    bash .just/scripts/clippy.sh

[doc("Auto-fix clippy warnings")]
[group("dev")]
clippy-fix:
    cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged -- -D warnings

TOML_FMT_PATHS := ".config/canibump.toml .config/gitleaks.toml .config/mutants.toml .config/nextest.toml .config/release.toml .config/tombi.toml .config/wt.toml .mindtape/config.toml Cargo.toml clippy.toml crates/mindtape-cli/Cargo.toml crates/mindtape-cli/src/cmd/init/axes/headers_spec.toml crates/mindtape-cli/src/cmd/init/axes/workflow_flow.toml crates/mindtape-cli/src/cmd/init/axes/workflow_gated.toml crates/mindtape-cli/src/cmd/init/axes/workflow_personal.toml crates/mindtape-cli/src/cmd/init/axes/workflow_todo_done.toml crates/mindtape-cli/src/cmd/init/presets/flow/config.toml crates/mindtape-cli/src/cmd/init/presets/gated/config.toml crates/mindtape-cli/src/cmd/init/presets/personal/config.toml crates/mindtape-cli/src/cmd/init/presets/todo-done/config.toml crates/mindtape/Cargo.toml crates/mindtape/src/at/board.toml crates/mindtape/src/config/defaults.toml crates/mindtape/src/framework/frameworks.toml crates/mindtape/src/packs/budget.toml crates/mindtape/src/packs/commits.toml crates/mindtape/src/packs/dogfood.toml crates/mindtape/src/packs/flow.toml crates/mindtape/src/packs/ice.toml crates/mindtape/src/packs/tasks.toml crates/mindtape/src/packs/tweaks.toml crates/mindtape/src/packs/wip-caps.toml examples/across-projects/.config/mindtape/at.toml examples/across-projects/projects/blog/.mindtape/config.toml examples/across-projects/projects/shop/.mindtape/config.toml examples/api-service/.mindtape/config.toml examples/derived-tags/.mindtape/config.toml examples/dsl/.mindtape/config.toml examples/estimates/.mindtape/config.toml examples/large/.mindtape/config.toml examples/links/.mindtape/config.toml examples/minimal/.mindtape/config.toml examples/named-sets/.mindtape/config.toml examples/priority/.mindtape/config.toml examples/recurring/.mindtape/config.toml examples/round-trip/.mindtape/config.toml examples/rules/.mindtape/config.toml examples/self-validating/.mindtape/config.toml examples/statuses/.mindtape/config.toml examples/task-files/.mindtape/config.toml lib/typst.toml rust-toolchain.toml rustfmt.toml www/tola.toml"
TYPST_FMT_PATHS := "lib truth .mindtape/templates examples/api-service/tasks examples/api-service/.mindtape/templates examples/estimates/.mindtape/templates examples/self-validating/.mindtape/templates examples/named-sets/.mindtape/templates examples/derived-tags/api examples/derived-tags/web examples/links/tasks examples/named-sets/tasks examples/priority/tasks examples/recurring/tasks examples/round-trip/tasks examples/dsl/tasks examples/task-files/tasks examples/across-projects/projects/blog/tasks examples/across-projects/projects/shop/tasks examples/lsp examples/rules examples/statuses/tasks www/content www/templates www/utils docs/src"

# A bare `just --fmt` skips every module file.

JUST_FMT_PATHS := ".just/*.just"

[doc("Format code")]
[group("dev")]
fmt:
    cargo fmt --all
    tombi format {{ TOML_FMT_PATHS }}
    typstyle --inplace {{ TYPST_FMT_PATHS }}
    just --fmt
    for f in {{ JUST_FMT_PATHS }}; do just --justfile "$f" --fmt; done

# `just --fmt --check` diffs with the whole file as context; `--dump` does not.

[doc("Check formatting without writing changes (CI-friendly)")]
[group("gate")]
fmt-check:
    cargo fmt --all -- --check
    tombi format --check --quiet {{ TOML_FMT_PATHS }}
    typstyle --check {{ TYPST_FMT_PATHS }}
    just --dump | diff -u Justfile -
    for f in {{ JUST_FMT_PATHS }}; do just --justfile "$f" --dump | diff -u "$f" -; done

[doc("Validate each example project against itself")]
[group("gate")]
check-examples:
    bash .just/scripts/check-examples.sh

[arg('PCT', pattern='(100|[1-9]?[0-9])', help='linecop baseline percentage (0-100) — files at or above it get ejected')]
[doc("Eject inline tests from Rust files nearing the linecop cap")]
[group("dev")]
eject PCT='90':
    linecop --baseline {{ PCT }} --format paths | ejectest apply . --files-from - --lenient --mod-rs-tests tests

[doc("Cross-check the workspace against Windows (x86_64-pc-windows-gnu)")]
[group("dev")]
check-windows:
    nix develop .#windows --command \
        cargo check --workspace --all-targets --locked \
        --target x86_64-pc-windows-gnu

# The config is shared by every worktree, so a driver goes only when no
# worktree's .gitattributes names it. `merge.<name>.mindtape` marks the ones
# this recipe registers; an unmarked driver that runs a .just/scripts/ script
# predates the mark. Anything else is the user's and stays.

[doc("Point git at the tracked hooks, register the merge drivers and drop stale ones")]
[extension('.sh')]
[group("dev")]
[script]
install-hooks:
    # git exports GIT_DIR to a hook's children and it beats `git -C`: drop it, so the recipe writes only the repo it runs in.
    unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE
    git config core.hooksPath .githooks
    register() {
        git config "merge.$1.name" "$2"
        git config "merge.$1.driver" "$3"
        git config "merge.$1.mindtape" true
    }
    register outdatty ".config/outdatty.lock, merged key by key" 'outdatty merge-lock %O %A %B'
    register jscpd-baseline ".config/jscpd-baseline.json, merged by jscpd fingerprint by fingerprint" 'jscpd --merge-baseline %O %A %B'
    register crap-baseline ".config/crap-baseline.json, merged entry by entry" 'bash .just/scripts/crap-baseline-merge.sh %O %A %B %P'
    named=$(git worktree list --porcelain | sed -n 's/^worktree //p' | while IFS= read -r tree; do
        sed -n 's/^[^#].*[[:space:]]merge=\([^[:space:]]*\).*/\1/p' "$tree/.gitattributes" 2>/dev/null || true
    done)
    git config --name-only --get-regexp '^merge\..*\.driver$' | sed -e 's/^merge\.//' -e 's/\.driver$//' | while IFS= read -r name; do
        if printf '%s\n' "$named" | grep -qxF -- "$name"; then continue; fi
        if [ "$(git config --get "merge.$name.mindtape")" = true ] || git config --get "merge.$name.driver" | grep -q '\.just/scripts/'; then
            git config --remove-section "merge.$name"
            echo "dropped the stale merge driver $name"
        fi
    done
    rm -f .git/hooks/pre-commit
    echo "installed .githooks and the .config/outdatty.lock, .config/jscpd-baseline.json and .config/crap-baseline.json merge drivers"

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

[arg('DRY_GAIN', long='dry-gain', pattern='\d+', help='dry arm: report baselined clones whose extraction saves N tokens or more')]
[arg('WRITE', long='write', pattern='[01]', value='1', help='file the tasks instead of printing what would be filed')]
[arg('REPEAT_GAIN', long='repeat-gain', pattern='\d+', help='repeats arm: report snippets whose extraction saves N tokens or more')]
[arg('BASELINE', long='baseline', pattern='(100|[1-9]?[0-9])', help='size arm: report files at or above PCT% of their linecop limit')]
[arg('CC_FLOOR', long='cc-floor', pattern='\d+', help='undertested arm: floor on cyclomatic complexity')]
[arg('FN_COVERAGE_FLOOR', long='fn-coverage-floor', pattern='(100|[1-9]?[0-9])', help='undertested arm: report functions below PCT% line coverage')]
[doc("File backlog tasks from advisory quality signals")]
[group("advisory")]
ask-refactoring $WRITE='0' $BASELINE='90' $CC_FLOOR='5' $FN_COVERAGE_FLOOR='80' $REPEAT_GAIN='300' $DRY_GAIN='300':
    bash contrib/ask-refactoring.sh --write "$WRITE" --baseline "$BASELINE" \
        --cc-floor "$CC_FLOOR" --fn-coverage-floor "$FN_COVERAGE_FLOOR" \
        --repeat-gain "$REPEAT_GAIN" --dry-gain "$DRY_GAIN"

# Flakes see only git-known files, so new ones are marked intent-to-add.

[doc("Verify nix build succeeds in a clean sandbox")]
[group("release")]
nix-build:
    git ls-files -z --others --exclude-standard | xargs -0 -r git add --intent-to-add --
    nix build .#default

[arg('DRY_RUN', long='dry-run', value='true', help='run every check, then say what would be pushed')]
[arg('VERSION', pattern='\d+\.\d+\.\d+', help='semver, no leading v (e.g. 0.3.0)')]
[doc("Push main, wait for its CI, then tag and push the tag")]
[group("release")]
[no-exit-message]
release VERSION $DRY_RUN='':
    bash .just/scripts/release.sh {{ VERSION }}
