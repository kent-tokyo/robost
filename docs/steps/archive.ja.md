# アーカイブ

## zip_create

1つ以上のファイルから ZIP アーカイブを作成します。

```yaml
- zip_create:
    output: "archive_{{ today }}.zip"
    files:
      - output/report.xlsx
      - output/log.txt
```

## zip_extract

ZIP アーカイブをディレクトリに展開します。

```yaml
- zip_extract:
    file: "archive.zip"
    dest: extracted/
```
