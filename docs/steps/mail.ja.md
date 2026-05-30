# メール

## smtp_send

SMTP でメールを送信します。

```yaml
- smtp_send:
    host: smtp.example.com
    port: 587
    user: "{{ smtp_user }}"
    password:
      secret_env: SMTP_PASSWORD
    from: "robot@example.com"
    to:
      - "manager@example.com"
    cc:
      - "team@example.com"
    subject: "Daily Report {{ today }}"
    body: "Please find the report attached."
    attachments:
      - "output/report_{{ today }}.xlsx"
    tls: starttls         # starttls | ssl | none (デフォルト: starttls)
```

## imap_receive

IMAP メールボックスからメールを取得します。

```yaml
- imap_receive:
    host: imap.example.com
    port: 993
    user: "{{ imap_user }}"
    password:
      secret_env: IMAP_PASSWORD
    folder: INBOX
    unseen_only: true
    save_as: emails       # { subject, from, date, body, attachments } のリスト
    tls: true
```
