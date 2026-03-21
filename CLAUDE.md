# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 Bevy 游戏引擎的 Rust GUI 应用，用于医学数据可视化（波形、影像等）。采用 Bevy 的 ECS 架构组织代码。

## 常用命令

```bash
# 构建（dev 模式，使用动态链接加速编译）
cargo build

# 构建 release 版本
cargo build --release

# 运行
cargo run

# 运行单个测试
cargo test -p <crate_name> -- --test-threads=1 <test_name>

# 代码检查
cargo clippy --workspace --all-targets --all-features

# 代码格式化
cargo fmt --all

# 检查格式化
cargo fmt --all -- --check

# 搜索某个包
cargo search xxx
```

## 架构

### Workspace 结构
- `entry/`: 程序入口点
- `ui/`: UI 组件（Bevy 插件），包含：
  - `homepage/`: 主页功能模块（About、Setting、RealtimePlot）
  - `title_bar/`: 自定义标题栏
  - `menu_bar/`: 菜单栏
- `config/`: 配置管理（读取/写入 JSON 配置）
- `i18n/`: 国际化支持
- `logger/`: 日志库
- `embedded_assets/`: 嵌入式资源（字体、图标）
- `utils/`: 工具库

### 核心模式
- 使用 Bevy 的 ECS（Entity Component System）架构
- UI 通过 Bevy 的 `UiBundle` 和自定义组件构建
- 状态管理使用 `State<Functions>` 和 `NextState`
- 插件系统：`Plugin` trait 用于模块化

### 配置文件
- 配置文件位于 `config/config_file/config.json`
- 程序启动时从 exe 同级目录读取配置
- 使用 `serde_json` 进行序列化/反序列化

### UI 层次结构
1. **TitleBar** (顶部): Logo + 标题 + MenuBar + 窗口控制按钮
2. **ContentArea** (下方): 根据当前 Functions 状态显示不同页面

### 日志
- 使用 `tracing` crate
- 通过 `custom_layer` 和 `fmt_layer` 自定义输出格式
- 日志级别通过 `RUST_LOG` 环境变量或 `LogPlugin` 配置

## 开发注意事项

- Windows release 模式使用 `windows_subsystem = "windows"` 隐藏控制台
- 代码中禁止使用 `unwrap()` 和 `expect()`（由 clippy 强制检查）
- 使用 `MessageReader` 进行跨系统通信
- 配置路径使用 `env::current_exe()` 获取 exe 所在目录

## 其他规范
1. 修改代码后记得更新 `structure.md`
2. 代码增加注释，使用中文
3. 根据方案完成后记得修改 `docs/requirements.md` 中对应部分
4. 提交前不能跳过hook
5. git message用中文，要写清楚功能和修复的问题
