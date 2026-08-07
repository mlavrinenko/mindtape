#!/usr/bin/env bash
# fzf-mt-go.sh — fuzzy-pick a task from `mt ls --format path` and open it.
#
# `mt ls -i` (`--interactive`) stays a plain key-loop highlighter over the
# printed table — it is deliberately not a fuzzy finder, since fzf (or sk,
# fzy, gum, ...) already does that well. `--format path` is the seam: a bare,
# newline-delimited list of task paths and nothing else (no header, no
# colour, no hyperlinks), ready to hand to whichever picker you already use.
# This file ships the recipe, not a dependency on fzf specifically.
#
# Usage: source this file (e.g. from ~/.bashrc or ~/.zshrc), then:
#   mtf                 # every task, picked with fzf, opened with `mt go`
#   mtf 'ready'         # any filter expression `mt ls` accepts
#
# Swap `fzf` below for `sk`, `fzy`, `gum choose`, or anything else that reads
# newline-delimited stdin and writes the chosen line to stdout — the rest of
# the pipeline is unchanged.
mtf() {
    local filter="${1-}"
    local picked
    picked="$(mt ls "$filter" --format path | fzf)" || return
    [ -n "$picked" ] && mt go "$picked"
}
