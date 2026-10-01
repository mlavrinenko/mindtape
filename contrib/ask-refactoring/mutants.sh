# ask-refactoring: the mutants arm — surviving mutants.
#
# Sourced by `contrib/ask-refactoring.sh`; not executable on its own. It reads
# that script's globals (`$repo`, `$MT`, `$WRITE`, the thresholds) and calls its
# `stem_for`. It owns three functions the driver looks up by name —
# `score_mutants`, `signal_mutants`, `emit_mutants` — reading the last sweep's
# outcomes.json.
#
# The reasoning behind this arm's numbers lives here with the code that
# applies them, deliberately: it is the most valuable thing in the arm and a
# doc it lived in instead would drift away from the constants it explains.

# ── signal: cargo-mutants ───────────────────────────────────────────────────
#
# A surviving mutant is a line the suite covers but does not pin — a bad test,
# stated precisely. It is the only one of the three signals whose producer this
# script will not run for you, and that asymmetry is the whole design of this
# arm.
#
# `just cover` re-runs in minutes and `just mmz cover` makes an unchanged tree
# free, so the coverage arm refreshes its own input and REFUSES a stale one. A
# full `cargo mutants` sweep is hours — it rebuilds and re-runs the suite once
# per mutant — so refreshing here is not an option and refusing would be worse:
# it would block the size and coverage arms behind an hours-long prerequisite
# for a signal that is deliberately not a gate. So this arm SKIPS instead, and
# says on stderr that it skipped and which recipe produces the data. What it
# must never do is stay quiet, because "no surviving mutants" and "nobody has
# measured" would then print identically, and the second one reads as good news.
#
# Staleness is the same call for the same reason. An outcomes.json a source has
# outrun can name a survivor that was pinned since, so the arm skips rather than
# files it — a phantom finding is the one thing the sweep cannot afford, and
# unlike the lcov there is no cheap way back to a fresh one.
#
# Scope comes out of the data, not a ledger. The outcomes.json this arm reads
# records every mutant that ran, so the set of files a sweep covered is
# derivable from the file itself — `--in-diff` and `-f` runs alike. That matters
# because a finding must never read as covering more than it does, and it means
# the honest answer needs no recorded last-sweep marker: a ledger would go
# stale, conflict between contributors, and be absent on a fresh clone, which is
# exactly the reasoning that kept the coverage arm off a stored baseline.
#
# Where to read it: `just mutants` (full) writes `mutants.out`, while a scoped
# run (`-f`, `--in-diff`) writes `target/mutants-scoped` so it cannot clobber
# the `--iterate` cache the full sweep reads (`.just/scripts/mutants.sh`). Both
# are complete sweep records, so the arm takes whichever ran last; a sweep is a
# sweep regardless of which directory the wrangler routed it to.
MUTANTS_JSON=""
for candidate in mutants.out/outcomes.json target/mutants-scoped/outcomes.json; do
    if [ -f "$candidate" ] && { [ -z "$MUTANTS_JSON" ] || [ "$candidate" -nt "$MUTANTS_JSON" ]; }; then
        MUTANTS_JSON="$candidate"
    fi
done

mutants_available() {
    if [ -z "$MUTANTS_JSON" ]; then
        echo "ask-refactoring: no outcomes.json under mutants.out or target/mutants-scoped — skipping the mutants arm" >&2
        echo "  (refresh with \`just mutants\` — hours — or \`just mutants -f FILE\`" >&2
        echo "   for each changed source; this arm never runs cargo-mutants itself)" >&2
        return 1
    fi
    local newer
    newer=$(find crates -name '*.rs' -newer "$MUTANTS_JSON" -print -quit)
    if [ -n "$newer" ]; then
        echo "ask-refactoring: $MUTANTS_JSON predates $newer — skipping the mutants arm" >&2
        echo "  (a survivor it names may have been pinned since; refresh it with" >&2
        echo "   \`just mutants\` — hours — or \`just mutants -f FILE\` per changed source)" >&2
        return 1
    fi
    return 0
}

