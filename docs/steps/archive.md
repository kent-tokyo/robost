# Archives

## zip_create

Create a ZIP archive from one or more files.

```yaml
- zip_create:
    output: "archive_{{ today }}.zip"
    files:
      - output/report.xlsx
      - output/log.txt
```

## zip_extract

Extract a ZIP archive to a directory.

```yaml
- zip_extract:
    file: "archive.zip"
    dest: extracted/
```
