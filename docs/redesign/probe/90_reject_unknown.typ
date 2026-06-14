// MUST FAIL: `contexts` at top level is neither a core slot nor an x-<ns>
// extension. MindTape rejects it.
#import "speconaut-stub.typ": *
#import "contexts-stub.typ": *
#show: task.with(
  title: "misplaced field",
  status: proposed(2026, 6, 13),
  priority: "normal",
  grade: "senior",
  estimate-hours: 8,
  contexts: (knowledge-context,),
)

== Summary
Stub body.
