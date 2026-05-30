# UIA（Windows UI 自动化）

UIA 步骤直接与 Windows 辅助功能 API 交互，无需图像识别。即使在被遮挡的窗口上也能正常工作，对于标准 Win32/WPF/WinForms 控件，其可靠性高于基于像素的操作。

!!! note
    UIA 步骤仅支持 Windows。在其他平台上将返回 `Unsupported` 错误。

## uia_get

读取 UI 元素的属性值。

```yaml
- uia_get:
    by:
      name: "Username"    # 无障碍标签
    property: value       # value | name | class
    save_as: username_text
```

选择器选项：

| 键 | 说明 |
|----|------|
| `name` | 元素的 Name 属性（无障碍标签） |
| `id` | 元素的 AutomationId |
| `class` | 元素的 ClassName |

## uia_set

设置文本字段的值。

```yaml
- uia_set:
    by:
      name: "Password"
    value: "{{ password }}"
```

## uia_click

点击（调用）按钮或交互元素。

```yaml
- uia_click:
    by:
      id: "btnSubmit"
```

## uia_find

查找元素并保存引用以供后续使用。

```yaml
- uia_find:
    by:
      class: "Edit"
    save_as: input_handle
```

## uia_select

通过显示名称在 ComboBox 或 ListBox 中选择项目。

```yaml
- uia_select:
    by:
      name: "Country"
    item: "Japan"
```

## uia_check

设置复选框的选中状态。

```yaml
- uia_check:
    by:
      name: "Agree to terms"
    checked: true
```
