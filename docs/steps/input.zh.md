# 输入操作

## type

通过键盘模拟输入字符串。

```yaml
- type: "Hello, world!"

# 使用变量
- type: "{{ username }}"

# 从环境变量读取（密钥）
- type:
    secret_env: MY_PASSWORD
```

## press

按下单个按键。

```yaml
- press: Enter
- press: Tab
- press: Escape
- press: F5
- press: Delete
- press: BackSpace
```

常用按键名称：`Enter`、`Tab`、`Escape`、`Space`、`BackSpace`、`Delete`、
`Up`、`Down`、`Left`、`Right`、`Home`、`End`、`PageUp`、`PageDown`、
`F1`–`F12`、`Insert`。

## key_combo

按下组合键。

```yaml
- key_combo:
    keys: [ctrl, c]       # Ctrl+C

- key_combo:
    keys: [ctrl, shift, s]  # Ctrl+Shift+S

- key_combo:
    keys: [alt, F4]
```

修饰键名称：`ctrl`、`shift`、`alt`、`meta`（Win/Cmd）。

## click_in_window

在窗口左上角的相对坐标处点击。

```yaml
- click_in_window:
    window: "Notepad"     # 标题包含匹配
    x: 100
    y: 50
    action: left          # left | right | middle | double（默认: left）
```
