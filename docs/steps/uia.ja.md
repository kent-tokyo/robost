# UIA (Windows UI Automation)

UIA ステップは画像認識を使わずに Windows アクセシビリティ API を直接操作します。隠れているウィンドウでも動作し、標準的な Win32/WPF/WinForms コントロールに対してピクセルベースの操作より高い安定性を発揮します。

!!! note
    UIA ステップは Windows 専用です。他のプラットフォームでは `Unsupported` エラーが返されます。

## uia_get

UI 要素のプロパティを読み取ります。

```yaml
- uia_get:
    by:
      name: "Username"    # アクセシビリティラベル
    property: value       # value | name | class
    save_as: username_text
```

セレクターのオプション:

| キー | 説明 |
|-----|-------------|
| `name` | 要素の Name プロパティ (アクセシビリティラベル) |
| `id` | 要素の AutomationId |
| `class` | 要素の ClassName |

## uia_set

テキストフィールドに値をセットします。

```yaml
- uia_set:
    by:
      name: "Password"
    value: "{{ password }}"
```

## uia_click

ボタンやインタラクティブな要素をクリック (呼び出し) します。

```yaml
- uia_click:
    by:
      id: "btnSubmit"
```

## uia_find

要素を検索して後で使用するための参照を保存します。

```yaml
- uia_find:
    by:
      class: "Edit"
    save_as: input_handle
```

## uia_select

ComboBox または ListBox でアイテムを表示名で選択します。

```yaml
- uia_select:
    by:
      name: "Country"
    item: "Japan"
```

## uia_check

チェックボックスのチェック状態をセットします。

```yaml
- uia_check:
    by:
      name: "Agree to terms"
    checked: true
```
