#!/usr/bin/env bash
# examples/large/generate.sh [N] — deterministically (re)generate N benchmark
# task files under examples/large/tasks/. Every varying value is derived from
# the file index, so re-running with the same N reproduces byte-identical
# output. Statuses stay non-terminal (proposed/accepted/wip) so strict-ICE
# priority (required = "non-terminal") is always satisfied, and roughly every
# third task (i > 1) gets a `depends-on` edge to the previous task, building a
# linear, acyclic dependency chain whose target always already exists.
set -eo pipefail

N="${1:-2000}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TASKS_DIR="${SCRIPT_DIR}/tasks"

rm -rf "${TASKS_DIR}"
mkdir -p "${TASKS_DIR}"

statuses=(proposed accepted wip)

for ((i = 1; i <= N; i++)); do
  stem=$(printf "task-%05d" "${i}")
  file="${TASKS_DIR}/${stem}.typ"

  impact=$(((i % 9) + 1))
  ease=$((((i * 3) % 9) + 1))
  # confidence in [0.5, 0.9], stepping by 0.05 across a 9-value cycle.
  conf_step=$((i % 9))
  confidence=$(awk -v s="${conf_step}" 'BEGIN { printf "%.2f", 0.5 + s * 0.05 }')

  status="${statuses[$((i % 3))]}"

  links_line=""
  if ((i > 1 && i % 3 == 0)); then
    prev=$(printf "task-%05d" "$((i - 1))")
    links_line="  links: depends-on(\"${prev}.typ\"),
"
  fi

  {
    printf '#import "@local/mindtape:0.2.0": *\n'
    printf '\n'
    printf '#show: task.with(\n'
    printf '  title: "Bench task %d",\n' "${i}"
    printf '  priority: ice(\n'
    printf '    impact: %d,\n' "${impact}"
    printf '    confidence: %s,\n' "${confidence}"
    printf '    ease: %d,\n' "${ease}"
    printf '  ),\n'
    printf '  tags: ("bench",),\n'
    printf '%s' "${links_line}"
    printf '  status: %s(2026, 7, 16),\n' "${status}"
    printf ')\n'
    printf '\n'
    printf '== Summary\n'
    printf '\n'
    printf 'Generated benchmark task %d. Synthetic fixture task for load-testing `mt ls` and `mt check` at scale. Task index %d of the deterministic bench batch.\n' "${i}" "${i}"
  } > "${file}"
done

echo "wrote ${N} task files to ${TASKS_DIR}"
