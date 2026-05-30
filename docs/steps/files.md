# Files

## file_copy

Copy a file.

```yaml
- file_copy:
    src: input/report.xlsx
    dst: archive/report_{{ today }}.xlsx
```

## file_move

Move (rename) a file.

```yaml
- file_move:
    src: temp/output.xlsx
    dst: done/output.xlsx
```

## file_delete

Delete a file.

```yaml
- file_delete:
    path: temp/scratch.txt
```

## file_rename

Rename a file within the same directory.

```yaml
- file_rename:
    path: report.xlsx
    name: "report_{{ today }}.xlsx"
```

## file_exists

Check whether a file exists and store the result.

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

Read a text file into a variable.

```yaml
- file_read:
    path: config.txt
    save_as: config_text
    encoding: utf-8    # optional, default: utf-8
```

## file_write

Write a string to a file.

```yaml
- file_write:
    path: output/log.txt
    content: "Run completed at {{ timestamp }}"
    append: false      # true to append (default: false)
```

## file_list

List files in a directory, optionally filtered by glob pattern.

```yaml
- file_list:
    dir: input/
    pattern: "*.xlsx"
    save_as: xlsx_files   # list of file paths
```
