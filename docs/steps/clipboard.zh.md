# 剪贴板

## clipboard_set

向系统剪贴板写入值。

```yaml
- clipboard_set:
    value: "text to copy"

- clipboard_set:
    value: "{{ my_var }}"
```

## clipboard_get

将当前剪贴板内容读取到变量中。

```yaml
- clipboard_get:
    save_as: clip_text
```
