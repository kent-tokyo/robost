# Input Operations

## type

Type a string via keyboard emulation.

```yaml
- type: "Hello, world!"

# Use a variable
- type: "{{ username }}"

# Read from environment variable (secrets)
- type:
    secret_env: MY_PASSWORD
```

## press

Press a single key.

```yaml
- press: Enter
- press: Tab
- press: Escape
- press: F5
- press: Delete
- press: BackSpace
```

Common key names: `Enter`, `Tab`, `Escape`, `Space`, `BackSpace`, `Delete`,
`Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`,
`F1`–`F12`, `Insert`.

## key_combo

Press a key combination.

```yaml
- key_combo:
    keys: [ctrl, c]       # Ctrl+C

- key_combo:
    keys: [ctrl, shift, s]  # Ctrl+Shift+S

- key_combo:
    keys: [alt, F4]
```

Modifier names: `ctrl`, `shift`, `alt`, `meta` (Win/Cmd).

## click_in_window

Click at a position relative to a window's top-left corner.

```yaml
- click_in_window:
    window: "Notepad"     # title_contains match
    x: 100
    y: 50
    action: left          # left | right | middle | double (default: left)
```
