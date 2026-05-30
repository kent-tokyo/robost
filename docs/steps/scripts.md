# Scripts

## shell

Run an OS shell command and capture its output.

```yaml
- shell:
    cmd: python3
    args: ["script.py", "--input", "{{ file }}"]
    save_as: output       # stdout as string
    timeout_ms: 30000
```

On Windows, use `cmd` or `powershell`:

```yaml
- shell:
    cmd: powershell
    args: ["-Command", "Get-Date -Format yyyyMMdd"]
    save_as: today
```

## script

Execute an inline [Rhai](https://rhai.rs/) script. The script can read/write scenario variables and return a value.

```yaml
- script: |
    let y = now_year();
    let m = now_month();
    let eom = end_of_month_str(y, m, "%Y/%m/%d");
    eom
  save_as: end_of_month
```

### Built-in Rhai functions

| Function | Return | Description |
|----------|--------|-------------|
| `now_year()` | `int` | Current year |
| `now_month()` | `int` | Current month (1–12) |
| `now_day()` | `int` | Current day |
| `today()` | `String` | `"YYYY-MM-DD"` |
| `end_of_month(year, month)` | `int` | Last day of month |
| `end_of_month_str(year, month, fmt)` | `String` | Last day formatted |

### Accessing scenario variables in scripts

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

Call a WASM plugin function.

```yaml
- library:
    name: excel-reader.read_sheet
    inputs:
      path: "data.xlsx"
      sheet: "Sheet1"
    save_as: rows
```

See [Plugin development](#) for how to write your own plugins.
