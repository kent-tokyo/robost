# 快速入门

## 1. 启动编辑器

```bash
cargo run -p robost-editor
```

## 2. 创建第一个场景

1. 在工具栏中输入**场景名称**
2. 从**节点**面板（左侧）将节点拖入**步骤**区域，或双击以追加节点
3. 选择步骤，在中央面板中编辑其属性
4. 按**保存**（Cmd+S）将 YAML 文件写入磁盘
5. 按**运行**（F5）执行场景

## 3. 直接编写 YAML

robost 场景是纯 YAML 文件。创建 `hello.yaml`：

```yaml
name: hello
steps:
  - wait_ms: 500
  - type: "Hello from robost!"
  - press: Enter
```

运行它：

```bash
cargo run -p robost-cli -- run hello.yaml
```

## 4. 使用变量

```yaml
name: with_variables
variables:
  target_app: "Notepad"
steps:
  - wait_window:
      title_contains: "{{ target_app }}"
      timeout_ms: 10000
  - type: "Automated by robost"
```

## 5. 采集模板图像

1. 打开目标应用程序
2. 从托盘启动 `robost-snip`，或运行 `cargo run -p robost-snip`
3. 按 **Ctrl+Shift+C** 冻结屏幕
4. 框选目标 UI 元素的矩形区域
5. 模板 PNG 将保存到项目文件夹
