//! Criterion benchmarks for the load path: what `mt ls` and `mt check` pay
//! before they filter, sort or render anything.
//!
//! Run with `just bench::suite`. Compare against a saved baseline with
//! `just bench::compare <name>`. The end-to-end command timings, which include
//! process start and output, come from `just bench::run`.
#![allow(clippy::unwrap_used)]

mod fixture;

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mindtape::{Filter, Readiness, default_order_with, load_config, load_project};

/// Backlog sizes the suite reports. 1000 is where the command starts to hurt.
/// 2000 is the size the docs quote.
const SIZES: [usize; 3] = [100, 1000, 2000];

/// A load with a warm cache: re-stat and re-hash every file, read each marker
/// from `.mindtape/cache/`, then annotate the set.
fn load_warm(c: &mut Criterion) {
    let mut group = c.benchmark_group("load/warm");
    for n in SIZES {
        let root = fixture::fixture(n);
        let config = load_config(&root).unwrap();
        // Prime the cache, so the measured run is the steady state.
        let _ = load_project(&root, &config);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(load_project(&root, &config)));
        });
    }
    group.finish();
}

/// A load with an empty cache: evaluate every file through Typst and write
/// every entry back. The cache clear is setup, so it stays out of the number.
fn load_cold(c: &mut Criterion) {
    let mut group = c.benchmark_group("load/cold");
    // A cold load is seconds, not microseconds: ten samples is plenty and
    // keeps the suite under a few minutes.
    group.sample_size(10);
    for n in [100, 1000] {
        let root = fixture::fixture(n);
        let config = load_config(&root).unwrap();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter_batched(
                || fixture::clear_cache(&root),
                |()| black_box(load_project(&root, &config)),
                BatchSize::PerIteration,
            );
        });
    }
    group.finish();
}

/// Parsing a filter expression, the cost every `mt ls <filter>` pays once.
fn filter_parse(c: &mut Criterion) {
    let expr =
        "!terminal && (miss(start) || start <= now()) && priority >= 3 && has_tag(\"bench\")";
    c.bench_function("filter/parse", |b| {
        b.iter(|| black_box(Filter::parse(black_box(expr)).unwrap()));
    });
}

/// The loaded set the query benches run over, and its config.
fn loaded(n: usize) -> (mindtape::Config, Vec<mindtape::Task>) {
    let root = fixture::fixture(n);
    let config = load_config(&root).unwrap();
    let outcome = load_project(&root, &config);
    (config, outcome.tasks)
}

/// Matching a parsed filter against every task: the per-task predicate, which
/// is what `mt ls` spends on the query itself.
fn filter_match(c: &mut Criterion) {
    let (config, tasks) = loaded(2000);
    let filter = Filter::parse("!terminal && priority >= 3 && has_tag(\"bench\")").unwrap();
    let readiness = Readiness::empty();
    c.bench_function("filter/match/2000", |b| {
        b.iter(|| {
            let mut kept = 0_usize;
            for task in &tasks {
                if filter.matches_with(task, &config, &readiness).unwrap() {
                    kept += 1;
                }
            }
            black_box(kept)
        });
    });
}

/// The default ordering over a loaded set. The clone is setup, so the number
/// is the sort alone.
fn sort_default(c: &mut Criterion) {
    let (config, tasks) = loaded(2000);
    let readiness = Readiness::empty();
    c.bench_function("sort/default/2000", |b| {
        b.iter_batched(
            || tasks.clone(),
            |mut tasks| {
                default_order_with(&mut tasks, &config, &readiness);
                black_box(tasks)
            },
            BatchSize::PerIteration,
        );
    });
}

criterion_group!(
    benches,
    load_warm,
    load_cold,
    filter_parse,
    filter_match,
    sort_default
);
criterion_main!(benches);
