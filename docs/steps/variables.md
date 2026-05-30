# Variables

## set

Set a variable to a literal value.

```yaml
- set:
    name: greeting
    value: "Hello!"

- set:
    name: count
    value: 0
```

## copy_var

Copy one variable's value to another.

```yaml
- copy_var:
    from: source_var
    to: destination_var
```

## get_datetime

Get the current date/time as a formatted string.

```yaml
- get_datetime:
    format: "%Y%m%d"      # → "20241215"
    save_as: today

- get_datetime:
    format: "%Y-%m-%d %H:%M:%S"
    save_as: timestamp
```

Format follows [chrono strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

## get_username

Get the current OS user name.

```yaml
- get_username:
    save_as: user
```

## calc

Evaluate an arithmetic expression.

```yaml
- calc:
    expr: "price * quantity"
    save_as: total

- calc:
    expr: "index + 1"
    save_as: next_index
```

## increment

Increment a numeric variable by 1 (or a custom step).

```yaml
- increment:
    name: counter

- increment:
    name: counter
    by: 5
```

## to_fullwidth / to_halfwidth

Convert between full-width and half-width characters (Japanese text handling).

```yaml
- to_fullwidth:
    value: "{{ text }}"
    save_as: full

- to_halfwidth:
    value: "{{ text }}"
    save_as: half
```

## import_vars

Load a row from a CSV or Excel file as variables.

```yaml
- import_vars:
    file: data.xlsx
    sheet: Sheet1         # optional, default: first sheet
    row: 0                # 0-based row index
```

Each column value is stored as a list in `__rows__`, or as named variables if the first row is a header.

## save_vars / load_vars

Persist variables to / restore from a JSON file.

```yaml
- save_vars:
    file: state.json

- load_vars:
    file: state.json
```
