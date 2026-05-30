# 入力操作

## type

キーボードエミュレーションで文字列を入力します。

```yaml
- type: "Hello, world!"

# 変数を使用する
- type: "{{ username }}"

# 環境変数から読み込む (シークレット用)
- type:
    secret_env: MY_PASSWORD
```

## press

単一キーを押します。

```yaml
- press: Enter
- press: Tab
- press: Escape
- press: F5
- press: Delete
- press: BackSpace
```

よく使うキー名: `Enter`, `Tab`, `Escape`, `Space`, `BackSpace`, `Delete`,
`Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`,
`F1`〜`F12`, `Insert`

## key_combo

キーの組み合わせを押します。

```yaml
- key_combo:
    keys: [ctrl, c]       # Ctrl+C

- key_combo:
    keys: [ctrl, shift, s]  # Ctrl+Shift+S

- key_combo:
    keys: [alt, F4]
```

修飾キー名: `ctrl`, `shift`, `alt`, `meta` (Win/Cmd)

## click_in_window

ウィンドウの左上を基準とした相対位置でクリックします。

```yaml
- click_in_window:
    window: "Notepad"     # title_contains でマッチング
    x: 100
    y: 50
    action: left          # left | right | middle | double (デフォルト: left)
```
