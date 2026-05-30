# 变量

## set

将变量设置为字面量值。

```yaml
- set:
    name: greeting
    value: "Hello!"

- set:
    name: count
    value: 0
```

## copy_var

将一个变量的值复制到另一个变量。

```yaml
- copy_var:
    from: source_var
    to: destination_var
```

## get_datetime

获取当前日期/时间的格式化字符串。

```yaml
- get_datetime:
    format: "%Y%m%d"      # → "20241215"
    save_as: today

- get_datetime:
    format: "%Y-%m-%d %H:%M:%S"
    save_as: timestamp
```

格式遵循 [chrono strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)。

## get_username

获取当前操作系统用户名。

```yaml
- get_username:
    save_as: user
```

## calc

计算算术表达式。

```yaml
- calc:
    expr: "price * quantity"
    save_as: total

- calc:
    expr: "index + 1"
    save_as: next_index
```

## increment

将数值变量递增 1（或自定义步长）。

```yaml
- increment:
    name: counter

- increment:
    name: counter
    by: 5
```

## to_fullwidth / to_halfwidth

在全角和半角字符之间转换（日文文本处理）。

```yaml
- to_fullwidth:
    value: "{{ text }}"
    save_as: full

- to_halfwidth:
    value: "{{ text }}"
    save_as: half
```

## import_vars

从 CSV 或 Excel 文件中加载一行数据作为变量。

```yaml
- import_vars:
    file: data.xlsx
    sheet: Sheet1         # 可选，默认: 第一个工作表
    row: 0                # 从 0 开始的行索引
```

每列的值存储为 `__rows__` 中的列表，或者若第一行为表头则存储为命名变量。

## save_vars / load_vars

将变量持久化到 JSON 文件，或从 JSON 文件恢复变量。

```yaml
- save_vars:
    file: state.json

- load_vars:
    file: state.json
```
