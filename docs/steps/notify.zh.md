# 通知

## notify

发送桌面通知。

```yaml
- notify:
    title: "robost"
    message: "Scenario completed successfully"
    icon: info            # info | warning | error（默认: info）
```

## notify_sound

播放系统声音。

```yaml
- notify_sound:
    sound: complete       # complete | error | ping（默认: complete）
```
