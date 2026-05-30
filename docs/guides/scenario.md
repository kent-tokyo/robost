# Scenario Format

Scenarios are YAML files with a `name`, optional `variables`, and a `steps` sequence.

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

## Top-level keys

| Key | Required | Description |
|-----|----------|-------------|
| `name` | yes | Scenario identifier |
| `variables` | no | Key-value pairs available as `{{ name }}` |
| `steps` | yes | Ordered list of step nodes |

## Variable interpolation

Use `{{ var_name }}` anywhere in string values:

```yaml
- type: "Hello {{ username }}!"
- file_copy:
    src: "{{ base_dir }}/input.xlsx"
    dst: "{{ base_dir }}/output.xlsx"
```

## Secrets

Never put passwords in the YAML file. Use environment variables:

```yaml
- type:
    secret_env: MY_PASSWORD    # reads $MY_PASSWORD at runtime
```

## Step result storage

Most steps accept `save_as` to store the result in a variable:

```yaml
- find_image:
    template: button.png
    save_as: btn_pos        # saves { x, y } coordinates

- get_datetime:
    format: "%Y%m%d"
    save_as: today          # saves e.g. "20241215"
```
