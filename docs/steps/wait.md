# Wait

## wait_ms

Pause execution for a fixed number of milliseconds.

```yaml
- wait_ms: 500
- wait_ms: 2000    # 2 seconds
```

## wait_window

Wait until a window with a matching title exists (or disappears).

```yaml
- wait_window:
    title_contains: "My Application"
    state: exists         # exists | closed (default: exists)
    timeout_ms: 10000
```

## wait_until

Poll a condition until it becomes true.

```yaml
- wait_until:
    cond: "status == \"done\""
    timeout_ms: 30000
    interval_ms: 500      # poll interval (default: 200)
```

## wait_process

Wait for a process to start or stop.

```yaml
- wait_process:
    name: notepad.exe
    state: started        # started | stopped (default: started)
    timeout_ms: 10000
```
