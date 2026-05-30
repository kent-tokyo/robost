# ファイル

## file_copy

ファイルをコピーします。

```yaml
- file_copy:
    src: input/report.xlsx
    dst: archive/report_{{ today }}.xlsx
```

## file_move

ファイルを移動 (名前変更) します。

```yaml
- file_move:
    src: temp/output.xlsx
    dst: done/output.xlsx
```

## file_delete

ファイルを削除します。

```yaml
- file_delete:
    path: temp/scratch.txt
```

## file_rename

同じディレクトリ内でファイルの名前を変更します。

```yaml
- file_rename:
    path: report.xlsx
    name: "report_{{ today }}.xlsx"
```

## file_exists

ファイルが存在するかどうかを確認し、結果を保存します。

```yaml
- file_exists:
    path: output/result.xlsx
    save_as: file_found

- if:
    cond: "file_found == true"
  then:
    - type: "File is ready"
```

## file_read

テキストファイルの内容を変数に読み込みます。

```yaml
- file_read:
    path: config.txt
    save_as: config_text
    encoding: utf-8    # 省略可、デフォルト: utf-8
```

## file_write

文字列をファイルに書き込みます。

```yaml
- file_write:
    path: output/log.txt
    content: "Run completed at {{ timestamp }}"
    append: false      # true で追記モード (デフォルト: false)
```

## file_list

ディレクトリ内のファイルを一覧します。glob パターンで絞り込むことができます。

```yaml
- file_list:
    dir: input/
    pattern: "*.xlsx"
    save_as: xlsx_files   # ファイルパスのリスト
```
