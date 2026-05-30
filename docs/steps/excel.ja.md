# Excel

すべての Excel ステップは読み込みに `calamine`、書き込みに `xlsxwriter` を使用します。Office のインストールは不要です。

## excel_read

シートを行のリストとして読み込みます。

```yaml
- excel_read:
    file: data.xlsx
    sheet: Sheet1
    has_header: true
    save_as: rows
```

各行はセル値のリストです。`has_header: true` を指定すると `{ 列名: 値 }` 形式のマップが提供されます。

## excel_write

新規または既存のシートに行を書き込みます。

```yaml
- excel_write:
    file: output.xlsx
    sheet: Results
    rows: "{{ result_rows }}"
```

## excel_add_sheet

既存のワークブックに新しいシートを追加します。

```yaml
- excel_add_sheet:
    file: report.xlsx
    sheet: Summary
```

## excel_delete_sheet

ワークブックからシートを削除します。

```yaml
- excel_delete_sheet:
    file: report.xlsx
    sheet: TempData
```

## excel_rename_sheet

シートの名前を変更します。

```yaml
- excel_rename_sheet:
    file: report.xlsx
    from: Sheet1
    to: "{{ today }}_Report"
```

## excel_cell_read

単一セルの値を読み取ります。

```yaml
- excel_cell_read:
    file: data.xlsx
    sheet: Sheet1
    row: 1             # 1 始まり
    col: 1             # 1 始まり (A=1)
    save_as: cell_val
```

## excel_cell_write

単一セルに値を書き込みます。

```yaml
- excel_cell_write:
    file: output.xlsx
    sheet: Sheet1
    row: 1
    col: 1
    value: "{{ result }}"
```
