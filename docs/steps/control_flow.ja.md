# 制御フロー

## if

条件分岐。条件が真であれば `then` のステップを実行し、そうでなければ `else` のステップを実行します。

```yaml
- if:
    cond: "{{ status }} == \"done\""
  then:
    - type: "Completed!"
  else:
    - type: "Still running..."
```

## foreach

リスト変数を繰り返し処理します。現在のアイテムは `{{ item }}` で参照できます。

```yaml
- foreach:
    var: __rows__        # 例: import_vars から得たリスト変数
    do:
      - type: "{{ item[0] }}"
      - press: Tab
      - type: "{{ item[1] }}"
      - press: Enter
```

## repeat

ステップを N 回繰り返します。ループカウンターは `{{ __i__ }}` (0 始まり) で参照できます。

```yaml
- repeat:
    count: 5
    do:
      - wait_ms: 1000
      - screenshot_save:
          path: "frame_{{ __i__ }}.png"
```

## while

条件が真の間ループします。各イテレーションの前に条件を確認します。

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

条件が真の間ループします。各イテレーションの後に条件を確認します (最低1回は実行されます)。

```yaml
- do_while:
    cond: "retry == true"
    do:
      - click_image:
          template: submit.png
      - wait_ms: 2000
```

## try_catch

`try` のステップを実行し、いずれかのステップが失敗した場合は代わりに `catch` のステップを実行します。

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

ステップの論理的なグループに名前を付けます (エディタでの視認性向上や折りたたみのために使用します)。

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

変数の値に応じて複数の分岐を実行します。

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

別のシナリオファイルをインクルードまたは呼び出します。

```yaml
- sub_scenario:
    path: common/login.yaml

- call_scenario:
    path: modules/export.yaml
    inputs:
      output_dir: "{{ today }}"
```

## exit

シナリオをただちに終了します。

```yaml
- if:
    cond: "error == true"
  then:
    - exit: ~
```

## break / continue

ループを終了するか、次のイテレーションにスキップします。

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
