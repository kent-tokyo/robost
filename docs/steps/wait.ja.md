# 待機

## wait_ms

指定したミリ秒数だけ実行を一時停止します。

```yaml
- wait_ms: 500
- wait_ms: 2000    # 2秒
```

## wait_window

一致するタイトルを持つウィンドウが存在する (または消える) まで待機します。

```yaml
- wait_window:
    title_contains: "My Application"
    state: exists         # exists | closed (デフォルト: exists)
    timeout_ms: 10000
```

## wait_until

条件が真になるまでポーリングします。

```yaml
- wait_until:
    cond: "status == \"done\""
    timeout_ms: 30000
    interval_ms: 500      # ポーリング間隔 (デフォルト: 200)
```

## wait_process

プロセスが開始または停止するまで待機します。

```yaml
- wait_process:
    name: notepad.exe
    state: started        # started | stopped (デフォルト: started)
    timeout_ms: 10000
```
