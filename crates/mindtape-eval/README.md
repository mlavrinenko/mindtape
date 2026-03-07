# mindtape-eval

Typst evaluation and task extraction for [MindTape](https://github.com/mlavrinenko/mindtape).

This crate provides the core evaluation engine that parses `.typ` files using the
Typst compiler crates and extracts structured task data (checkboxes, due dates,
tags, priorities, and metadata).

## Usage

This crate is used internally by the `mindtape` CLI. See the
[main repository](https://github.com/mlavrinenko/mindtape) for end-user
documentation.
