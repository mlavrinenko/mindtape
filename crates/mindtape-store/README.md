# mindtape-store

Store trait and SQLite backend for [MindTape](https://github.com/mlavrinenko/mindtape).

This crate provides the `Store` trait that abstracts task persistence, plus a
full SQLite implementation that indexes tasks extracted by `mindtape-eval` into
a queryable database.

## Usage

This crate is used internally by the `mindtape` CLI. See the
[main repository](https://github.com/mlavrinenko/mindtape) for end-user
documentation.
