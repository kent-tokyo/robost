# シナリオ形式

シナリオは `name`、省略可能な `variables`、そして `steps` シーケンスを持つ YAML ファイルです。

```yaml
name: my_scenario

variables:
  server: "192.168.1.1"
  user: "admin"

steps:
  - wait_image:
      template: login.png
      timeout_ms: 5000
  - type: "{{ user }}"
  - press: Tab
  - type:
      secret_env: MY_PASSWORD
  - press: Enter
```

## トップレベルキー

| キー | 必須 | 説明 |
|-----|----------|-------------|
| `name` | 必須 | シナリオの識別子 |
| `variables` | 省略可 | `{{ name }}` として参照できるキーと値のペア |
| `steps` | 必須 | ステップノードの順序付きリスト |

## 変数の展開

文字列値のどこにでも `{{ var_name }}` を使用できます:

```yaml
- type: "Hello {{ username }}!"
- file_copy:
    src: "{{ base_dir }}/input.xlsx"
    dst: "{{ base_dir }}/output.xlsx"
```

## シークレット

パスワードを YAML ファイルに直接書かないでください。環境変数を使用します:

```yaml
- type:
    secret_env: MY_PASSWORD    # 実行時に $MY_PASSWORD を読み込む
```

## ステップ結果の保存

ほとんどのステップは `save_as` を指定して結果を変数に保存できます:

```yaml
- find_image:
    template: button.png
    save_as: btn_pos        # { x, y } 座標を保存

- get_datetime:
    format: "%Y%m%d"
    save_as: today          # 例: "20241215" を保存
```
