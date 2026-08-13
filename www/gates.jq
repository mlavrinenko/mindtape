# www/gates.jq — the jq program www/generate-facts.sh runs over `just --dump
# --dump-format json` (as $JUST_DUMP, piped in) to build gates.json. Split out
# of the shell script because it is long enough on its own to blow that
# script's linecop cap; kept as `jq -f`, the same external-program pattern
# `contrib/jq/*.jq` already uses for the import cookbook.
#
# Gate membership + row order both come from the Justfile: every gate recipe
# carries `[group("gate")]` (truth/tasks/tag-just-recipes-with-groups.typ),
# grouped together in the Justfile in the exact order this table should read.
# The JSON dump sorts recipes alphabetically and carries no source order, so
# `just --summary --unsorted` (source order, passed in as $order) is the
# ordering key instead.
#
# Still cross-checked against `check`'s own (mmz-unwrapped) dependency list —
# the old derivation got membership-equals-dependencies for free by
# construction, and losing that silently would let a recipe carry
# `[group("gate")]` without actually gating anything, or gate something with
# no `[group("gate")]` tag to show for it. Disagreement fails here, naming
# both sides of the mismatch, rather than downstream in check-doc-facts.
#
# Each gate also carries a derived `command`: the recipe's own `.body` when
# it is a single non-shebang line short enough to sit in a table cell, else
# `null` — docs/src/contributing.typ falls back to `[doc(...)]` then. Fixes
# the drift that started this task: CONTRIBUTING.md hand-quoted `cargo
# clippy --all-targets -D warnings` while the Justfile had grown
# `--workspace` and a `-q -- -D warnings` separator; reading `.body` back
# instead of retyping it makes that drift impossible.
#
# `.body` is structured (one array per line, of string or `{{ VAR }}`
# fragments — `[["variable","VAR"]]`); render_line renders both readably as
# `{{ VAR }}`, the Justfile's own syntax, instead of dumping raw JSON.

def render_frag:
  if type == "string" then .
  else map(if .[0] == "variable" then "{{ " + .[1] + " }}"
            else "{{ " + (.[0] // "expr") + " }}" end) | join("")
  end;
def render_line: map(render_frag) | join("");

# Quotable = no shebang, exactly one body line, and that line renders under
# 160 chars — comfortably above the longest one-line gate body today
# (docs-check, 74 chars) with headroom for a future recipe to still qualify.
def gate_command:
  if .shebang or (.body | length) != 1 then null
  else (.body[0] | render_line) as $line
    | if ($line | length) > 160 then null else $line end
  end;

([.recipes | to_entries[]
  | select(.value.attributes // [] | any(.group? == "gate"))
  | .key] | sort) as $gate_group
| ([.recipes.check.dependencies[]
    | if .recipe == "mmz" then .arguments[0] else .recipe end] | sort) as $check_deps
| if $gate_group != $check_deps then
    ("gate group and check dependencies disagree:\n"
      + "  only in [group(\"gate\")]: " + (($gate_group - $check_deps) | join(", ")) + "\n"
      + "  only in check dependencies: " + (($check_deps - $gate_group) | join(", ")))
    | halt_error(1)
  else
    ($order | map(select(. as $n | $gate_group | index($n) != null))) as $names
    | {gates: [$names[] as $n
        | .recipes[$n] as $r
        | {name: $n, doc: $r.doc, command: ($r | gate_command)}]}
  end
