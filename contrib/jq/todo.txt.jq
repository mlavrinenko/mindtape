# todo.txt.jq — map a todo.txt file (https://github.com/todotxt/todo.txt,
# read as a JSON array of its non-empty lines) into the canonical task-JSON
# schema `mt import` consumes (`crates/mindtape/src/canonical.rs`,
# SCHEMA_VERSION 1). Pure structural transform: config-blind, like
# taskwarrior.jq and github.jq — no `.mindtape/config.toml` read here.
#
# todo.txt is a line format, not JSON, so this filter's INPUT is not the raw
# file: split it into lines first — the same `jq -R -s` pass
# `www/content/import-export.typ` shows inline.
#
# Usage:
#   jq -R -s 'split("\n") | map(select(length > 0))' todo.txt \
#     | jq -f contrib/jq/todo.txt.jq | mt import
#
# Computed and empty fields (`id`, `file`, `derived_tags`, …) default when
# omitted — the branches below still emit every key for readability, but a
# missing key no longer causes a parse failure.
#
# A todo.txt line, per the format spec, reads:
#   x 2026-06-03 (A) 2026-06-01 Call Mom +Family @iPhone due:2026-06-05
#   [x completion-date] [(priority)] [creation-date] description...
# every bracketed part optional, in that order; per spec a completion date is
# only ever present alongside the `x ` marker.
#
# Mapping decisions:
#
# - status: an `x ` prefix means "done"; its absence means "proposed", the
#   open status of the legend `mt init` writes by default — todo.txt
#   has no third state. A cancelled todo.txt task is conventionally just
#   deleted from the file, so this filter never emits "cancelled" (unlike
#   taskwarrior.jq/github.jq, which both have a real source state for it).
# - status date: the completion date when done, falling back to the creation
#   date when the line carries no completion date (the same best-available-
#   proxy reasoning taskwarrior.jq documents for its own fallback chain);
#   the creation date when not done. A line with neither date emits a null
#   status date.
# - priority: todo.txt's `(A)`-`(Z)` marker maps onto the SAME three-band
#   0..9 scale taskwarrior.jq's H/M/L does — A/B/C -> 8/5/2 — since A/B/C is
#   the common subset every todo.txt tool agrees on. D-Z (rare in practice)
#   pass through unmapped as `null` rather than guessing a 26-point scale,
#   but still carry losslessly in the `x-todotxt` extra below.
# - due: a `due:YYYY-MM-DD` key:value token (the one extension every
#   todo.txt tool recognises) is lifted into the canonical `due` field and
#   removed from the title; any other `key:value` token is left in the title
#   untouched — this filter only understands `due`.
# - title/body: the description, with the `x `/date/priority prefix,
#   `+project` tags and the recognised `due:` token stripped, becomes
#   `title`. Like Taskwarrior's `description`, a todo.txt line is a
#   one-liner, not prose, so it is *also* folded into `body` as a `==
#   Summary` section, mirroring taskwarrior.jq's reasoning exactly. A
#   `@context` marker is left verbatim in the title — todo.txt's own
#   convention for where it displays — but also lifted into the `x-todotxt`
#   extra below.
# - tags: `+project` words are lifted into `tags`, stripped from the title,
#   the same as taskwarrior.jq folds Taskwarrior's `project` field in.
# - the raw priority letter (kept even when it mapped, for lossless
#   round-trip — taskwarrior.jq keeps its raw `status` string the same way),
#   `@context` words, and any `key:value` token besides `due` carry through
#   losslessly as the `x-todotxt` extra — none of them have a canonical
#   field to land in.
# - kind/difficulty/estimate/links/reviewed have no todo.txt equivalent and
#   are left unset (`null`/empty).
# - `body` is spliced straight into `#show: task.with(...)`'s trailing Typst
#   source (`crates/mindtape/src/canonical.rs`'s doc: "raw Typst source"), so
#   a description containing a bare `#` (e.g. "fix issue #123") would open
#   Typst code mode and fail to compile. `esc_typ` backslash-escapes `#`
#   (and any literal `\`, so the escape itself round-trips) before it lands
#   in `body` — the minimal safe-passthrough, not a markdown-to-Typst
#   rewrite, mirroring taskwarrior.jq/github.jq exactly.

