# taskwarrior.jq — map Taskwarrior's `task export` JSON (an array of task
# records) into the canonical task-JSON schema `mt import` consumes
# (`crates/mindtape/src/canonical.rs`, SCHEMA_VERSION 1). Pure structural
# transform: config-blind (no `.mindtape/config.toml` read here — `mt import`
# applies the per-project policy map: status legend membership, tag
# allow-list, priority scale, ...).
#
# Usage:
#   task export | jq -f contrib/jq/taskwarrior.jq | mt import
#
# Every field CanonicalTask declares must be present in the output (even when
# null/empty — the schema has no `#[serde(default)]`), so every branch below
# emits its key even when the source has nothing to offer it.
#
# Mapping decisions:
#
# - status: Taskwarrior's three lifecycle states map onto neutral canonical
#   names, shared with github.jq so a project's status legend only has to
#   accommodate one vocabulary regardless of source:
#     pending   -> "todo"      (open work)
#     completed -> "done"
#     deleted   -> "cancelled"
#   Any other/unrecognised status string (e.g. a future Taskwarrior spelling)
#   passes through unchanged rather than being guessed at — `mt import`'s
#   report surfaces it as an out-of-legend violation instead of silently
#   mis-filing it under an existing status.
# - status date: Taskwarrior has no single "status changed" timestamp, so we
#   pick the best available proxy — `end` (the close timestamp) for a
#   completed/deleted task, else `modified`, else `entry`.
# - priority: Taskwarrior's H/M/L maps onto MindTape's default 0..9 priority
#   scale — H -> 8, M -> 5, L -> 2 (roughly the high/mid/low band of the
#   project's own default 3-band split). A task with no `priority` set emits
#   `"priority": null` rather than guessing a default.
# - title/body: `description` becomes `title` verbatim, and is *also* folded
#   into `body` as a `== Summary` section (Taskwarrior's `description` is a
#   one-liner, not prose, so there is nothing else to put under a Summary
#   heading) — this way an imported task satisfies a project's "every task
#   has a Summary" body-header rule out of the box, mirroring the shape
#   `crates/mindtape/src/import_map/body.rs`'s own synthesis fallback
#   produces, just filled with the real description instead of the title.
# - tags: `project` (if set) becomes one tag, verbatim — Taskwarrior's dotted
#   project hierarchy (e.g. "work.backend") has no clean 1:1 with MindTape's
#   flat tag set, so it is kept as a single tag string rather than split —
#   unioned with Taskwarrior's own `tags` array.
# - due: Taskwarrior's ISO-8601 basic UTC timestamp (`YYYYMMDDTHHMMSSZ`)
#   truncates to the date-only `YYYY-MM-DD` the canonical schema expects;
#   time-of-day has no canonical home.
# - uuid/id/urgency/annotations/depends/the raw status string all carry
#   through losslessly as the `x-tw` extra — none of them have a canonical
#   field to land in, and dropping them would lose real Taskwarrior context.
# - kind/difficulty/estimate/links/start/reviewed have no Taskwarrior
#   equivalent and are left unset (`null`/empty).
# - `body` is spliced straight into `#show: task.with(...)`'s trailing Typst
#   source (`crates/mindtape/src/canonical.rs`'s doc: "raw Typst source"), so
#   a `description` containing a bare `#` (e.g. "fix issue #123") would open
#   Typst code mode and fail to compile. `esc_typ` backslash-escapes `#` (and
#   any literal `\`, so the escape itself round-trips) before it lands in
#   `body` — the minimal safe-passthrough, not a markdown-to-Typst rewrite.

def tw_status_map: {"pending": "todo", "completed": "done", "deleted": "cancelled"};
def tw_priority_map: {"H": 8, "M": 5, "L": 2};

# Neutralise the two characters that break Typst compilation when spliced
# into markup verbatim: a literal `\` (escape char) doubles up so it still
# reads as one literal backslash, then `#` (opens code mode) gets escaped.
def esc_typ:
  (split("\\") | join("\\\\")) | (split("#") | join("\\#"));

# Taskwarrior's basic ISO-8601 UTC stamp ("YYYYMMDDTHHMMSSZ") -> "YYYY-MM-DD".
def tw_date:
  if . == null or (. | length) < 8 then null
  else .[0:4] + "-" + .[4:6] + "-" + .[6:8]
  end;

def tw_status_date:
  if .status == "completed" or .status == "deleted"
  then (.end // .modified // .entry)
  else (.modified // .entry)
  end | tw_date;

def to_canonical:
  {
    id: "",
    file: "",
    title: (.description // ""),
    status: (
      if .status then
        {
          name: (tw_status_map[.status] // .status),
          date: tw_status_date,
          note: null,
        }
      else null end
    ),
    priority: (if .priority then (tw_priority_map[.priority] // null) else null end),
    difficulty: null,
    kind: null,
    estimate: {},
    links: [],
    tags: ((if .project then [.project] else [] end) + (.tags // [])),
    derived_tags: [],
    due: (.due | tw_date),
    start: null,
    reviewed: null,
    body: ("== Summary\n\n" + ((.description // "") | esc_typ)),
    extras: {
      "x-tw": {
        uuid: (.uuid // null),
        id: (.id // null),
        urgency: (.urgency // null),
        status: (.status // null),
        annotations: (.annotations // []),
        depends: (.depends // null),
      }
    },
  };

map(to_canonical)
