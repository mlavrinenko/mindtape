#import "@local/mindtape:0.4.0": *

#show: task.with(
  title: "Pick a session store",
  status: done(2026, 9, 18)[Redis, for its key expiry],
  priority: 5,
  estimate: (hours: 3),
  tags: ("backend",),
)

Compare Redis and Postgres for session storage.
