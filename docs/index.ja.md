# robost

[🇺🇸 English](https://kent-tokyo.github.io/robost/) | [🇨🇳 中文](https://kent-tokyo.github.io/robost/zh/)

Rust 製の RPA 自動化ツール。画像認識・OCR・WASM プラグインに対応。

## robost とは?

robost は UI フレームワークへのアクセスを必要とせず、デスクトップ操作を自動化します。RDP/Citrix/VNC セッションに対しても、画面をキャプチャしてマウス・キーボード入力を再現する方式で動作します。シナリオはシンプルな YAML で記述します。

```yaml
name: login_example
steps:
  - wait_image:
      template: login_button.png
      timeout_ms: 5000
  - click_image:
      template: login_button.png
  - type: "myusername"
  - press: Tab
  - type:
      secret_env: PASSWORD
  - press: Enter
```

## 主な機能

- **画像ベースの自動化** — テンプレートマッチ、OCR、ML 検出
- **リモートデスクトップ対応** — 外部キャプチャ方式で RDP/Citrix/VNC に対応
- **WASM プラグイン** — 任意の言語でサンドボックス化されたプラグインを追加可能
- **YAML シナリオ** — 人が読みやすく、バージョン管理に適した形式
- **ビジュアルエディタ** — ドラッグ&ドロップでシナリオを作成、ライブプレビュー付き

## クイックナビゲーション

- [インストール](guides/install.md)
- [クイックスタート](guides/quickstart.md)
- [ステップリファレンス](steps/control_flow.md)
