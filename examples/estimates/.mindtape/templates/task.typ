#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot(
    "title",
    prompt: "What changes",
    example: "Cache the parsed config between commands",
  ),
  estimate: (
    files: slot(
      "files",
      kind: "int",
      prompt: "How many files the change touches",
      example: 3,
    ),
  ),
)
