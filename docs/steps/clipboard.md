# Clipboard

## clipboard_set

Write a value to the system clipboard.

```yaml
- clipboard_set:
    value: "text to copy"

- clipboard_set:
    value: "{{ my_var }}"
```

## clipboard_get

Read the current clipboard contents into a variable.

```yaml
- clipboard_get:
    save_as: clip_text
```
