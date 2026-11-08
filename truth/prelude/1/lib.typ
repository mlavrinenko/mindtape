// Generation 1 of this repo's task-facing prelude.
//
// A compatible change edits this file. An incompatible one adds
// `truth/prelude/2/lib.typ` and rewires the templates; a task that imports
// this path keeps generation 1 for its whole life.
//
// Do not import a mutable helper from here. A generation must keep evaluating
// the same way after `truth/lib.typ` changes.
#import "@local/mindtape:0.4.0": *

// A finding points at the task that surfaced it. The `surfaced-by` kind is
// declared in `.mindtape/config.toml`; this is the task-side spelling.
#let surfaced-by(..args) = edge("surfaced-by", ..args)
