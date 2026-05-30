# スクリプト

## shell

OS のシェルコマンドを実行し、その出力をキャプチャします。

```yaml
- shell:
    cmd: python3
    args: ["script.py", "--input", "{{ file }}"]
    save_as: output       # 標準出力を文字列として保存
    timeout_ms: 30000
```

Windows では `cmd` または `powershell` を使用します:

```yaml
- shell:
    cmd: powershell
    args: ["-Command", "Get-Date -Format yyyyMMdd"]
    save_as: today
```

## script

インライン [Rhai](https://rhai.rs/) スクリプトを実行します。スクリプトはシナリオ変数の読み書きと値の返却ができます。

```yaml
- script: |
    let y = now_year();
    let m = now_month();
    let eom = end_of_month_str(y, m, "%Y/%m/%d");
    eom
  save_as: end_of_month
```

### 組み込み Rhai 関数

| 関数 | 戻り値 | 説明 |
|----------|--------|-------------|
| `now_year()` | `int` | 現在の年 |
| `now_month()` | `int` | 現在の月 (1〜12) |
| `now_day()` | `int` | 現在の日 |
| `today()` | `String` | `"YYYY-MM-DD"` 形式の今日の日付 |
| `end_of_month(year, month)` | `int` | 月の最終日 |
| `end_of_month_str(year, month, fmt)` | `String` | フォーマットされた月の最終日 |

### スクリプト内でシナリオ変数にアクセスする

```yaml
- set:
    name: base_price
    value: 1000

- script: |
    let tax = base_price * 0.1;
    base_price + tax
  save_as: total_price
```

## library

WASM プラグイン関数を呼び出します。

```yaml
- library:
    name: excel-reader.read_sheet
    inputs:
      path: "data.xlsx"
      sheet: "Sheet1"
    save_as: rows
```

独自プラグインの作成方法については [プラグイン開発](#) を参照してください。
