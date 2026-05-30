# Web / HTTP

## http_get

HTTP GET リクエストを送信します。

```yaml
- http_get:
    url: "https://api.example.com/data"
    headers:
      Authorization: "Bearer {{ token }}"
    save_as: response     # { status, body, headers }
```

## http_post

JSON ボディを含む HTTP POST リクエストを送信します。

```yaml
- http_post:
    url: "https://api.example.com/submit"
    body:
      name: "{{ username }}"
      date: "{{ today }}"
    headers:
      Content-Type: application/json
    save_as: response
```

## http_delete / http_patch

```yaml
- http_delete:
    url: "https://api.example.com/items/{{ id }}"
    headers:
      Authorization: "Bearer {{ token }}"

- http_patch:
    url: "https://api.example.com/items/{{ id }}"
    body:
      status: "done"
```

## ftp_upload / ftp_download

FTP または FTPS でファイルを転送します。

```yaml
- ftp_upload:
    host: ftp.example.com
    port: 21
    user: "{{ ftp_user }}"
    password:
      secret_env: FTP_PASSWORD
    local: output/report.xlsx
    remote: /reports/report.xlsx
    tls: true             # FTPS (デフォルト: false)

- ftp_download:
    host: ftp.example.com
    user: "{{ ftp_user }}"
    password:
      secret_env: FTP_PASSWORD
    remote: /data/input.csv
    local: input/input.csv
```
