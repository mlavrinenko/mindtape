#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Replace the deprecated search client",
  kind: "feat",
  priority: ice(impact: 5, confidence: 0.5, ease: 3),
  status: proposed(2026, 8, 20),
  estimate: (src-files: 3),
  tags: ("search",),
  due: on(2026, 10, 20),
)

== Problem
+ Run `cargo build 2>&1 | grep elasticsearch`.
+ 🔴 It warns that the `elasticsearch` crate is deprecated, and the vendor ships fixes to its replacement only.

== TO-BE
- 🟢 Search runs on the vendor's supported client and the build prints no deprecation warning.

== Changes
- `Cargo.toml`: swap the `elasticsearch` crate for `opensearch`.
- `src/search/client.rs`: build the new client from `SEARCH_URL`.
- `src/search/query.rs`: port the query builders to the new types.

== Alternatives
- Call the HTTP API with `reqwest`. We would own every request and response type.

== Out of scope
- Changing the index mappings.
