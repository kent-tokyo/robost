# データベース

## db_query

SELECT クエリを実行して結果を保存します。

```yaml
- db_query:
    dsn: "sqlite://./data.db"
    sql: "SELECT * FROM orders WHERE status = 'pending'"
    save_as: rows         # 行マップのリスト
```

サポートされる DSN プレフィックス: `sqlite://`, `mysql://`, `postgres://`

## db_execute

SELECT 以外の SQL 文 (INSERT、UPDATE、DELETE など) を実行します。

```yaml
- db_execute:
    dsn: "sqlite://./data.db"
    sql: "UPDATE orders SET status = 'done' WHERE id = ?"
    params:
      - "{{ order_id }}"
```
