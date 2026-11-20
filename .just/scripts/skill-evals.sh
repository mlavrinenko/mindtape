#!/usr/bin/env bash
# .just/scripts/skill-evals.sh: the body of the `skill-evals` recipe. Runs from
# the repo root, like the recipe that invokes it.
#
# One eval, one arm, one sandbox. The harness copies the fixture outside the
# repo and installs the prelude into the sandbox home. It runs the executor
# with `HOME` and the four XDG dirs under the run directory. The executor
# cannot see a skill, session or checkout of this repo. A second sandbox grades
# the trace against the eval's assertions.
#
# The model and its key come from `.env`, which `.env.schema` documents and
# `.gitignore` keeps out of git. The harness refuses a placeholder before any
# run.
# Results land in target/skill-evals/<run-id>/eval-<n>/<arm>/.
set -eo pipefail

arm=${1:-both}
only=${2:-}
manifest=skills/mindtape/evals/evals.json

die() {
    echo "skill-evals: $*" >&2
    exit 1
}

fill="copy .env.schema to .env and fill in both values"

# The two variables, and nothing else: a line without `=` is a comment or a
# mistake, and neither reaches the run.
[ -f .env ] || die "no .env; $fill"
while IFS='=' read -r key value || [ -n "$key" ]; do
    case "$key" in
        '' | '#'*) continue ;;
    esac
    value=${value%$'\r'}
    case "$value" in
        \"*\") value=${value#\"}; value=${value%\"} ;;
        \'*\') value=${value#\'}; value=${value%\'} ;;
    esac
    export "$key=$value"
done <.env

