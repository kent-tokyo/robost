# Notifications

## notify

Send a desktop notification.

```yaml
- notify:
    title: "robost"
    message: "Scenario completed successfully"
    icon: info            # info | warning | error (default: info)
```

## notify_sound

Play a system sound.

```yaml
- notify_sound:
    sound: complete       # complete | error | ping (default: complete)
```
