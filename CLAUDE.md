# CLAUDE.md

このファイルは Claude Code がこのリポジトリで作業する際のガイダンスを提供します。

## プロジェクト概要

Rust 製の OSS RPA ツール。差別化ポイント:

- **画像認識ベースの自動化** — テンプレートマッチ、OCR、ML 検出
- **リモートデスクトップアプリ対応** — RDP/Citrix/VNC を外部キャプチャ方式で操作
- **WinActor 級のテンプレート採取 UX** — ドロップダウン等の transient UI を取れる
- **WASM プラグインによる拡張性** — コミュニティが標準ライブラリを積み上げられる設計
- **Workspace構成** — コア部品は単独で再利用可能

**現フェーズ: Phase 1** — 単一 workspace、内部 API のみ。公開リリースなし。

## アーキテクチャ

```
rust_rpa/
├── crates/
│   ├── rpa-capture/        # 画面/ウィンドウキャプチャ (xcap ラッパー)
│   ├── rpa-input/          # マウス/キー入力 + フォーカス管理 (enigo ラッパー)
│   ├── rpa-vision/         # テンプレマッチ / OCR / ML 検出
│   ├── rpa-backend/        # Local / RDP / VNC を統一する trait
│   ├── rpa-core/           # シナリオエンジン、リトライ、待機
│   ├── rpa-snip/           # テンプレート採取 GUI (tray app)
│   ├── rpa-template/       # テンプレート形式の共有型定義
│   ├── rpa-plugin-api/     # プラグイン作者向け公開API (crates.io 早期公開対象)
│   ├── rpa-plugin-host/    # wasmtime ベースのプラグインローダ、権限管理
│   ├── rpa-script/         # Rhai 統合 (インラインスクリプト)
│   ├── rpa-stdlib/         # ビルトインライブラリ群
│   └── rpa-cli/            # バイナリ
├── plugins/                # 公式プラグインのリファレンス実装 (別 workspace)
└── Cargo.toml
```

### Backend trait

`rpa-backend` の `Backend` trait がローカル/リモートを隠蔽する。新バックエンドはここに追加し、`rpa-core` には書かない。

```rust
pub trait Backend {
    fn capture(&self, target: &Target) -> Result<RgbaImage>;
    fn click(&self, point: Point) -> Result<()>;
    fn type_text(&self, text: &str) -> Result<()>;
}
```

RDP バックエンドは **路線A (外部キャプチャ方式) のみ**。RDP プロトコルは喋らない。将来 IronRDP ベースのプロトコル直結を実装する場合は別 crate `rpa-backend-rdp-native` として追加。

## プラグイン機構

WinActor 互換の拡張性を実現する **3層構成**。シナリオ呼び出し構文はビルトインもプラグインも統一(作者に意識させない)。

| 層 | 用途 | 技術 |
|---|---|---|
| Built-in | 標準ライブラリ | `rpa-stdlib`、Rust 静的リンク |
| WASM Plugin | 外部配布拡張 | `wasmtime` + WASI 0.2 Component Model |
| Inline Script | シナリオ内の一時ロジック | Rhai (`rpa-script`) |

### なぜ WASM か

- サンドボックス: プラグイン障害がワーカーを巻き込まない
- 言語非依存: Rust/AssemblyScript/Go/C++ 等から `.wasm` を生成可
- 配布が一ファイル: 環境依存なし
- WASI Component Model で型付きインタフェース
- 将来のレジストリ化が綺麗

DLL/native plugin、組み込み Python/Lua は **採用しない**。ABI 地獄とサンドボックス欠如のため。

### プラグインマニフェスト

`.wasm` + `plugin.toml` のペア配布。権限は宣言必須。

```toml
[plugin]
name = "excel-reader"
version = "0.1.0"

[[function]]
name = "read_sheet"

[[function.inputs]]
name = "path"
type = "string"

[[function.outputs]]
name = "rows"
type = "list<list<string>>"

[permissions]
filesystem = ["read"]
network = false
screen = false
```

