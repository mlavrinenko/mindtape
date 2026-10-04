#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot(
    "title",
    prompt: "What changes",
    example: "Fix the refund rounding",
  ),
  x-team: slot("team", prompt: "A team from @teams", example: "payments"),
)
