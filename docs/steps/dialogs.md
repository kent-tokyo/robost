# Dialogs

Dialog steps pause the scenario and show a popup to the operator.

## dialog_wait

Show a message and wait for the operator to click OK.

```yaml
- dialog_wait:
    message: "Please insert the USB drive, then click OK."
    title: "Action required"   # optional
```

## dialog_input

Ask the operator to type a value.

```yaml
- dialog_input:
    message: "Enter the report date (YYYYMMDD):"
    save_as: report_date
```

The operator's input is stored in `report_date`.

## dialog_select

Ask the operator to pick from a list.

```yaml
- dialog_select:
    message: "Select target environment:"
    options:
      - Production
      - Staging
      - Development
    save_as: target_env
```
