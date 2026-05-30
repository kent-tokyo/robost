# Database

## db_query

Execute a SELECT query and store results.

```yaml
- db_query:
    dsn: "sqlite://./data.db"
    sql: "SELECT * FROM orders WHERE status = 'pending'"
    save_as: rows         # list of row maps
```

Supported DSN prefixes: `sqlite://`, `mysql://`, `postgres://`.

## db_execute

Execute a non-SELECT SQL statement (INSERT, UPDATE, DELETE, etc.).

```yaml
- db_execute:
    dsn: "sqlite://./data.db"
    sql: "UPDATE orders SET status = 'done' WHERE id = ?"
    params:
      - "{{ order_id }}"
```
