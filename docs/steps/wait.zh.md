# 等待

## wait_ms

暂停执行指定的毫秒数。

```yaml
- wait_ms: 500
- wait_ms: 2000    # 2 秒
```

## wait_window

等待标题匹配的窗口出现（或消失）。

```yaml
- wait_window:
    title_contains: "My Application"
    state: exists         # exists | closed（默认: exists）
    timeout_ms: 10000
```

## wait_until

轮询条件直到其变为真。

```yaml
- wait_until:
    cond: "status == \"done\""
    timeout_ms: 30000
    interval_ms: 500      # 轮询间隔（默认: 200）
```

## wait_process

等待进程启动或停止。

```yaml
- wait_process:
    name: notepad.exe
    state: started        # started | stopped（默认: started）
    timeout_ms: 10000
```
