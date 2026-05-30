# 通知

## notify

デスクトップ通知を送信します。

```yaml
- notify:
    title: "robost"
    message: "Scenario completed successfully"
    icon: info            # info | warning | error (デフォルト: info)
```

## notify_sound

システムサウンドを再生します。

```yaml
- notify_sound:
    sound: complete       # complete | error | ping (デフォルト: complete)
```