# One sentence naming what the recorded sweep actually covered, derived from
# outcomes.json itself. Read on a task filed weeks later, this is what stops a
# single-file `-f` run from reading like a verdict on the workspace.
#
# The file list is capped at six names. It lands in the task's `Problem`
# section, whose word cap is 120, and a full-sweep outcomes.json holds a run
# across every file — spelling all of them out pushed the section over the cap
# and `mt add` refused the whole sweep with nothing filed. Six names carry the
# same "how wide was this" the full list did, and `and N more` keeps the count
# honest; the arm only ever filed cleanly for `--in-diff` runs before this.
mutants_scope() {
    jq -r '
      ([.outcomes[] | select(.scenario | type == "object") | .scenario.Mutant.file]
        | unique) as $files
      | (if ($files | length) <= 6 then ($files | join(", "))
         else (($files[0:6] | join(", ")) + ", and \(($files | length) - 6) more")
         end) as $listed
      | "That sweep ran \(.total_mutants) mutants across \($files | length) file(s) "
        + "(\($listed)) and left \(.missed) unpinned"
        + (if .timeout > 0 then ", with \(.timeout) timing out — a timeout is a
non-verdict, not a pass" else "" end)
        + "."
    ' "$MUTANTS_JSON" | tr '\n' ' '
}

# Scoring, and why these numbers:
#
#   impact = clamp(round(3 + 5 * share + (fn > 20 lines)), 1, 10)
#     `share` is how much of the function the mutation replaced: 1.0 when the
#     whole body went and the suite still passed, which says the function's
#     entire contract is unpinned rather than one decision inside it. A flipped
#     operator scores near the floor because it is a narrower claim — real, but
#     about one expression. The size bump is there because a longer function
#     decides more, so the same share buys more risk.
#
#   confidence = 0.8, constant
#     cargo-mutants RAN the suite, so survival is measured, not forecast. Lower
#     than the 0.9 the size and coverage arms carry because a survivor can be
#     provably unkillable — `.cargo/mutants.toml` documents two such equivalent
#     mutants today — so unlike a line count, the finding itself can turn out to
#     be a non-problem. That is a different uncertainty from "the remedy is a
#     judgment call", and it belongs in the number.
#
#   ease = clamp(9 - round(fn_lines / 10), 1, 10)
#     The mutant marks the line; what costs time is reaching it from a test. A
#     short function takes a direct assertion, a long one takes fixture setup
#     before the branch is even reachable — the template's "how far the mutant
#     sits from a seam", measured by the only proxy the data carries.
#
#   difficulty tracks the same length on the config's judgment rubric, and the
#   estimate is a flat `(src-files: 1)`: the test module gains a case, or
#   `.cargo/mutants.toml` gains an exclusion. Both remedies touch one file, so a
#   varying number here would be invented rather than measured.
score_mutants() {
    jq -r '
      def clamp($lo; $hi): [[., $lo] | max, $hi] | min;
      .outcomes[]
      | select(.summary == "MissedMutant")
      | .scenario.Mutant                                    as $m
      | ($m.span.end.line - $m.span.start.line + 1)         as $mutated
      | ($m.function.span.end.line - $m.function.span.start.line + 1) as $fn_lines
      | (if $fn_lines > 0 then ($mutated / $fn_lines) else 1 end | clamp(0; 1)) as $share
      | ((3 + 5 * $share + (if $fn_lines > 20 then 1 else 0 end)) | round | clamp(1; 10)) as $impact
      | ((9 - (($fn_lines / 10) | round)) | clamp(1; 10))   as $ease
      | (if $fn_lines <= 10 then 2 elif $fn_lines <= 30 then 4 else 6 end) as $difficulty
      # cargo-mutants own wording, taken from the mutant name with its
      # `file:line:col: ` prefix stripped, rather than the bare `replacement`
      # field. A deletion (`delete ! in confirm_retire`) has an EMPTY
      # replacement, so a task built from that field would read "the suite still
      # passed after ." — and this phrasing is what `mutants.out/missed.txt`
      # prints, so a filed task greps against what cargo-mutants itself wrote.
      # (No apostrophes in here: the jq program is single-quoted.)
      | ($m.name | sub("^[^ ]+:[0-9]+:[0-9]+: "; ""))       as $mutation
      | [
          $m.file,
          $m.function.function_name,
          ($m.span.start.line | tostring),
          $mutation,
          ($impact | tostring),
          "0.8",
          ($ease | tostring),
          ($difficulty | tostring),
          "\($m.span.start.line):\($m.span.start.column)"
        ]
      | @tsv
    ' "$MUTANTS_JSON"
}

signal_mutants() {
    mutants_available || return 0
    score_mutants
}

emit_mutants() {
    local path="$1" function="$2" line="$3" mutation="$4"
    local impact="$5" confidence="$6" ease="$7" difficulty="$8"
    local location="$9"

    # The stem carries the mutation's span as well as its description. The
    # description alone is not unique — three `replace > with >=` survivors in
    # `sweep_operator` share it — and `--dedup skip` then hides every sibling
    # after the first, which is the bug this scheme fixes. `stem_for` spells
    # the operators out, so the description half stays readable. A survivor
    # whose line shifts is refiled under a new stem; a killed one never
    # reappears, and a survivor that moved is a fresh finding either way.
    local stem
    stem="$(stem_for mutants "$path-$location-$mutation")"

    local args=(
        add "pin $function in $path"
        -t sweep-mutants
        --name "$stem"
        --set "path=$path"
        --set "function=$function"
        --set "line=$line"
        --set "mutation=$mutation"
        --set "scope=$(mutants_scope)"
        --set "impact=$impact"
        --set "confidence=$confidence"
        --set "ease=$ease"
        --set "difficulty=$difficulty"
        --set "estimate=(src-files: 1)"
        --set 'tags+=swept' --set 'tags+=mutants'
    )

    if [ "$WRITE" -eq 1 ]; then
        local out outcome dest
        out="$("$MT" "${args[@]}" --dedup skip --format json)"
        outcome="$(printf '%s' "$out" | jq -r '.outcome')"
        dest="$(printf '%s' "$out" | jq -r '.path')"
        case "$outcome" in
            wrote) echo "filed   $dest" ;;
            skipped)
                echo "skipped $dest" >&2
                echo "        the stem is taken, so this mutant was filed before and closed" >&2
                echo "        without being killed — reopen it, or add it to" >&2
                echo "        \`.cargo/mutants.toml\` \`exclude_re\` with its reason" >&2
                ;;
            *) echo "$outcome $dest" ;;
        esac
    else
        echo "── would file: $stem  [ice($impact, $confidence, $ease)]"
        "$MT" "${args[@]}" --print
        echo
    fi
}
