#import "@local/mindtape:0.4.0": *
#show: task.with(
  title: "Index the orders by customer",
  kind: "fix",
  priority: ice(impact: 9, confidence: 0.95, ease: 9),
  status: proposed(2026, 9, 29),
  estimate: (src-files: 1),
  tags: ("billing", "stateless"),
  due: on(2026, 10, 2),
)

== AS-IS
+ Run `curl localhost:8080/customers/4711/orders`.
+ 🔴 It takes 3 seconds, because Postgres scans every row of `orders`.

```sh
$ psql "$DATABASE_URL" -c "EXPLAIN ANALYZE SELECT * FROM orders WHERE customer_id = 4711" | tail -1
Seq Scan on orders  (actual time=0.020..2810.134 rows=12 loops=1)
```

== Environment
- Production, release `v1.8.2`, with 12 million orders.

== TO-BE
- 🟢 The same request returns in under 50 ms.

```sh
$ curl -w '%{time_total}s\n' localhost:8080/customers/4711/orders
0.012s
```

== Changes
- `migrations/0008_orders_customer_index.sql`, new: create the index with `CONCURRENTLY`, so writes keep flowing.

== Out of scope
- Paginating the orders list.
