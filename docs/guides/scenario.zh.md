# 场景格式

场景是包含 `name`、可选 `variables` 和 `steps` 序列的 YAML 文件。

```yaml
name: my_scenario

variables:
  server: "192.168.1.1"
  user: "admin"

steps:
  - wait_image:
      template: login.png
      timeout_ms: 5000
  - type: "{{ user }}"
  - press: Tab
  - type:
      secret_env: MY_PASSWORD
  - press: Enter
```

## 顶层键

| 键 | 必填 | 说明 |
|----|------|------|
| `name` | 是 | 场景标识符 |
| `variables` | 否 | 键值对，以 `{{ name }}` 方式引用 |
| `steps` | 是 | 有序的步骤节点列表 |

## 变量插值

在任意字符串值中使用 `{{ var_name }}`：

```yaml
- type: "Hello {{ username }}!"
- file_copy:
    src: "{{ base_dir }}/input.xlsx"
    dst: "{{ base_dir }}/output.xlsx"
```

## 密钥

切勿在 YAML 文件中明文存放密码。请使用环境变量：

```yaml
- type:
    secret_env: MY_PASSWORD    # 运行时读取 $MY_PASSWORD
```

## 步骤结果存储

大多数步骤支持 `save_as`，可将结果保存到变量中：

```yaml
- find_image:
    template: button.png
    save_as: btn_pos        # 保存 { x, y } 坐标

- get_datetime:
    format: "%Y%m%d"
    save_as: today          # 保存例如 "20241215"
```
