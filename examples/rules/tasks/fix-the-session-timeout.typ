#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Fix the session timeout",
  status: wip(2026, 7, 2),
  kind: "bug",
  priority: 8,
  assignee: "ada",
  estimate: (hours: 4),
  x-acceptance: (
    "a session idle for 30 minutes signs out",
    "an active session never signs out",
  ),
)

= Summary

Sessions sign out after five minutes, whatever the idle setting says.

= Steps to reproduce

Set the idle timeout to 30 minutes, sign in, keep clicking through the
dashboard for six minutes, and watch the login page come back.

// TODO: link the support ticket once it has a number.
