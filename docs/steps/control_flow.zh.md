# 控制流

## if

条件分支。条件为真时执行 `then` 步骤，否则执行 `else` 步骤。

```yaml
- if:
    cond: "{{ status }} == \"done\""
  then:
    - type: "Completed!"
  else:
    - type: "Still running..."
```

## foreach

遍历列表变量。当前元素可通过 `{{ item }}` 访问。

```yaml
- foreach:
    var: __rows__        # 来自 import_vars 等步骤的列表变量
    do:
      - type: "{{ item[0] }}"
      - press: Tab
      - type: "{{ item[1] }}"
      - press: Enter
```

## repeat

重复执行步骤 N 次。循环计数器为 `{{ __i__ }}`（从 0 开始）。

```yaml
- repeat:
    count: 5
    do:
      - wait_ms: 1000
      - screenshot_save:
          path: "frame_{{ __i__ }}.png"
```

## while

当条件为真时循环。每次迭代前检查条件。

```yaml
- while:
    cond: "found == false"
    do:
      - wait_ms: 500
      - find_image:
          template: ready.png
          save_as: found
```

## do_while

当条件为真时循环。每次迭代后检查条件（至少执行一次）。

```yaml
- do_while:
    cond: "retry == true"
    do:
      - click_image:
          template: submit.png
      - wait_ms: 2000
```

## try_catch

执行 `try` 步骤；若任何步骤失败，则执行 `catch` 步骤。

```yaml
- try_catch:
    try:
      - wait_image:
          template: success.png
          timeout_ms: 3000
    catch:
      - screenshot_save:
          path: error.png
      - exit: ~
```

## group

将步骤命名为一个逻辑分组（用于在编辑器中提高可读性和折叠显示）。

```yaml
- group:
    name: "Login sequence"
    steps:
      - wait_image:
          template: login.png
      - click_image:
          template: login.png
```

## switch

根据变量值进行多路分支。

```yaml
- switch:
    on: "{{ env }}"
    cases:
      - when: prod
        do:
          - type: "production"
      - when: staging
        do:
          - type: "staging"
```

## sub_scenario / call_scenario

引入或调用另一个场景文件。

```yaml
- sub_scenario:
    path: common/login.yaml

- call_scenario:
    path: modules/export.yaml
    inputs:
      output_dir: "{{ today }}"
```

## exit

立即终止场景。

```yaml
- if:
    cond: "error == true"
  then:
    - exit: ~
```

## break / continue

退出循环或跳至下一次迭代。

```yaml
- foreach:
    var: items
    do:
      - if:
          cond: "{{ item }} == \"skip\""
        then:
          - continue: ~
      - type: "{{ item }}"
```
