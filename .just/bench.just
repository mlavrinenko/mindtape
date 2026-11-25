set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

# The backlog size every recipe defaults to. The criterion suite reports 100,
# 1000 and 2000 regardless; this is the size the end-to-end recipes time.

BENCH_N := "2000"

# The bench-profile binary: release codegen, plus the symbols a profiler needs
# (`[profile.bench]` in Cargo.toml). Its own target dir keeps it off the lock
# `just check` holds on target/, and off target/release/mt.

BENCH_BIN := "target/bench-build/release/mt"

[default]
[private]
list:
    @just --list bench

# The fixture is built by the bench binary itself, so one generator serves the
# criterion suite and the end-to-end recipes. `--test` runs the selected
# benchmark once: enough to write the files, cheap enough to repeat.

[arg('N', pattern='\d+', help='number of deterministic task files to generate')]
[doc("Regenerate the gitignored target/bench/tasks-N benchmark fixture")]
gen N=BENCH_N:
    cargo bench -p mindtape --bench load -- --test "load/warm/{{ N }}"

[arg('N', pattern='\d+', help='number of generated task files to benchmark against')]
[doc("Benchmark mt ls and mt check end to end against a large task set")]
[script('bash')]
run N=BENCH_N:
    cargo build --profile bench -p mindtape-cli --target-dir target/bench-build
    just bench::gen {{ N }}
    export MINDTAPE_NOW=2026-07-16
    fixture="target/bench/tasks-{{ N }}"
    mkdir -p target/bench
    # `--prepare` runs before each timed run, so the cold arm starts from an
    # empty cache every time. Warm and no-cache get their own invocation
    # because hyperfine applies one prepare to every command in a run.
    hyperfine --warmup 1 --runs 5 --export-json target/bench/e2e-warm.json \
        --command-name "mt ls (warm, {{ N }} tasks)" "{{ BENCH_BIN }} -w $fixture ls" \
        --command-name "mt check (warm, {{ N }} tasks)" "{{ BENCH_BIN }} -w $fixture check"
    hyperfine --warmup 0 --runs 3 --prepare "rm -rf $fixture/.mindtape/cache" \
        --export-json target/bench/e2e-cold.json \
        --command-name "mt ls (cold, {{ N }} tasks)" "{{ BENCH_BIN }} -w $fixture ls"
    hyperfine --warmup 1 --runs 5 --export-json target/bench/e2e-nocache.json \
        --command-name "mt ls (no cache, {{ N }} tasks)" \
        "MINDTAPE_NO_CACHE=1 {{ BENCH_BIN }} -w $fixture ls"

# `-p mindtape --bench load` names the one criterion target. `--workspace` would
# also hand these flags to the lib's libtest harness, which rejects
# `--save-baseline` and `--baseline` as unrecognized options.

[doc("Run the criterion suite: load cold and warm, filter, sort")]
suite *ARGS:
    cargo bench -p mindtape --bench load -- {{ ARGS }}

[arg('NAME', help='baseline name, such as main')]
[doc("Run the criterion suite and save it as a named baseline")]
save NAME *ARGS:
    cargo bench -p mindtape --bench load -- --save-baseline {{ NAME }} {{ ARGS }}

[arg('NAME', help='a baseline name saved by `just bench::save`')]
[doc("Run the criterion suite against a saved baseline and report the change")]
compare NAME *ARGS:
    cargo bench -p mindtape --bench load -- --baseline {{ NAME }} {{ ARGS }}

# samply needs `perf_event_paranoid` at 1 or lower on Linux; the check below
# says so instead of letting the profiler fail with a sysctl lecture. macOS
# needs nothing.

[arg('N', pattern='\d+', help='number of task files')]
[doc("Record a samply profile of `mt ls` and open it in the Firefox Profiler")]
[script('bash')]
profile N=BENCH_N:
    if [ "$(uname -s)" = Linux ] && [ "$(cat /proc/sys/kernel/perf_event_paranoid)" -gt 1 ]; then
        echo "samply needs kernel.perf_event_paranoid <= 1:" >&2
        echo "    echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid" >&2
        exit 1
    fi
    cargo build --profile bench -p mindtape-cli --target-dir target/bench-build
    just bench::gen {{ N }}
    export MINDTAPE_NOW=2026-07-16
    samply record --save-only -o target/bench/mt-ls.json -- \
        {{ BENCH_BIN }} -w target/bench/tasks-{{ N }} ls
    echo "wrote target/bench/mt-ls.json — open it at https://profiler.firefox.com"
