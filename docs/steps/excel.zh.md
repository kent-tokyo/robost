# Excel

所有 Excel 步骤使用 `calamine` 进行读取，使用 `xlsxwriter` 进行写入。无需安装 Office。

## excel_read

将工作表读取为行列表。

```yaml
- excel_read:
    file: data.xlsx
    sheet: Sheet1
    has_header: true
    save_as: rows
```

每行是单元格值的列表。当 `has_header: true` 时，提供 `{ 列名: 值 }` 的映射。

## excel_write

将行写入新工作表或现有工作表。

```yaml
- excel_write:
    file: output.xlsx
    sheet: Results
    rows: "{{ result_rows }}"
```

## excel_add_sheet

向现有工作簿添加新工作表。

```yaml
- excel_add_sheet:
    file: report.xlsx
    sheet: Summary
```

## excel_delete_sheet

从工作簿中删除工作表。

```yaml
- excel_delete_sheet:
    file: report.xlsx
    sheet: TempData
```

## excel_rename_sheet

重命名工作表。

```yaml
- excel_rename_sheet:
    file: report.xlsx
    from: Sheet1
    to: "{{ today }}_Report"
```

## excel_cell_read

读取单个单元格的值。

```yaml
- excel_cell_read:
    file: data.xlsx
    sheet: Sheet1
    row: 1             # 从 1 开始
    col: 1             # 从 1 开始（A=1）
    save_as: cell_val
```

## excel_cell_write

向单个单元格写入值。

```yaml
- excel_cell_write:
    file: output.xlsx
    sheet: Sheet1
    row: 1
    col: 1
    value: "{{ result }}"
```
