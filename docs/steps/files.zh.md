# 文件

## file_copy

复制文件。

```yaml
- file_copy:
    src: input/report.xlsx
    dst: archive/report_{{ today }}.xlsx
```

## file_move

移动（重命名）文件。

```yaml
- file_move:
    src: temp/output.xlsx
    dst: done/output.xlsx
```

## file_delete

删除文件。

```yaml
- file_delete:
    path: temp/scratch.txt
```

## file_rename

在同一目录内重命名文件。

```yaml
- file_rename:
    path: report.xlsx
    name: "report_{{ today }}.xlsx"
```

## file_exists

检查文件是否存在并保存结果。

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

将文本文件读取到变量中。

```yaml
- file_read:
    path: config.txt
    save_as: config_text
    encoding: utf-8    # 可选，默认: utf-8
```

## file_write

将字符串写入文件。

```yaml
- file_write:
    path: output/log.txt
    content: "Run completed at {{ timestamp }}"
    append: false      # true 表示追加（默认: false）
```

## file_list

列出目录中的文件，可选按 glob 模式过滤。

```yaml
- file_list:
    dir: input/
    pattern: "*.xlsx"
    save_as: xlsx_files   # 文件路径列表
```
