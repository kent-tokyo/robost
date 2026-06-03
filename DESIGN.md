# DESIGN.md — robost デザインガイドライン

> Windows アプリ設計の公式指針 ([Microsoft Design Guidelines](https://learn.microsoft.com/en-us/windows/apps/design/guidelines-overview)) を参照し、
> robost の技術スタック・ユーザー像に合わせて再解釈したドキュメントです。

---

## 0. デザイン哲学

robost のユーザーは **RPA 実務者・エンジニア** であり、UI の "かわいさ" より **作業の速さと正確さ** を優先する。

| 原則 | 意味 |
|------|------|
| **透明性** | ツールが何をしているかをユーザーが常に把握できる |
| **邪魔しない** | tray アイコンも overlay も、ユーザーの操作対象 UI を塞がない |
| **エラーを隠さない** | 失敗は即座にスクリーンショット付きで提示する |
| **一貫性** | ビルトイン / プラグイン / スクリプトで同じ呼び出し構文、同じ視覚フィードバック |

---

## 1. カラー (Color)

Microsoft 指針: *色を使って階層・意味・視覚的同一性を確立する*

### 1.1 セマンティックカラー

| 用途 | 推奨値 (Light) | 推奨値 (Dark) | 意味 |
|------|--------------|--------------|------|
| **Accent** | `#0078D4` (Windows Blue) | `#60CDFF` | 選択状態、進行中 |
| **Success** | `#107C10` | `#6CCB5F` | マッチ成功、ステップ完了 |
| **Warning** | `#C19C00` | `#FCE100` | タイムアウト近接、閾値下限 |
| **Error** | `#C42B1C` | `#FF99A4` | マッチ失敗、ステップエラー |
| **Overlay BG** | `rgba(0,0,0,0.55)` | — | snip ツールの凍結オーバーレイ背景 |
| **Selection** | `rgba(0,120,212,0.35)` | `rgba(96,205,255,0.35)` | snip 矩形選択 |

### 1.2 カラー使用ルール

- ステップアイコン・ステータスバッジ以外に色を多用しない（最大 3 アクセントカラーまで）
- テキスト on 背景は **WCAG AA 以上** (コントラスト比 4.5:1 以上)
- アクセントカラーのみでエラー状態を伝えない。アイコン・テキストと組み合わせる

---

## 2. タイポグラフィ (Typography)

Microsoft 指針: *一貫した書体と階層でトーンを設定し可読性を高める*

egui はシステムフォントを利用しないため、**Noto Sans / Noto Sans JP** を同梱する。

### 2.1 タイプスケール

| ロール | サイズ | ウェイト | 用途 |
|--------|--------|---------|------|
| `Display` | 28 px | SemiBold | ウィンドウタイトル |
| `Title` | 20 px | SemiBold | パネル見出し |
| `Body` | 14 px | Regular | 通常テキスト、ラベル |
| `Caption` | 12 px | Regular | メタ情報、ヒント |
| `Monospace` | 13 px | Regular | シナリオ YAML、ログ出力 |

### 2.2 ルール

- UI ラベルは文章の先頭のみ大文字 (Sentence case)。`"Save File"` × → `"Save file"` ○
- 長い文を折り返す場合は `line-height: 1.5` を基準に
- シナリオ YAML はモノスペースで常に表示する。絶対にプロポーショナルフォントを使わない

---

## 3. ジオメトリ・レイアウト (Geometry & Layout)

Microsoft 指針: *形・サイズ・空間的関係で均衡のとれた予測可能なレイアウトを作る*

### 3.1 グリッドと余白

- 基本単位: **4 px**
- 要素間余白: 4 / 8 / 12 / 16 / 24 / 32 px のいずれか
- 角丸: コントロール 4 px、パネル 8 px、モーダル 12 px
- パネル最小幅: 240 px、推奨幅: 320 px

### 3.2 コンポーネント別サイズ

| コンポーネント | 推奨高さ | 備考 |
|--------------|---------|------|
| ボタン (primary) | 32 px | min-width: 80 px |
| テキストフィールド | 32 px | |
| ステップ行 (シナリオエディタ) | 48 px | ドラッグ把持用余白を含む |
| Tray メニュー項目 | 32 px | |
| snip 選択ハンドル | 10×10 px | 視認性確保 |

### 3.3 snip オーバーレイ固有

- オーバーレイは **全モニター** を覆う
- 凍結前のキャプチャ画像は 1:1 ピクセルで表示（拡大縮小禁止）
- 選択矩形の寸法 (w × h px) は カーソル脇にリアルタイム表示
- ツールバーはデスクトップ **下端中央** に配置。ターゲット UI を遮らない

---

## 4. 高さ・奥行き (Elevation)

Microsoft 指針: *深度とレイヤーでフォーカスを誘導し構造を強化する*

robost は egui を使うため Mica/Acrylic 素材は使えない。代わりに **影とボーダーカラー** で奥行きを表現する。

| レイヤー | 表現 | 使う場面 |
|---------|------|---------|
| L0: 背景 | `bg_color` | アプリ背景 |
| L1: パネル | `bg_color + 10` (明度差) | サイドパネル、ツールバー |
| L2: カード | `bg_color + 20` + 影 1px | ステップカード、設定パネル |
| L3: ポップオーバー | `bg_color + 30` + 影 4px | ドロップダウン、ツールチップ |
| L4: モーダル | dim overlay + 影 8px | 確認ダイアログ |

---

## 5. モーション (Motion)

Microsoft 指針: *フィードバックと注目誘導にモーションを使い、スムーズな操作感を生む*

**基本方針: 省略可能なアニメーションはデフォルト OFF**。RPA 実行中に視覚的ノイズを増やさない。

### 5.1 所要時間ガイドライン

| 種類 | 時間 | イージング |
|------|------|---------|
| スライドイン/アウト | 150 ms | `ease-out` |
| フェードイン/アウト | 100 ms | linear |
| プログレスバー更新 | — | linear |
| スクロール | 即時 | — |

### 5.2 snip ツール固有タイミング制約

> キャプチャ → オーバーレイ表示: **50 ms 以内**

- ホットキー受信からキャプチャ完了まで: ≤ 30 ms
- egui ウィンドウ生成・描画: ≤ 20 ms
- この制約を破るアニメーション (フェードイン等) を snip オーバーレイに追加しない

---

## 6. ナビゲーション (Navigation)

Microsoft 指針: *予測可能な構造でユーザーを誘導する*

現フェーズの GUI コンポーネントは **単一ウィンドウ・フラット構造** が基本。

### 6.1 シナリオエディタ

```
┌─────────────────────────────────────────────┐
│ Toolbar (run / stop / save / snip)          │
├────────────┬────────────────────────────────┤
│ Step List  │  Step Inspector (selected step) │
│ (left)     │  (right)                        │
└────────────┴────────────────────────────────┘
```

- 選択中ステップは Inspector に即時反映。別ウィンドウを開かない
- `Esc` で選択解除、`Delete` でステップ削除、`Ctrl+Z` で Undo

### 6.2 tray メニュー構造

```
robost
├── Open Scenario Editor
├── New Template (snip)
├── Recent Scenarios >
├── ─────────────────
├── Settings
└── Quit
```

- 階層は 2 段まで。それ以上はエディタ内に移す

---

## 7. コマンド体系 (Commanding)

Microsoft 指針: *ユーザーが何をできるかを明快で一貫したパターンで提示する*

### 7.1 グローバルショートカット (tray 常駐時 常時有効)

| キー | 動作 |
|------|------|
| `Ctrl+Shift+C` | snip ツール起動 (テンプレート採取) |
| `Ctrl+Shift+S` | シナリオエディタ起動 / 前面化 |

### 7.2 snip オーバーレイ内

| キー | 動作 |
|------|------|
| `Escape` | snip キャンセル、オーバーレイ閉じる |
| `Enter` | 選択確定、保存ダイアログへ |
| `Space` | マスク領域モード切り替え |

### 7.3 シナリオエディタ内

| キー | 動作 |
|------|------|
| `Ctrl+R` | シナリオ実行 |
| `Ctrl+S` | 保存 |
| `Ctrl+Z` / `Ctrl+Shift+Z` | Undo / Redo |
| `Del` | 選択ステップ削除 |
| `↑` / `↓` | ステップ選択移動 |
| `Ctrl+↑` / `Ctrl+↓` | ステップ並び替え |

---

## 8. アイコノグラフィ (Iconography)

Microsoft 指針: *なじみのある目的あるアイコンでアクションや概念を素早く伝える*

- **Fluent System Icons** ([microsoft/fluentui-system-icons](https://github.com/microsoft/fluentui-system-icons)) を使用する
- アイコンサイズ: 16 px (ツールバー内ラベルあり)、20 px (ツールバーアイコンのみ)、24 px (特大ボタン)
- 独自アイコンが必要な場合は SVG (単色、ストローク統一) で作成し `assets/icons/` に格納

### 8.1 ステップ種別アイコン対応

| ステップ種別 | アイコン名 | 色 |
|------------|----------|-----|
| `click_image` | `cursor_click` | Accent |
| `wait_image` | `timer` | Warning |
| `type` | `keyboard` | neutral |
| `press` | `keyboard` | neutral |
| `script` | `code` | neutral |
| `library` (plugin) | `puzzle` | `#7B68EE` (Plugin Purple) |
| `foreach` | `arrow_repeat` | neutral |

---

## 9. ユーザビリティ (Usability)

Microsoft 指針: *直感的な操作・明確なアフォーダンス・アクセシビリティを確保する*

### 9.1 アクセシビリティ

- キーボードのみで全機能操作可能にする (Tab 順序を正しく設定)
- egui の `AccessKit` 統合を有効化する
- フォーカスリングは常に表示。`:hover` と `:focus` を区別するスタイルを使う
- エラーや警告メッセージはアイコン + テキストで伝える (色のみに依存しない)

### 9.2 インタラクティブ状態

各コントロールに以下 4 状態のスタイルを必ず定義する:

| 状態 | 視覚変化 |
|------|---------|
| Default | ベーススタイル |
| Hover | 背景明度 +10%、ポインタ `pointer` |
| Pressed | 背景明度 -10%、2px 内側 |
| Disabled | opacity 0.38、ポインタ `not-allowed` |

### 9.3 エラーとフィードバック

- マッチ失敗時: エラーパネルに **失敗スクリーンショット + マッチスコア** を即時表示
- 処理中は必ずプログレス表示 (不定プログレスバー or スピナー)
- 破壊的操作 (テンプレート上書き、ステップ削除) には確認ダイアログを挟む
- シナリオ実行完了/失敗は tray バルーン通知でフィードバックする

---

## 10. マテリアル・仕上げ (Materials)

Microsoft 指針: *Mica や Acrylic などで深みと温もりを加える*

egui では OS 素材は使えないが、以下で代替する:

- **ヘッダー背景**: Accent カラーの 10% opacity で薄くグラデーション
- **パネル区切り**: 1 px セパレータ (`stroke_color`)
- **スクロールバー**: 細め (4 px)、ホバー時のみ可視化

将来的に WebView2 / WinUI ベースのエディタへ移行する場合は Mica Base を採用する。

---

## 11. ウィジェット設計 (Widgets)

Microsoft 指針: *要点をひと目で確認できるインタラクティブなサーフェスを提供する*

現フェーズでは Windows ウィジェットは実装しないが、以下を設計原則とする:

- ステップカードは **1 行で意図が読める** 自己完結テキストを持つ
- テンプレート画像はサムネイル (48×48 px) で常にプレビュー表示
- マッチスコアは数値 + プログレスバーで同時表示

---

## 12. ライティング (Writing)

Microsoft 指針: *明確・簡潔・親切な言葉で理解を助け認知負荷を下げる*

### 12.1 基本方針

- UI テキストは **日本語・英語両対応** を前提に設計 (UI 文字列は `i18n/` に分離)
- 動詞で始めるボタンラベル: `"Save"` / `"Cancel"` / `"Run scenario"`
- ユーザーを責めるエラーメッセージを書かない

### 12.2 良い例 / 悪い例

| 場面 | NG | OK |
|------|----|----|
| マッチ失敗 | "Error: template not found" | "Template not found — try lowering the threshold or recapturing" |
| 保存完了 | "Done." | "Saved to `templates/login_button.png`" |
| 削除確認 | "Are you sure?" | "Delete 'login_button' template? This can't be undone." |
| プラグイン権限 | "Permission required" | "'excel-reader' needs filesystem read access. Allow?" |

### 12.3 トーン

- 丁寧かつ端的。冗長な「〜してください」より命令形
- エラーには必ず **次のアクション提案** を添える
- 専門用語 (テンプレート、閾値、OCR) はそのまま使う。RPA 実務者向けのため言い換え不要

---

## 付録A: egui 実装メモ

- `egui::Style` でトークンを一元定義し、ハードコードしない
- `egui::Context::set_pixels_per_point()` で DPI を正しく反映する
- フォントファイルは `include_bytes!()` でバイナリに埋め込む
- `egui_extras` の `TableBuilder` を使ってステップ一覧を仮想スクロールする (大規模シナリオ対応)

## 付録B: デザイントークン (暫定)

```rust
// crates/robost-ui/src/tokens.rs (仮)
pub const ACCENT:       egui::Color32 = egui::Color32::from_rgb(0x00, 0x78, 0xD4);
pub const SUCCESS:      egui::Color32 = egui::Color32::from_rgb(0x10, 0x7C, 0x10);
pub const WARNING:      egui::Color32 = egui::Color32::from_rgb(0xC1, 0x9C, 0x00);
pub const ERROR:        egui::Color32 = egui::Color32::from_rgb(0xC4, 0x2B, 0x1C);
pub const PLUGIN_PURPLE: egui::Color32 = egui::Color32::from_rgb(0x7B, 0x68, 0xEE);

pub const SPACING_XS: f32 = 4.0;
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 16.0;
pub const SPACING_LG: f32 = 24.0;

pub const ROUNDING_SM: egui::Rounding = egui::Rounding::same(4.0);
pub const ROUNDING_MD: egui::Rounding = egui::Rounding::same(8.0);

pub const STEP_ROW_HEIGHT: f32 = 48.0;
pub const TOOLBAR_HEIGHT:  f32 = 40.0;
```
