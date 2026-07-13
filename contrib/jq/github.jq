# github.jq — map `gh issue list --json
# number,title,body,state,labels,assignees,createdAt,url` output (a JSON
# array of issues) into the canonical task-JSON schema `mt import` consumes
# (`crates/mindtape/src/canonical.rs`, SCHEMA_VERSION 1). Pure structural
# transform: config-blind, like taskwarrior.jq — no `.mindtape/config.toml`
# read here.
#
# Usage:
#   gh issue list --json number,title,body,state,labels,assignees,createdAt,url \
#     | jq -f contrib/jq/github.jq | mt import
#
# Computed and empty fields (`id`, `file`, `derived_tags`, …) default when
# omitted — the branches below still emit every key for readability, but a
# missing key no longer causes a parse failure.
#
# Mapping decisions:
#
# - status: GitHub's issue `state` is GraphQL-cased ("OPEN"/"CLOSED"); we
#   lower-case it before mapping, onto the SAME neutral canonical names
#   taskwarrior.jq uses so a project's status legend only has to accommodate
#   one vocabulary regardless of source:
#     open   -> "todo"
#     closed -> "done"
#   Any other/unrecognised state string passes through unchanged rather than
#   being guessed at — `mt import`'s report surfaces it as an out-of-legend
#   violation instead of silently mis-filing it.
# - status date: falls back to `createdAt` (the only timestamp the requested
#   `--json` field set carries — no `closedAt`/`updatedAt`). `mt import`'s
#   mapper stamps today's date on a name-only status, so omitting the date
#   is safe, but `createdAt` is supplied here for accuracy.
#   `createdAt` is an acknowledged approximation — it is also `start`
#   below, so a closed issue reads as if it closed the moment it opened. Add
#   `closedAt` to the `gh issue list --json` flags and this jq program for a
#   real "status changed on" date.
# - title/body: pass through verbatim. Unlike Taskwarrior's one-line
#   `description`, a GitHub issue body is normally already prose, so no
#   Summary-heading synthesis happens here — `mt import`'s own body-header
#   fallback (`crates/mindtape/src/import_map/body.rs`) still synthesizes a
#   `== Summary` from the title if the project requires one and this issue's
#   body happens to be empty.
# - labels: each label's `name` becomes a tag verbatim; the label's id/color
#   have no canonical home and are dropped.
# - createdAt: the date part (first 10 characters, already `YYYY-MM-DD` in
#   GitHub's ISO-8601 timestamp) becomes `start` — the closest canonical
#   field to "when this unit of work entered the tracker".
# - number/url/assignees: no canonical field fits an issue number, its
#   permalink, or its assignees' logins, so all three carry through
#   losslessly as the `x-gh` extra.
# - kind/difficulty/priority/estimate/links/due/reviewed have no GitHub
#   Issues equivalent (in this field set) and are left unset (`null`/empty).
# - `body` is spliced straight into `#show: task.with(...)`'s trailing Typst
#   source (`crates/mindtape/src/canonical.rs`'s doc: "raw Typst source"), so
#   a Markdown body's `#`-headings (or any literal `#`) would open Typst
#   code mode and fail to compile. `esc_typ` backslash-escapes `#` (and any
#   literal `\`, so the escape itself round-trips) before it lands in `body`
#   — the minimal safe-passthrough, not a Markdown-to-Typst rewrite.

def gh_status_map: {"open": "todo", "closed": "done"};

# Neutralise the two characters that break Typst compilation when spliced
# into markup verbatim: a literal `\` (escape char) doubles up so it still
# reads as one literal backslash, then `#` (opens code mode) gets escaped.
def esc_typ:
  (split("\\") | join("\\\\")) | (split("#") | join("\\#"));

def gh_date: if . then .[0:10] else null end;

def to_canonical:
  {
    id: "",
    file: "",
    title: (.title // ""),
    status: (
      if .state then
        {
          name: (gh_status_map[(.state | ascii_downcase)] // .state),
          date: (.createdAt | gh_date),
          note: null,
        }
      else null end
    ),
    priority: null,
    difficulty: null,
    kind: null,
    estimate: {},
    links: [],
    tags: [.labels[]?.name],
    derived_tags: [],
    due: null,
    start: (.createdAt | gh_date),
    reviewed: null,
    body: ((.body // "") | esc_typ),
    extras: {
      "x-gh": {
        number: (.number // null),
        url: (.url // null),
        assignees: [.assignees[]?.login],
      }
    },
  };

map(to_canonical)
