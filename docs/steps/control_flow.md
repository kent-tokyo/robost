# Control Flow

## if

Conditional branch. Executes `then` steps if the condition is truthy, otherwise `else` steps.

```yaml
- if:
    cond: "{{ status }} == \"done\""
  then:
    - type: "Completed!"
  else:
    - type: "Still running..."
```

## foreach

Iterate over a list variable. The current item is available as `{{ item }}`.

```yaml
- foreach:
    var: __rows__        # list variable from e.g. import_vars
    do:
      - type: "{{ item[0] }}"
      - press: Tab
      - type: "{{ item[1] }}"
      - press: Enter
```

## repeat

Repeat steps N times. The loop counter is `{{ __i__ }}` (0-based).

```yaml
- repeat:
    count: 5
    do:
      - wait_ms: 1000
      - screenshot_save:
          path: "frame_{{ __i__ }}.png"
```

## while

Loop while a condition is true. Check before each iteration.

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

Loop while a condition is true. Check after each iteration (runs at least once).

```yaml
- do_while:
    cond: "retry == true"
    do:
      - click_image:
          template: submit.png
      - wait_ms: 2000
```

## try_catch

Execute `try` steps; if any step fails, execute `catch` steps instead.

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

Name a logical group of steps (used for clarity and collapsing in editor).

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

Multi-branch on a variable value.

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

Include or call another scenario file.

```yaml
- sub_scenario:
    path: common/login.yaml

- call_scenario:
    path: modules/export.yaml
    inputs:
      output_dir: "{{ today }}"
```

## exit

Terminate the scenario immediately.

```yaml
- if:
    cond: "error == true"
  then:
    - exit: ~
```

## break / continue

Exit a loop or skip to the next iteration.

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
