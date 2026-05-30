# 脚本

## shell

运行操作系统 shell 命令并捕获其输出。

```yaml
- shell:
    cmd: python3
    args: ["script.py", "--input", "{{ file }}"]
    save_as: output       # 标准输出转为字符串
    timeout_ms: 30000
```

在 Windows 上，使用 `cmd` 或 `powershell`：

```yaml
- shell:
    cmd: powershell
    args: ["-Command", "Get-Date -Format yyyyMMdd"]
    save_as: today
```

## script

执行内联 [Rhai](https://rhai.rs/) 脚本。脚本可以读写场景变量并返回值。

```yaml
- script: |
    let y = now_year();
    let m = now_month();
    let eom = end_of_month_str(y, m, "%Y/%m/%d");
    eom
  save_as: end_of_month
```

### 内置 Rhai 函数

| 函数 | 返回值 | 说明 |
|------|--------|------|
| `now_year()` | `int` | 当前年份 |
| `now_month()` | `int` | 当前月份（1–12） |
| `now_day()` | `int` | 当前日期 |
| `today()` | `String` | `"YYYY-MM-DD"` |
| `end_of_month(year, month)` | `int` | 当月最后一天 |
| `end_of_month_str(year, month, fmt)` | `String` | 格式化的当月最后一天 |

### 在脚本中访问场景变量

```yaml
- set:
    name: base_price
    value: 1000

- script: |
    let tax = base_price * 0.1;
    base_price + tax
  save_as: total_price
```

## library

调用 WASM 插件函数。

```yaml
- library:
    name: excel-reader.read_sheet
    inputs:
      path: "data.xlsx"
      sheet: "Sheet1"
    save_as: rows
```

有关如何编写自己的插件，请参阅[插件开发](#)。
