# クイックスタート

## Windows — ビルド不要

1. [rpa-x86_64-windows.zip](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-windows.zip) をダウンロード
2. 任意の場所（例: `C:\Tools\robost`）に展開
3. **`rpa.exe`** をダブルクリック — ブラウザが自動でビジュアルエディタを開く

Rust・Cargo・Visual Studio は不要です。

コマンドラインからシナリオを実行する:

```
rpa run examples\windows\calculator.yaml
rpa run examples\windows\calculator.yaml --dry-run
```

> **開発者向け（ソースからビルド）:** `cargo build --workspace`

---

## 1. エディタを起動する

**インストーラー / ポータブル ZIP:**

```
rpa.exe
```

**ソースから:**

```bash
cargo run -p robost-cli --features embed-editor -- agent
```

## 2. 最初のシナリオを作成する

1. ツールバーで **シナリオ名** を入力する
2. **ノード** パネル (左) からステップへノードをドラッグするか、ダブルクリックで追記する
3. ステップを選択すると、中央パネルでプロパティを編集できる
4. **保存** (Cmd+S) を押して YAML ファイルを書き出す
5. **実行** (F5) を押してシナリオを実行する

## 3. YAML を直接書く

robost のシナリオはシンプルな YAML ファイルです。`hello.yaml` を作成してみましょう:

```yaml
name: hello
steps:
  - wait_ms: 500
  - type: "Hello from robost!"
  - press: Enter
```

実行する:

```bash
cargo run -p robost-cli -- run hello.yaml
```

## 4. 変数を使う

```yaml
name: with_variables
variables:
  target_app: "Notepad"
steps:
  - wait_window:
      title_contains: "{{ target_app }}"
      timeout_ms: 10000
  - type: "Automated by robost"
```

## 5. テンプレート画像を採取する

1. 対象アプリケーションを開く
2. トレイから `robost-snip` を起動するか、`cargo run -p robost-snip` を実行する
3. **Ctrl+Shift+C** を押して画面をフリーズする
4. UI 要素を矩形で囲む
5. テンプレート PNG がプロジェクトフォルダに保存される
