# UIA (Windows UI Automation)

UIA steps interact with Windows accessibility APIs directly, without image recognition. They work even on obscured windows and are more reliable than pixel-based operations for standard Win32/WPF/WinForms controls.

!!! note
    UIA steps are Windows-only. On other platforms they return an `Unsupported` error.

## uia_get

Read a property of a UI element.

```yaml
- uia_get:
    by:
      name: "Username"    # accessibility label
    property: value       # value | name | class
    save_as: username_text
```

Selector options:

| Key | Description |
|-----|-------------|
| `name` | Element's Name property (accessibility label) |
| `id` | Element's AutomationId |
| `class` | Element's ClassName |

## uia_set

Set the value of a text field.

```yaml
- uia_set:
    by:
      name: "Password"
    value: "{{ password }}"
```

## uia_click

Click (invoke) a button or interactive element.

```yaml
- uia_click:
    by:
      id: "btnSubmit"
```

## uia_find

Find an element and store a reference for later use.

```yaml
- uia_find:
    by:
      class: "Edit"
    save_as: input_handle
```

## uia_select

Select an item in a ComboBox or ListBox by its display name.

```yaml
- uia_select:
    by:
      name: "Country"
    item: "Japan"
```

## uia_check

Set the checked state of a checkbox.

```yaml
- uia_check:
    by:
      name: "Agree to terms"
    checked: true
```
