# 归档

## zip_create

从一个或多个文件创建 ZIP 归档。

```yaml
- zip_create:
    output: "archive_{{ today }}.zip"
    files:
      - output/report.xlsx
      - output/log.txt
```

## zip_extract

将 ZIP 归档解压到目录。

```yaml
- zip_extract:
    file: "archive.zip"
    dest: extracted/
```
