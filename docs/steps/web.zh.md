# Web / HTTP

## http_get

发送 HTTP GET 请求。

```yaml
- http_get:
    url: "https://api.example.com/data"
    headers:
      Authorization: "Bearer {{ token }}"
    save_as: response     # { status, body, headers }
```

## http_post

发送带有 JSON 请求体的 HTTP POST 请求。

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

通过 FTP 或 FTPS 传输文件。

```yaml
- ftp_upload:
    host: ftp.example.com
    port: 21
    user: "{{ ftp_user }}"
    password:
      secret_env: FTP_PASSWORD
    local: output/report.xlsx
    remote: /reports/report.xlsx
    tls: true             # FTPS（默认: false）

- ftp_download:
    host: ftp.example.com
    user: "{{ ftp_user }}"
    password:
      secret_env: FTP_PASSWORD
    remote: /data/input.csv
    local: input/input.csv
```
