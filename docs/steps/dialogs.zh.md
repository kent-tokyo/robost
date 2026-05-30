# 对话框

对话框步骤会暂停场景，并向操作人员显示弹窗。

## dialog_wait

显示消息并等待操作人员点击确定。

```yaml
- dialog_wait:
    message: "Please insert the USB drive, then click OK."
    title: "Action required"   # 可选
```

## dialog_input

请求操作人员输入一个值。

```yaml
- dialog_input:
    message: "Enter the report date (YYYYMMDD):"
    save_as: report_date
```

操作人员的输入保存到 `report_date`。

## dialog_select

请求操作人员从列表中选择一项。

```yaml
- dialog_select:
    message: "Select target environment:"
    options:
      - Production
      - Staging
      - Development
    save_as: target_env
```