権限なし機能へのアクセスは実行時に拒否。プラグイン初回ロード時にユーザー承認 UI を出す。

### `rpa-plugin-api` の早期公開

プラグイン作者が依存するのはこの crate のみ。**他crateの内部実装変更がプラグイン互換性を壊さない** 設計とし、Phase 2 を待たずに crates.io 公開する候補。

## テンプレート採取ツール (rpa-snip)

WinActor の最大の競合優位はテンプレート採取 UX。ドロップダウン、ツールチップ、ホバー状態など **transient UI を取れる** ことが必須。

### 設計: フリーズ + 遅延キャプチャ

1. tray app として常駐 (ウィンドウなし、フォーカス奪わない)
2. ユーザーが対象 UI を手動で表示
3. グローバルホットキー (Ctrl+Shift+C) でフルスクリーンキャプチャ
4. ピクセルを「凍結」してフルスクリーンオーバーレイ表示
5. 凍結画像から矩形選択 (元 UI は閉じて OK)
6. PNG + メタデータ YAML として保存

**キャプチャからオーバーレイ表示までを 50ms 以内** に保つ (体感ラグ防止)。

### 追加機能

- **アンカー + オフセット** — 基準画像からの相対座標でクリック点定義
- **マスク領域** — 動的部分 (タイムスタンプ、バッジ) をマッチング時に除外
- **マルチスケール保存** — DPI 100/125/150% を自動生成、RDP解像度変動への耐性
- **UI Automation連携** (Windowsローカル時のみ) — 画像 + UIA要素情報のハイブリッド保存
- **ライブテスト** — 採取直後に現在画面でのマッチング確認

### 依存

```toml
xcap = "0.9"
global-hotkey = "0.6"
winit = "0.30"
egui = "0.28"
egui-winit = "0.28"
```

## 技術スタック

- `xcap` — キャプチャ (クロスプラットフォーム優先、scap は性能要件が出てから検討)
- `enigo` 0.3+ — 入力エミュレーション
- `image` / `imageproc` — 画像処理、テンプレートマッチ
- `leptess` — Tesseract OCR バインディング
- `ort` — ONNX Runtime (ML 検出、YOLO 等)
- `wasmtime` — WASM プラグイン実行
- `rhai` — インラインスクリプト
- `tokio` — 非同期ランタイム
- `tracing` — ログ (`log` / `env_logger` は使わない)
- `serde` + `serde_yaml` — シナリオ/マニフェスト パース
- `thiserror` (lib) + `anyhow` (bin/test)
- `ureq` = "2" — HTTP クライアント (feature = "http"、blocking + json feature)

## 致命的な落とし穴 — 必ずハンドリングする

### RDP/リモートウィンドウ固有

1. **フォーカス必須** — enigo は OS レベル入力。RDP ウィンドウが最前面でないとキーが他所へ飛ぶ。`rpa-input` は送信前に対象ウィンドウを前面化する
2. **DPI スケーリング** — Windows 拡大率 125%/150% でキャプチャ座標とマウス座標がズレる。起動時に `SetProcessDpiAwareness` を呼ぶ
3. **RDP 圧縮アーティファクト** — 可変圧縮による色差。正規化相互相関 + 閾値 0.85〜0.90、完全一致は使わない
4. **描画遅延** — クリック後の画面更新に 100〜500ms。次状態の polling 必須
5. **再接続でハンドル無効化** — RDP 再接続後はウィンドウハンドルが変わる。シナリオステップ毎に `Window::all()` で再解決
6. **解像度変動** — RDP セッションリサイズでテンプレ無効化。マルチスケール or 相対アンカーで対応

### プラグイン固有

7. **権限チェックは必ず実行時** — マニフェストの宣言だけで信用しない。WASI capability で物理的にアクセス遮断
8. **プラグインのパニック** — wasmtime の trap として捕捉。ワーカープロセスは絶対に巻き込まない
9. **長時間実行プラグイン** — タイムアウト + キャンセル機構必須

### 全般