def todotxt_priority_map: {"A": 8, "B": 5, "C": 2};

# Neutralise the two characters that break Typst compilation when spliced
# into markup verbatim: a literal `\` (escape char) doubles up so it still
# reads as one literal backslash, then `#` (opens code mode) gets escaped.
def esc_typ:
  (split("\\") | join("\\\\")) | (split("#") | join("\\#"));

def trimmed_date(text):
  (text // "") | rtrimstr(" ") | if . == "" then null else . end;

# Split one todo.txt line into its structural parts. The "completed" capture
# is the completion date ONLY when the line also carries the `x ` marker;
# otherwise it is really the creation date (the one date slot a not-done
# line can have before its description), reassigned below.
def parse_todotxt:
  capture(
    "^(?<done>x )?(?<completed>\\d{4}-\\d{2}-\\d{2} )?(?:\\((?<priority>[A-Z])\\) )?(?<created>\\d{4}-\\d{2}-\\d{2} )?(?<desc>.*)$"
  ) as $m
  | ($m.done != null) as $done
  | {
      done: $done,
      priority: $m.priority,
      completed: (if $done then trimmed_date($m.completed) else null end),
      created: (
        if $done
        then trimmed_date($m.created)
        else trimmed_date($m.created // $m.completed)
        end
      ),
      desc: $m.desc,
    };

# `+project` tags: lifted into `tags`, stripped from the title.
def desc_tags: [scan("\\+\\S+")] | map(ltrimstr("+"));

# `@context` markers: lifted into the `x-todotxt` extra; left verbatim in
# the title (see the mapping-decision comment above).
def desc_contexts: [scan("@\\S+")] | map(ltrimstr("@"));

# `due:YYYY-MM-DD`: the one `key:value` token this filter understands.
def desc_due:
  (capture("(?:^|\\s)due:(?<date>\\d{4}-\\d{2}-\\d{2})(?:\\s|$)"; "g")? | .date) // null;

# Every other `key:value` token, `due:` excluded (it has its own canonical
# field above) — carried losslessly, since none has a canonical home.
def desc_other_kv:
  [scan("(?:^|\\s)([A-Za-z][\\w-]*):(\\S+)")]
  | map(select(.[0] != "due"))
  | map({(.[0]): .[1]});

def desc_title:
  gsub("\\+\\S+"; "")
  | gsub("(?:^|(?<=\\s))due:\\d{4}-\\d{2}-\\d{2}(?=\\s|$)"; "")
  | gsub("\\s+"; " ")
  | ltrimstr(" ")
  | rtrimstr(" ");

def to_canonical:
  parse_todotxt as $t
  | ($t.desc | desc_tags) as $tags
  | ($t.desc | desc_contexts) as $contexts
  | ($t.desc | desc_due) as $due
  | ($t.desc | desc_other_kv) as $other_kv
  | ($t.desc | desc_title) as $title
  | {
      id: "",
      file: "",
      title: $title,
      status: {
        name: (if $t.done then "done" else "proposed" end),
        date: (if $t.done then ($t.completed // $t.created) else $t.created end),
        note: null,
      },
      priority: (if $t.priority then (todotxt_priority_map[$t.priority] // null) else null end),
      difficulty: null,
      kind: null,
      estimate: {},
      links: [],
      tags: $tags,
      derived_tags: [],
      due: $due,
      start: null,
      reviewed: null,
      body: ("== Summary\n\n" + ($title | esc_typ)),
      extras: {
        "x-todotxt": {
          priority: $t.priority,
          contexts: $contexts,
          kv: (reduce $other_kv[] as $pair ({}; . + $pair)),
        }
      },
    };

map(to_canonical)