model_id=${SKILL_EVALS_MODEL_ID:-}
api_key=${SKILL_EVALS_API_KEY:-}
[ -n "$model_id" ] || die "SKILL_EVALS_MODEL_ID is empty in .env"
[ "$model_id" != example-provider/example-model ] || die "SKILL_EVALS_MODEL_ID still holds the placeholder; $fill"
[ -n "$api_key" ] || die "SKILL_EVALS_API_KEY is empty in .env"
[ "$api_key" != replace-with-a-real-key ] || die "SKILL_EVALS_API_KEY still holds the placeholder; $fill"
case "$model_id" in
    */*) ;;
    *) die "SKILL_EVALS_MODEL_ID must be provider/model, not '$model_id'" ;;
esac
provider=${model_id%%/*}

command -v bwrap >/dev/null || die "no bwrap; run this in the dev shell (nix develop)"
command -v jq >/dev/null || die "no jq; run this in the dev shell (nix develop)"
opencode_bin=$(command -v opencode || true)
[ -n "$opencode_bin" ] || die "no opencode on PATH"
opencode_bin=$(readlink -f "$opencode_bin")

mt_bin=
for candidate in target/mt-dev/debug/mt target/debug/mt; do
    if [ -x "$candidate" ]; then
        mt_bin=$(readlink -f "$candidate")
        break
    fi
done
[ -n "$mt_bin" ] || mt_bin=$(command -v mt || true)
[ -n "$mt_bin" ] && [ -x "$mt_bin" ] || die "no mt; run just build-mt"

# The fixture's one clock, declared in www/capture.sh and read by the e2e test
# that pins the evals' facts. A run on the wall clock moves every due date.
mindtape_now=$(sed -n 's/^API_SERVICE_NOW=//p' www/capture.sh | head -n 1)
[ -n "$mindtape_now" ] || die "www/capture.sh declares no API_SERVICE_NOW"

mt_version=$("$mt_bin" --version | awk '{print $2}')
opencode_version=$("$opencode_bin" --version | sed 's/^opencode v//')
commit=$(git rev-parse --short HEAD)

run_id=$(date -u +%Y-%m-%dT%H-%MZ)
base=$run_id
n=1
while [ -e "target/skill-evals/$run_id" ]; do
    n=$((n + 1))
    run_id=$base-$n
done
results=target/skill-evals/$run_id
work_root=$(mktemp -d "${TMPDIR:-/tmp}/mindtape-skill-evals.XXXXXX")
trap 'rm -rf "$work_root"' EXIT

# The sandbox carries no host environment, no repo and no home. `/etc` comes
# along for the resolver, the certificates and the NixOS profiles, and a tmpfs
# masks the host keys. The network stays shared, because the executor calls a
# model.
sandbox() { # <run-dir> <work-dir> <command...>
    local run=$1 work=$2
    shift 2
    bwrap \
        --unshare-all --share-net --die-with-parent --new-session --clearenv \
        --proc /proc --dev /dev --tmpfs /tmp \
        --ro-bind /nix /nix \
        --ro-bind-try /usr /usr \
        --ro-bind-try /bin /bin \
        --ro-bind-try /lib /lib \
        --ro-bind-try /lib64 /lib64 \
        --ro-bind-try /etc /etc \
        --tmpfs /etc/ssh \
        --ro-bind-try /run/current-system /run/current-system \
        --bind "$run" "$run" \
        --ro-bind "$mt_bin" "$run/bin/mt" \
        --chdir "$work" \
        --setenv HOME "$run/home" \
        --setenv XDG_CONFIG_HOME "$run/home/.config" \
        --setenv XDG_DATA_HOME "$run/home/.local/share" \
        --setenv XDG_CACHE_HOME "$run/home/.cache" \
        --setenv XDG_STATE_HOME "$run/home/.local/state" \
        --setenv PATH "$run/bin:/etc/profiles/per-user/$(id -un)/bin:/run/current-system/sw/bin:/usr/local/bin:/usr/bin:/bin" \
        --setenv USER "$(id -un)" \
        --setenv LOGNAME "$(id -un)" \
        --setenv SHELL /bin/sh \
        --setenv TMPDIR /tmp \
        --setenv LANG C.UTF-8 \
        --setenv TERM dumb \
        --setenv MINDTAPE_NOW "$mindtape_now" \
        --setenv SKILL_EVALS_API_KEY "$api_key" \
        -- "$@"
}

# The fixture's `[[typst.local-packages]]` points `@local/mindtape` at this
# checkout's lib/, which a copy outside the repo cannot reach. Drop the table
# and the comment block that introduces it. The installed prelude resolves the
# import instead.
drop_local_packages() { # <config>
    awk '
        { line[NR] = $0 }
        END {
            start = 0
            for (i = 1; i <= NR; i++) {
                if (line[i] == "[[typst.local-packages]]") { start = i; break }
            }
            if (!start) { for (i = 1; i <= NR; i++) print line[i]; exit }
            first = start
            while (first > 1 && line[first - 1] ~ /^#/) first--
            last = start
            while (last < NR && line[last + 1] !~ /^\[/) last++
            n = 0
            for (i = 1; i <= NR; i++) {
                if (i < first || i > last) out[++n] = line[i]
            }
            while (n > 0 && out[n] == "") n--
            for (i = 1; i <= n; i++) print out[i]
        }
    ' "$1" >"$1.tmp"
    mv "$1.tmp" "$1"
}

# The sandbox's config specifies the provider from the model id and reads the
# key from the environment. No agent, permission, plugin or skill config comes
# along.
write_config() { # <path>
    jq -n --arg provider "$provider" '{
        "$schema": "https://opencode.ai/config.json",
        autoupdate: false,
        provider: { ($provider): { options: { apiKey: "{env:SKILL_EVALS_API_KEY}" } } }
    }' >"$1"
}

prepare() { # <run-dir> <fixture> <arm>
    local run=$1 fixture=$2 arm=$3
    [ -d "$fixture" ] || die "no fixture $fixture"
    mkdir -p "$run/home/.config/opencode" "$run/home/.local/share" \
        "$run/home/.cache" "$run/home/.local/state" "$run/bin" "$run/work"
    cp -a "$fixture/." "$run/work/"
    rm -rf "$run/work/.mindtape/cache"
    drop_local_packages "$run/work/.mindtape/config.toml"
    if ! env -i HOME="$run/home" \
        XDG_CONFIG_HOME="$run/home/.config" \
        XDG_DATA_HOME="$run/home/.local/share" \
        XDG_CACHE_HOME="$run/home/.cache" \
        XDG_STATE_HOME="$run/home/.local/state" \
        PATH="$run/bin:/usr/bin:/bin" \
        "$mt_bin" install --no-defaults --no-schema --no-completions \
        >"$run/install.log" 2>&1; then
        cat "$run/install.log" >&2
        die "mt install failed in $run"
    fi
    write_config "$run/home/.config/opencode/opencode.json"
    if [ "$arm" = with-skill ]; then
        mkdir -p "$run/home/.config/opencode/skills"
        cp -R skills/mindtape "$run/home/.config/opencode/skills/mindtape"
        # The assertions and the baseline are the answer key, not the skill.
        rm -rf "$run/home/.config/opencode/skills/mindtape/evals"
    fi
}

grader_prompt() {
    cat <<'EOF'
You grade one agent transcript against a list of assertions.

`trace.jsonl` is the transcript: one JSON event per line. A `text` event is what the agent said; a `tool_use` event is a command it ran and the output it got.

`assertions.json` is the list to grade, in order.

For each assertion, decide `pass` or `fail` from the transcript alone. Pass only when the transcript shows the assertion held. Do not assume a step the transcript does not show. Give one sentence of evidence in `reason`.

Write `verdict.json` in this directory, and nothing else:

{"assertions": [{"text": "<the assertion text, verbatim>", "pass": true, "reason": "<one sentence>"}]}

One entry per assertion, in the same order. Run no command.
EOF
}

run_grader() { # <run-dir>
    local run=$1 grade=$1/grade
    mkdir -p "$grade/home/.config/opencode" "$grade/home/.local/share" \
        "$grade/home/.cache" "$grade/home/.local/state" "$grade/bin" "$grade/work"
    cp "$run/trace.jsonl" "$grade/work/trace.jsonl"
    cp "$run/assertions.json" "$grade/work/assertions.json"
    write_config "$grade/home/.config/opencode/opencode.json"
    sandbox "$grade" "$grade/work" "$opencode_bin" run --standalone --format json \
        --model "$model_id" "$(grader_prompt)" >"$grade/grader.jsonl" 2>"$grade/grader.log"
}

write_run_json() { # <run> <id> <name> <arm> <fixture> <exit> <started> <finished>
    jq -n \
        --argjson eval "$2" --arg name "$3" --arg arm "$4" --arg fixture "$5" \
        --arg model "$model_id" --arg opencode "$opencode_version" \
        --arg mt "$mt_version" --arg now "$mindtape_now" --arg commit "$commit" \
        --argjson exit "$6" --arg started "$7" --arg finished "$8" \
        '{eval: $eval, name: $name, arm: $arm, fixture: $fixture, model: $model,
          opencode: $opencode, mt: $mt, mindtape_now: $now, commit: $commit,
          started: $started, finished: $finished, exit: $exit}' >"$1/run.json"
}

case "$arm" in
    both) arms=(no-docs with-skill) ;;
    no-docs | with-skill) arms=("$arm") ;;
    *) die "unknown arm '$arm'; use no-docs, with-skill or both" ;;
esac

if [ -n "$only" ]; then
    jq -e --argjson id "$only" 'any(.evals[]; .id == $id)' "$manifest" >/dev/null ||
        die "no eval $only; the manifest lists $(jq -r '[.evals[].id] | join(", ")' "$manifest")"
    ids=$only
else
    ids=$(jq -r '.evals[].id' "$manifest")
fi

failed=0
for id in $ids; do
    name=$(jq -r --argjson id "$id" '.evals[] | select(.id == $id) | .name' "$manifest")
    fixture=$(jq -r --argjson id "$id" '.evals[] | select(.id == $id) | .fixture' "$manifest")
    prompt=$(jq -r --argjson id "$id" '.evals[] | select(.id == $id) | .prompt' "$manifest")
    for a in "${arms[@]}"; do
        run=$work_root/eval-$id/$a
        prepare "$run" "$fixture" "$a"
        printf '%s\n' "$prompt" >"$run/prompt.md"
        if [ "$a" = with-skill ]; then
            jq --argjson id "$id" '.evals[] | select(.id == $id) | .assertions' "$manifest" >"$run/assertions.json"
        else
            jq --argjson id "$id" '.evals[] | select(.id == $id) | .without.assertions' "$manifest" >"$run/assertions.json"
        fi
        total=$(jq 'length' "$run/assertions.json")
        started=$(date -u +%Y-%m-%dT%H:%M:%SZ)
        rc=0
        sandbox "$run" "$run/work" "$opencode_bin" run --standalone --format json \
            --model "$model_id" "$prompt" >"$run/trace.jsonl" 2>"$run/executor.log" || rc=$?
        finished=$(date -u +%Y-%m-%dT%H:%M:%SZ)
        write_run_json "$run" "$id" "$name" "$a" "$fixture" "$rc" "$started" "$finished"
        verdict=error
        passed=0
        if [ "$rc" -eq 0 ]; then
            grc=0
            run_grader "$run" || grc=$?
            if [ "$grc" -eq 0 ] && jq -e --argjson n "$total" \
                '.assertions | length == $n and all(.[]; (.pass | type) == "boolean")' \
                "$run/grade/work/verdict.json" >/dev/null 2>&1; then
                cp "$run/grade/work/verdict.json" "$run/verdict.json"
                passed=$(jq '[.assertions[] | select(.pass)] | length' "$run/verdict.json")
                if [ "$passed" -eq "$total" ]; then verdict=pass; else verdict=fail; fi
            fi
        fi
        mkdir -p "$results/eval-$id/$a"
        cp "$run/run.json" "$run/trace.jsonl" "$run/executor.log" \
            "$run/prompt.md" "$run/assertions.json" "$results/eval-$id/$a/"
        [ -f "$run/verdict.json" ] && cp "$run/verdict.json" "$results/eval-$id/$a/"
        [ -f "$run/grade/grader.log" ] && cp "$run/grade/grader.log" "$results/eval-$id/$a/"
        if [ "$arm" = both ]; then
            printf 'eval %s  %-10s %-5s %s/%s\n' "$id" "$a" "$verdict" "$passed" "$total"
        else
            printf 'eval %s  %-5s %s/%s\n' "$id" "$verdict" "$passed" "$total"
        fi
        [ "$verdict" = pass ] || failed=1
    done
done

echo "run: $results"
[ "$failed" -eq 0 ]
