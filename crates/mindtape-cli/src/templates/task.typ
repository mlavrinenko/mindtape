#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Task title", example: "Write the first task"),
)
