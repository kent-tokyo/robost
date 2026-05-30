# 変数

## set

変数にリテラル値をセットします。

```yaml
- set:
    name: greeting
    value: "Hello!"

- set:
    name: count
    value: 0
```

## copy_var

ある変数の値を別の変数にコピーします。

```yaml
- copy_var:
    from: source_var
    to: destination_var
```

## get_datetime

現在の日時をフォーマットされた文字列として取得します。

```yaml
- get_datetime:
    format: "%Y%m%d"      # → "20241215"
    save_as: today

- get_datetime:
    format: "%Y-%m-%d %H:%M:%S"
    save_as: timestamp
```

フォーマットは [chrono strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) に従います。

## get_username

現在の OS ユーザー名を取得します。

```yaml
- get_username:
    save_as: user
```

## calc

算術式を評価します。

```yaml
- calc:
    expr: "price * quantity"
    save_as: total

- calc:
    expr: "index + 1"
    save_as: next_index
```

## increment

数値変数を 1 (またはカスタムステップ値) だけインクリメントします。

```yaml
- increment:
    name: counter

- increment:
    name: counter
    by: 5
```

## to_fullwidth / to_halfwidth

全角文字と半角文字を相互変換します (日本語テキストの処理)。

```yaml
- to_fullwidth:
    value: "{{ text }}"
    save_as: full

- to_halfwidth:
    value: "{{ text }}"
    save_as: half
```

## import_vars

CSV または Excel ファイルの行を変数として読み込みます。

```yaml
- import_vars:
    file: data.xlsx
    sheet: Sheet1         # 省略可、デフォルト: 最初のシート
    row: 0                # 0 始まりの行インデックス
```

各列の値は `__rows__` にリストとして格納されます。最初の行がヘッダーの場合は名前付き変数としても格納されます。

## save_vars / load_vars

変数を JSON ファイルに保存または復元します。

```yaml
- save_vars:
    file: state.json

- load_vars:
    file: state.json
```
