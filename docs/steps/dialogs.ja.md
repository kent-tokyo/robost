# ダイアログ

ダイアログステップはシナリオを一時停止し、オペレーターにポップアップを表示します。

## dialog_wait

メッセージを表示してオペレーターが OK をクリックするまで待機します。

```yaml
- dialog_wait:
    message: "Please insert the USB drive, then click OK."
    title: "Action required"   # 省略可
```

## dialog_input

オペレーターに値の入力を促します。

```yaml
- dialog_input:
    message: "Enter the report date (YYYYMMDD):"
    save_as: report_date
```

オペレーターが入力した値は `report_date` に格納されます。

## dialog_select

オペレーターにリストから選択させます。

```yaml
- dialog_select:
    message: "Select target environment:"
    options:
      - Production
      - Staging
      - Development
    save_as: target_env
```
