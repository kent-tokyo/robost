# 图像操作

所有图像步骤均使用通过 `robost-snip` 采集的模板 PNG 文件。

## wait_image

等待模板出现在屏幕上。

```yaml
- wait_image:
    template: button.png
    timeout_ms: 5000      # 默认: 10000
    threshold: 0.88       # 匹配置信度 0–1（默认: 0.88）
```

## click_image

等待模板出现后点击其中心。

```yaml
- click_image:
    template: submit.png
    timeout_ms: 5000
    button: left          # left | right | middle（默认: left）
    offset_x: 0           # 相对中心的像素偏移
    offset_y: 0
```

## find_image

查找模板并保存其位置，不执行点击。

```yaml
- find_image:
    template: icon.png
    save_as: pos          # { x, y, width, height }
    timeout_ms: 5000
```

## match_rect

在指定屏幕区域内匹配模板。

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

等待模板从屏幕上消失。

```yaml
- wait_no_image:
    template: loading.png
    timeout_ms: 30000
```

## wait_change

等待屏幕发生变化（区域内任意像素差异）。

```yaml
- wait_change:
    timeout_ms: 5000
    region:               # 可选——仅监视该区域
      x: 0
      y: 0
      width: 800
      height: 600
```

## ocr_match

使用 OCR 等待文字出现在屏幕上。

```yaml
- ocr_match:
    contains: "处理完了"
    lang: jpn+eng         # Tesseract 语言代码
    timeout_ms: 10000
    save_as: matched_text
```

## ml_detect

使用 ONNX 模型检测对象（兼容 YOLO）。

```yaml
- ml_detect:
    model: model.onnx
    threshold: 0.7
    save_as: detections   # { label, x, y, width, height, confidence } 的列表
```

## screenshot_save

将当前屏幕保存为 PNG 文件。

```yaml
- screenshot_save:
    path: "screenshots/{{ today }}.png"
```

## get_pixel_color

读取单个像素的颜色。

```yaml
- get_pixel_color:
    x: 500
    y: 300
    save_as: col          # "#RRGGBB" 字符串
```

## wait_color

等待像素达到指定颜色。

```yaml
- wait_color:
    x: 500
    y: 300
    color: "#00FF00"
    tolerance: 10         # 每通道容差（默认: 0）
    timeout_ms: 10000
```

## window_control

将窗口置于前台、最小化、最大化或关闭窗口。

```yaml
- window_control:
    title_contains: "Notepad"
    action: focus         # focus | minimize | maximize | close | restore
```
