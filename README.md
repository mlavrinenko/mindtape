<!-- Generated from docs/src/readme.typ by `just docs md`. Do not edit; edit the source. -->

<h1 align="center"><img src="https://raw.githubusercontent.com/mlavrinenko/mindtape/main/www/assets/images/logo.svg" alt="MindTape" width="120" /><br />
 MindTape</h1>

<h4 align="center">A task tracker in your repo that enforces your team's rules on every task and commit.</h4>

<h4 align="center"><a href="https://mlavrinenko.github.io/mindtape/">Documentation</a> · <a href="https://mlavrinenko.github.io/mindtape/quickstart/">Quickstart</a></h4>

---

Each task is a [Typst](https://typst.app/) file in your repository. `mt` checks every task and commit against the rules your team writes in `.mindtape/config.toml`, and refuses work that breaks one. It runs offline, on the files in your checkout.

[`examples/api-service/tasks/throttle-failed-logins-per-ip.typ`](https://github.com/mlavrinenko/mindtape/blob/main/examples/api-service/tasks/throttle-failed-logins-per-ip.typ)

```typ
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Throttle failed logins per IP",
  kind: "feat",
  priority: ice(impact: 7, confidence: 0.7, ease: 4),
  status: proposed(2026, 9, 28),
  estimate: (src-files: 2),
  links: depends-on("add-redis-client.typ"),
)

== Problem
+ Run `for i in $(seq 50); do curl -so /dev/null -w '%{http_code} ' -d user=ada -d password=x localhost:8080/login; done`.
+ 🔴 It prints `401` fifty times. A script can guess passwords as fast as the server answers.

== TO-BE
- 🟢 After 10 failed logins from one IP in 10 minutes, `/login` returns `429` with a `Retry-After` header.

== Changes
- `src/auth/limiter.rs`, new: count failures per IP in Redis, each count expiring after 10 minutes.
- `src/auth/login.rs`: ask the limiter before checking the password, and record each failure.

== Alternatives
- A counter in process memory. Each replica would count on its own, so three replicas would allow 30 tries.

== Out of scope
- Locking accounts by username, which needs its own unlock flow.
```

## What it solves

A few lines of config solve each problem below. Under each is the error `mt` prints when work breaks the rule, and the link shows the config and the whole session.

### [Tasks too big to finish in one change](https://mlavrinenko.github.io/mindtape/split-oversized-tasks/)

Cap how many source files a task may touch, so whoever files a big task has to split it first.

```
error: the task as filled:2: rules.split-big-work: split it into tasks of 5 source files or fewer
```

### [Tasks too vague to start](https://mlavrinenko.github.io/mindtape/require-a-task-shape/)

Refuse a task that does not say how to reproduce the problem, what done looks like and which files change.

```
Error: missing required slot(s): to-be, changes, out-of-scope
```

### [Tasks closed while the tests fail](https://mlavrinenko.github.io/mindtape/gate-done-on-checks/)

Run your test command when a task closes, and refuse to close it while that command fails.

```
error: reject-expired-sessions-on-token-refresh: gate "scripts/test" failed (exit 1); nothing written — fix it, or re-run with --force (the override is recorded in the status note)
```

### [Work started before the last task is done](https://mlavrinenko.github.io/mindtape/one-task-at-a-time/)

Allow one task in progress, and refuse to start a task before the tasks it depends on are done.

```
error: tasks/add-redis-client.typ:2: rules.one-wip: finish the task in progress before you start another
```

### [Commits nobody can trace to a task](https://mlavrinenko.github.io/mindtape/trace-commits-to-tasks/)

Refuse a commit that names no task, then ask which tasks changed a file before you edit it.

```
error: .git/COMMIT_EDITMSG: commits.rules.work-names-a-task: name the task this commit works on: `Refs: <stem>`
```

## Install

```sh
cargo install mindtape-cli
```

[Quickstart](https://mlavrinenko.github.io/mindtape/quickstart/) covers prebuilt binaries, `nix run` and a first project from `mt init` to `mt check`.

## Documentation

- **Start**: [Quickstart](https://mlavrinenko.github.io/mindtape/quickstart/) · [Comparison](https://mlavrinenko.github.io/mindtape/comparison/)
- **Model**: [Concepts](https://mlavrinenko.github.io/mindtape/concepts/)
- **Tasks**: [Read and write a task file](https://mlavrinenko.github.io/mindtape/task-files/) · [Task DSL](https://mlavrinenko.github.io/mindtape/dsl/) · [Task links](https://mlavrinenko.github.io/mindtape/links/) · [Recurring tasks](https://mlavrinenko.github.io/mindtape/recurring/) · [Templates & slots](https://mlavrinenko.github.io/mindtape/templates/) · [Derived tags](https://mlavrinenko.github.io/mindtape/derived-tags/)
- **Reference**: [CLI reference](https://mlavrinenko.github.io/mindtape/cli/) · [Filter language](https://mlavrinenko.github.io/mindtape/filters/) · [Priority](https://mlavrinenko.github.io/mindtape/priority/) · [Configuration](https://mlavrinenko.github.io/mindtape/config/) · [The status legend](https://mlavrinenko.github.io/mindtape/statuses/) · [Rules](https://mlavrinenko.github.io/mindtape/rules/) · [Rule packs](https://mlavrinenko.github.io/mindtape/rule-packs/) · [Commit rules](https://mlavrinenko.github.io/mindtape/commit-rules/) · [Named views and dictionaries](https://mlavrinenko.github.io/mindtape/named-sets/) · [Flip hooks](https://mlavrinenko.github.io/mindtape/hooks/) · [Estimate caps and working-tree budgets](https://mlavrinenko.github.io/mindtape/estimates/) · [Canonical schema](https://mlavrinenko.github.io/mindtape/schema/) · [Import & export](https://mlavrinenko.github.io/mindtape/import-export/)
- **Integrate**: [Self-validating tasks](https://mlavrinenko.github.io/mindtape/self-validating-tasks/) · [For AI agents](https://mlavrinenko.github.io/mindtape/agents/) · [Internals](https://mlavrinenko.github.io/mindtape/internals/)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE).
