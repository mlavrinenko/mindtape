// MUST FAIL: `contxts` is a typo'd speconaut field; task-extras' closed
// allowlist rejects it (the validation speconaut owns, not MindTape).
#import "speconaut-stub.typ": *
#import "contexts-stub.typ": *
#show: task.with(
  title: "typo in extras",
  status: proposed(2026, 6, 13),
  priority: "normal",
  grade: "senior",
  estimate-hours: 8,
  x-speconaut: task-extras(
    contxts: (knowledge-context,),
  ),
)

== Summary
Stub body.
