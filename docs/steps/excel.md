# Excel

All Excel steps use `calamine` for reading and `xlsxwriter` for writing. No Office installation required.

## excel_read

Read a sheet into a list of rows.

```yaml
- excel_read:
    file: data.xlsx
    sheet: Sheet1
    has_header: true
    save_as: rows
```

Each row is a list of cell values. With `has_header: true`, a map of `{ column_name: value }` is provided.

## excel_write

Write rows to a new or existing sheet.

```yaml
- excel_write:
    file: output.xlsx
    sheet: Results
    rows: "{{ result_rows }}"
```

## excel_add_sheet

Add a new sheet to an existing workbook.

```yaml
- excel_add_sheet:
    file: report.xlsx
    sheet: Summary
```

## excel_delete_sheet

Remove a sheet from a workbook.

```yaml
- excel_delete_sheet:
    file: report.xlsx
    sheet: TempData
```

## excel_rename_sheet

Rename a sheet.

```yaml
- excel_rename_sheet:
    file: report.xlsx
    from: Sheet1
    to: "{{ today }}_Report"
```

## excel_cell_read

Read a single cell value.

```yaml
- excel_cell_read:
    file: data.xlsx
    sheet: Sheet1
    row: 1             # 1-based
    col: 1             # 1-based (A=1)
    save_as: cell_val
```

## excel_cell_write

Write a value to a single cell.

```yaml
- excel_cell_write:
    file: output.xlsx
    sheet: Sheet1
    row: 1
    col: 1
    value: "{{ result }}"
```
