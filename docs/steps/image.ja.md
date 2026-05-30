# 画像操作

すべての画像ステップは `robost-snip` で採取したテンプレート PNG ファイルを使用します。

## wait_image

テンプレートが画面に表示されるまで待機します。

```yaml
- wait_image:
    template: button.png
    timeout_ms: 5000      # デフォルト: 10000
    threshold: 0.88       # マッチング信頼度 0〜1 (デフォルト: 0.88)
```

## click_image

テンプレートを待ってその中央をクリックします。

```yaml
- click_image:
    template: submit.png
    timeout_ms: 5000
    button: left          # left | right | middle (デフォルト: left)
    offset_x: 0           # 中央からのピクセルオフセット
    offset_y: 0
```

## find_image

テンプレートを検索してクリックせずにその位置を保存します。

```yaml
- find_image:
    template: icon.png
    save_as: pos          # { x, y, width, height }
    timeout_ms: 5000
```

## match_rect

指定した画面領域内でテンプレートをマッチングします。

```yaml
- match_rect:
    template: button.png
    rect:
      x: 200
      y: 100
      width: 400
      height: 200
    timeout_ms: 5000
```

## wait_no_image

テンプレートが画面から消えるまで待機します。

```yaml
- wait_no_image:
    template: loading.png
    timeout_ms: 30000
```

## wait_change

画面が変化する (指定領域内のピクセルに差異が生じる) まで待機します。

```yaml
- wait_change:
    timeout_ms: 5000
    region:               # 省略可 — この領域のみを監視する
      x: 0
      y: 0
      width: 800
      height: 600
```

## ocr_match

OCR を使用して画面にテキストが表示されるまで待機します。

```yaml
- ocr_match:
    contains: "処理完了"
    lang: jpn+eng         # Tesseract 言語コード
    timeout_ms: 10000
    save_as: matched_text
```

## ml_detect

ONNX モデル (YOLO 互換) を使用してオブジェクトを検出します。

```yaml
- ml_detect:
    model: model.onnx
    threshold: 0.7
    save_as: detections   # { label, x, y, width, height, confidence } のリスト
```

## screenshot_save

現在の画面を PNG ファイルに保存します。

```yaml
- screenshot_save:
    path: "screenshots/{{ today }}.png"
```

## get_pixel_color

単一ピクセルの色を読み取ります。

```yaml
- get_pixel_color:
    x: 500
    y: 300
    save_as: col          # "#RRGGBB" 形式の文字列
```

## wait_color

指定したピクセルが指定した色になるまで待機します。

```yaml
- wait_color:
    x: 500
    y: 300
    color: "#00FF00"
    tolerance: 10         # チャンネルごとの許容値 (デフォルト: 0)
    timeout_ms: 10000
```

## window_control

ウィンドウをフォーカス、最小化、最大化、または閉じます。

```yaml
- window_control:
    title_contains: "Notepad"
    action: focus         # focus | minimize | maximize | close | restore
```
