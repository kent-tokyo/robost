# 数据库

## db_query

执行 SELECT 查询并保存结果。

```yaml
- db_query:
    dsn: "sqlite://./data.db"
    sql: "SELECT * FROM orders WHERE status = 'pending'"
    save_as: rows         # 行映射的列表
```

支持的 DSN 前缀：`sqlite://`、`mysql://`、`postgres://`。

## db_execute

执行非 SELECT 的 SQL 语句（INSERT、UPDATE、DELETE 等）。

```yaml
- db_execute:
    dsn: "sqlite://./data.db"
    sql: "UPDATE orders SET status = 'done' WHERE id = ?"
    params:
      - "{{ order_id }}"
```
