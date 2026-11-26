//! Instruction-count benchmarks: the CPU-bound cases, counted under `callgrind`.
//!
//! `just bench::instructions` runs this target. `benches/load.rs` reports
//! wall-clock time and needs a quiet machine. This target counts the
//! instructions the CPU retired instead, and that count is the same on a shared
//! CI runner as on a laptop. Each case carries an absolute ceiling: a run that
//! retires more instructions than its ceiling fails.
//!
//! The ceilings are not a target. Each sits about 5% above the measured cost,
//! and the run-to-run spread is under 1%. A change that makes the sort or the
//! filter meaningfully more expensive fails here instead of waiting for someone
//! to run the wall-clock suite. A change that makes them cheaper should lower
//! the ceiling in the same commit, which ratchets the gate down.
//!
//! `#[library_benchmark]` rejects every attribute it does not own, a doc
//! comment included. Each case explains itself in a `//` comment for that
//! reason.
// `main!` expands to a `std::process::exit` and to the macros that report a
// failed case, which the workspace lints refuse in library code. A bench
// binary's whole job is to exit non-zero on a regression.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::unwrap_used
)]

// `clear_cache` belongs to `benches/load.rs`, which shares this module.
#[allow(dead_code)]
mod fixture;

use std::hint::black_box;

use iai_callgrind::{
    Callgrind, EventKind, LibraryBenchmarkConfig, library_benchmark, library_benchmark_group, main,
};
use mindtape::{Config, Filter, Readiness, Task, default_order_with, load_config, load_project};

/// The backlog size both cases run over, matching `benches/load.rs`.
const N: usize = 2000;

/// The filter `filter/match` matches, parsed once in setup.
const FILTER: &str = "!terminal && priority >= 3 && has_tag(\"bench\")";

/// The instruction ceiling for `filter/match/2000`. Measured at 32.1 M on
/// 2026-11-27, and the ceiling leaves 6% of headroom over that.
const FILTER_MATCH_CEILING: u64 = 34_000_000;

/// The instruction ceiling for `sort/default/2000`. Measured at 18.3 M on
/// 2026-11-25, which is 9.1 k instructions per task.
const SORT_DEFAULT_CEILING: u64 = 19_500_000;

/// The instruction ceiling for `load/warm/2000`: a whole warm load, the region
/// `mt ls` pays before it filters anything. Measured at 173.1 M on 2026-11-27.
/// The dependency-hash memo and the cached call spans took it down from 2.5 G
/// on the flagship's own backlog. The ceiling leaves 5% of headroom.
const LOAD_WARM_CEILING: u64 = 182_000_000;

/// The loaded set and its config, built once per case and excluded from the
/// count: iai-callgrind zeroes the counters after setup returns.
fn loaded() -> (Config, Vec<Task>) {
    let root = fixture::fixture(N);
    let config = load_config(&root).unwrap();
    let outcome = load_project(&root, &config);
    (config, outcome.tasks)
}

/// The fixture's root and config, with its cache primed so the measured load is
/// the steady state rather than the cold one.
fn warm_fixture() -> (std::path::PathBuf, Config) {
    let root = fixture::fixture(N);
    let config = load_config(&root).unwrap();
    let _ = load_project(&root, &config);
    (root, config)
}

/// The loaded set plus the parsed filter. The measured region is the match loop
/// alone.
fn loaded_with_filter() -> (Config, Vec<Task>, Filter) {
    let (config, tasks) = loaded();
    (config, tasks, Filter::parse(FILTER).unwrap())
}

/// A config that fails the case when it retires more than `limit`
/// instructions.
fn ceiling(limit: u64) -> LibraryBenchmarkConfig {
    let mut callgrind = Callgrind::default();
    callgrind.hard_limits([(EventKind::Ir, limit)]);
    let mut config = LibraryBenchmarkConfig::default();
    config.tool(callgrind);
    config
}

// Matching a parsed filter against every task: the per-task predicate, which is
// what `mt ls` spends on the query itself.
#[library_benchmark(config = ceiling(FILTER_MATCH_CEILING))]
#[bench::n2000(setup = loaded_with_filter)]
fn filter_match((config, tasks, filter): (Config, Vec<Task>, Filter)) -> usize {
    let readiness = Readiness::empty();
    let mut kept = 0_usize;
    for task in &tasks {
        if filter.matches_with(task, &config, &readiness).unwrap() {
            kept += 1;
        }
    }
    black_box(kept)
}

// The default ordering over a loaded set. The set arrives from setup, so the
// count is the sort alone.
#[library_benchmark(config = ceiling(SORT_DEFAULT_CEILING))]
#[bench::n2000(setup = loaded)]
fn sort_default((config, mut tasks): (Config, Vec<Task>)) -> Vec<Task> {
    let readiness = Readiness::empty();
    default_order_with(&mut tasks, &config, &readiness);
    black_box(tasks)
}

// A whole warm load: re-stat and re-hash every file, read each entry from the
// cache, and assemble the set. This is the region `mt ls` pays before it
// filters, sorts or renders anything, and the one the dependency-hash memo and
// the cached call spans made four and a half times cheaper.
#[library_benchmark(config = ceiling(LOAD_WARM_CEILING))]
#[bench::n2000(setup = warm_fixture)]
fn load_warm((root, config): (std::path::PathBuf, Config)) -> usize {
    black_box(load_project(&root, &config).tasks.len())
}

library_benchmark_group!(
    name = cpu_bound;
    benchmarks = filter_match, sort_default, load_warm
);

main!(
    config = LibraryBenchmarkConfig::default().pass_through_envs([
        // iai-callgrind clears the environment before a case runs. The load
        // path shells out to `git` to list a root, so without `PATH` every
        // case measures an empty set and reports a number that means nothing.
        "PATH",
        // `dirs` resolves the Typst local-package directory from these.
        "HOME",
        "XDG_DATA_HOME",
        "XDG_CACHE_HOME",
        // Pins "today" for a run that wants a fixed date, as `just bench::run`
        // does. Absent, the load reads the machine's clock.
        "MINDTAPE_NOW",
    ]);
    library_benchmark_groups = cpu_bound
);
