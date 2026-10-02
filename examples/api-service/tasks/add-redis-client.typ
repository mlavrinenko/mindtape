#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Add a Redis client to the app state",
  kind: "feat",
  status: proposed(2026, 9, 25),
  estimate: (src-files: 3),
)

== Problem
+ Open `src/state.rs`.
+ 🔴 `AppState` holds a Postgres pool and nothing else. A handler that needs a counter shared by every replica has nowhere to keep it.

== TO-BE
- 🟢 Handlers reach a pooled Redis connection through `AppState`.

== Changes
- `Cargo.toml`: add the `redis` crate with its `tokio-comp` and `connection-manager` features.
- `src/config.rs`: read `REDIS_URL` from the environment.
- `src/state.rs`: open a `ConnectionManager` from `REDIS_URL` at startup and keep it in `AppState`.

== Alternatives
- Counters in Postgres. Every failed login would become a write to the primary database.

== Out of scope
- Caching responses in Redis. This task only connects to it.
