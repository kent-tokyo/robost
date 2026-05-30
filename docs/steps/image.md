# Image Operations

All image steps use template PNG files captured with `robost-snip`.

## wait_image

Wait until a template appears on screen.

```yaml
- wait_image:
    template: button.png
    timeout_ms: 5000      # default: 10000
    threshold: 0.88       # match confidence 0–1 (default: 0.88)
```

## click_image

Wait for a template then click its center.

```yaml
- click_image:
    template: submit.png
    timeout_ms: 5000
    button: left          # left | right | middle (default: left)
    offset_x: 0           # pixel offset from center
    offset_y: 0
```

## find_image

Search for a template and save its position without clicking.

```yaml
- find_image:
    template: icon.png
    save_as: pos          # { x, y, width, height }
    timeout_ms: 5000
```

## match_rect

Match a template within a restricted screen region.

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

Wait until a template disappears from the screen.

```yaml
- wait_no_image:
    template: loading.png
    timeout_ms: 30000
```

## wait_change

Wait until the screen changes (any pixel difference in the region).

```yaml
- wait_change:
    timeout_ms: 5000
    region:               # optional — watch only this area
      x: 0
      y: 0
      width: 800
      height: 600
```

## ocr_match

Wait for text to appear on screen using OCR.

```yaml
- ocr_match:
    contains: "処理完了"
    lang: jpn+eng         # Tesseract language codes
    timeout_ms: 10000
    save_as: matched_text
```

## ml_detect

Detect objects using an ONNX model (YOLO-compatible).

```yaml
- ml_detect:
    model: model.onnx
    threshold: 0.7
    save_as: detections   # list of { label, x, y, width, height, confidence }
```

## screenshot_save

Save the current screen to a PNG file.

```yaml
- screenshot_save:
    path: "screenshots/{{ today }}.png"
```

## get_pixel_color

Read the color of a single pixel.

```yaml
- get_pixel_color:
    x: 500
    y: 300
    save_as: col          # "#RRGGBB" string
```

## wait_color

Wait until a pixel reaches the specified color.

```yaml
- wait_color:
    x: 500
    y: 300
    color: "#00FF00"
    tolerance: 10         # per-channel tolerance (default: 0)
    timeout_ms: 10000
```

## window_control

Bring a window to focus, minimize, maximize, or close it.

```yaml
- window_control:
    title_contains: "Notepad"
    action: focus         # focus | minimize | maximize | close | restore
```