- 座標: 関数シグネチャに window-local か screen-global かを明記。Backend 境界では screen-global がデフォルト
- シナリオエンジンはリトライ + 失敗時スクリーンショット保存を必ず組み込む
- 画像処理(CPU bound) で async runtime をブロックしない。`tokio::task::spawn_blocking` 経由
- シークレット: 環境変数または OS keychain 経由。シナリオファイルに平文を書かせない

## コード規約

- **エラー**: 各 crate で `thiserror` の独自 `Error` enum。bin/test は `anyhow` で OK
- **async**: `tokio` 統一。CPU bound 処理は `spawn_blocking`
- **公開 API**: 入力は借用、出力は所有。trait object のジェネリック爆発を避ける
- **ログ**: `tracing` でシナリオステップ毎に span を切る。テンプレマッチスコア、リトライ回数、失敗時のスクリーンショットパスを記録
- **テスト**: 事前キャプチャ画像を入力にする hermetic test を優先。ライブディスプレイ依存テストは最小限
- **MSRV**: stable Rust、6ヶ月以上経過したバージョン
- **依存追加**: `cargo add` 前にこの CLAUDE.md の技術スタック節を更新

## コマンド

```bash
cargo build --workspace
cargo test --workspace
cargo run -p rpa-cli -- run scenario.yaml
cargo run -p rpa-snip                            # テンプレ採取ツール起動
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

# プラグイン関連
cargo build -p excel-reader --target wasm32-wasip2
cargo run -p rpa-cli -- plugin install ./plugins/excel-reader/target/...wasm
cargo run -p rpa-cli -- plugin list
```

## シナリオ形式 (draft)

YAML ベース。仕様は流動的。

```yaml
name: "example"
target:
  kind: window
  title_contains: "リモートデスクトップ"
steps:
  - wait_image: { template: "login_button.png", timeout_ms: 5000 }
  - click_image: { template: "login_button.png" }
  - type: "username"
  - press: Tab
  - type: { secret_env: "PASSWORD" }
  - press: Enter

  # プラグイン呼び出し (ビルトインと同構文)
  - library: excel-reader.read_sheet
    inputs:
      path: "data.xlsx"
      sheet: "Sheet1"
    save_as: rows

  # インラインスクリプト
  - script: |
      let date = now().format("yyyyMMdd");
      return `report_${date}.xlsx`;
    save_as: filename

  # 反復
  - foreach: rows
    do:
      - click_image: { template: "row_button.png" }
      - type: "{{ item[0] }}"
```

## スコープ外 — 追加しない

- **WinActor `.ums6`/`.ums7` のパース** — もし互換層を作る場合は別リポジトリで、出力はこのプロジェクトのシナリオ形式のみ。法務/IP 上の理由
- **IronRDP によるプロトコル直結** — Phase 1 は路線 A のみ。プロトコル方式は将来別 crate
- **Web UI / シナリオエディタ** — Phase 1 は CLI + YAML のみ。GUI は `rpa-snip` のみ
- **クラウドオーケストレーション** — Phase 3 まで保留
- **DLL/ネイティブプラグイン** — WASM のみ。例外なし
- **プラグインレジストリ (中央サーバー)** — Phase 3 まで保留、Phase 2 は GitHub release ベース

## ロードマップ

- **Phase 1 (現在)**: 単一 workspace、内部 API、ローカルプラグイン読み込み
- **Phase 2**: `rpa-plugin-api` / `rpa-vision` / `rpa-capture` / `rpa-input` を crates.io 公開。プラグインを GitHub release から install できる仕組み
- **Phase 3**: gRPC/JSON-RPC サーバー化、シナリオ層の言語非依存化、公式プラグインレジストリ

次フェーズへの移行トリガーは **実需** (外部ユーザー or 具体的な統合先)。憶測では動かさない。

## 戦略的補足

このプラグイン機構が機能し始めると、副次効果として **WinActor からの移行パスが現実的になる**。「ums の各ノードを対応するプラグイン呼び出しに変換する」設計が取れるため、ライブラリ再実装責任をコミュニティに分散できる。本体は実行基盤のみに集中。