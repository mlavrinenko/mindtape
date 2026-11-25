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

# Attribution runs no sampler. callgrind counts instructions and strace counts
# syscalls, and both run unprivileged, so a recipe built on either works for a
# normal user. They also answer the two questions this repo has: the warm path
# is CPU, which callgrind attributes to a function, and the cold path is
# blocked, which strace attributes to a syscall.

[arg('N', pattern='\d+', help='number of task files')]
[doc("Attribute the warm path's CPU to functions with callgrind")]
[script('bash')]
profile N=BENCH_N:
    cargo build --profile bench -p mindtape-cli --target-dir target/bench-build
    just bench::gen {{ N }}
    export MINDTAPE_NOW=2026-07-16
    mkdir -p target/bench
    # callgrind runs the binary about 200x slower, so 2000 tasks take about 40 s.
    valgrind --tool=callgrind --callgrind-out-file=target/bench/callgrind.out \
        {{ BENCH_BIN }} -w target/bench/tasks-{{ N }} ls > /dev/null
    # `--auto=no` keeps the report to the function list; auto-annotation prints
    # every source file it can reach before the numbers. The distribution is
    # flat, so 80% of the instructions spans thousands of functions: the whole
    # report goes to a file and the top of it to the terminal.
    callgrind_annotate --auto=no --threshold=80 target/bench/callgrind.out \
        > target/bench/callgrind.txt
    awk '/file:function/{seen=1} seen && n++ < 25' target/bench/callgrind.txt
    echo "full report: target/bench/callgrind.txt"

[arg('N', pattern='\d+', help='number of task files')]
[doc("Attribute a cold run's time to syscalls with strace")]
[script('bash')]
syscalls N=BENCH_N:
    cargo build --profile bench -p mindtape-cli --target-dir target/bench-build
    just bench::gen {{ N }}
    export MINDTAPE_NOW=2026-07-16
    mkdir -p target/bench
    # `-c` totals each syscall, and `-f` follows the threads. The cold arm is
    # the one worth reading: a warm run makes too few calls to attribute.
    rm -rf target/bench/tasks-{{ N }}/.mindtape/cache
    strace -c -f -o target/bench/strace-cold.txt \
        {{ BENCH_BIN }} -w target/bench/tasks-{{ N }} ls > /dev/null
    cat target/bench/strace-cold.txt
