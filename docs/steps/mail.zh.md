# 邮件

## smtp_send

通过 SMTP 发送电子邮件。

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
    tls: starttls         # starttls | ssl | none（默认: starttls）
```

## imap_receive

从 IMAP 邮箱获取电子邮件。

```yaml
- imap_receive:
    host: imap.example.com
    port: 993
    user: "{{ imap_user }}"
    password:
      secret_env: IMAP_PASSWORD
    folder: INBOX
    unseen_only: true
    save_as: emails       # { subject, from, date, body, attachments } 的列表
    tls: true
```
