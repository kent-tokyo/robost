# Web / HTTP

## http_get

Send an HTTP GET request.

```yaml
- http_get:
    url: "https://api.example.com/data"
    headers:
      Authorization: "Bearer {{ token }}"
    save_as: response     # { status, body, headers }
```

## http_post

Send an HTTP POST request with a JSON body.

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

Transfer files over FTP or FTPS.

```yaml
- ftp_upload:
    host: ftp.example.com
    port: 21
    user: "{{ ftp_user }}"
    password:
      secret_env: FTP_PASSWORD
    local: output/report.xlsx
    remote: /reports/report.xlsx
    tls: true             # FTPS (default: false)

- ftp_download:
    host: ftp.example.com
    user: "{{ ftp_user }}"
    password:
      secret_env: FTP_PASSWORD
    remote: /data/input.csv
    local: input/input.csv
```
