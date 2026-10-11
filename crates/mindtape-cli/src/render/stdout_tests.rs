//! Unit tests for the stdout writer's error classification.
//!
//! The behaviour that matters end to end — a pipeline whose reader exits does
//! not panic `mt` — is `tests/it/broken_pipe_e2e.rs`: it takes a real closed pipe,
//! which no in-process test can stage. What is pinned here is the one decision
//! that test cannot distinguish from luck, namely *which* write errors are
//! swallowed and which stay as loud as libstd left them.

use std::io::{Error, ErrorKind};
use std::sync::atomic::Ordering;

use super::{READER_GONE, handle};

#[test]
fn a_broken_pipe_is_recorded_and_not_raised() {
    let before = READER_GONE.swap(false, Ordering::Relaxed);
    handle(&Error::from(ErrorKind::BrokenPipe));
    let recorded = READER_GONE.load(Ordering::Relaxed);
    READER_GONE.store(before, Ordering::Relaxed);
    assert!(recorded, "a broken pipe must mute every later write");
}

#[test]
#[should_panic(expected = "failed printing to stdout")]
fn any_other_write_error_still_panics() {
    // A full disk under `mt ls > out.txt` is a real failure. Swallowing it
    // would report a clean run over truncated output.
    handle(&Error::from(ErrorKind::PermissionDenied));
}
