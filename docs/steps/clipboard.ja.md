# クリップボード

## clipboard_set

システムクリップボードに値を書き込みます。

```yaml
- clipboard_set:
    value: "text to copy"

- clipboard_set:
    value: "{{ my_var }}"
```

## clipboard_get

現在のクリップボードの内容を変数に読み込みます。

```yaml
- clipboard_get:
    save_as: clip_text
```
