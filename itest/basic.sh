#!/usr/bin/env bash

# Get two tasks with due and sorted by due too (inherited from sole filter).
output=$(mindtape res/piano.typ --due -2)
expected_output="- (due 2026-04-01) Learn 5 Hanon exercises
- (due 2026-05-02) Finish learning Lilium"
if [ "$output" != "$expected_output" ]; then
    echo "Test failed: Output does not match expected"
    exit 1
fi
