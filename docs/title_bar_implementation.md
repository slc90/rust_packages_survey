# 自定义标题栏实现方案

## 概述

本方案提供了一个完整的自定义标题栏实现，适用于 Bevy 0.17.3 应用程序。该标题栏支持窗口拖动、最小化/最大化/关闭控制按钮、自定义样式以及完善的输入事件处理。

## 实现状态
- 自定义标题栏 UI 组件和 Bundle
- 窗口拖动功能（支持标题栏区域）
- 控制按钮（最小化/最大化/关闭）
- 防连点机制（双重保护）
- 窗口最小化恢复后的焦点管理
- 输入状态清理（防止残留按下状态）
- 插件化集成

## 架构设计

### 模块结构
```
title_bar/
├── components.rs      # 组件定义
├── resources.rs       # 资源定义
├── systems.rs         # 系统逻辑
└── plugin.rs          # 插件集成
```

## 核心组件

### 1. 标记组件
```rust
/// 标题栏容器标记组件
TitleBarMarker

/// 标题栏占位区域标记组件
TitleBarPlaceholderMarker
```

### 2. 业务逻辑组件
```rust
/// 控制按钮类型枚举
enum TitleBarButtonEnum {
    Minimize,   // 最小化
    Maximize,   // 最大化
    Restore,    // 恢复（从最大化）
    Close,      // 关闭
}

/// 防连点状态跟踪组件
struct PreviousInteraction {
    interaction: Option<Interaction>,
}
```

### 3. 样式组件
```rust
/// 标题栏样式配置
struct TitleBarStyle {
    height: f32,  // 高度（默认 40px）
}
```

### 4. Bundle 集合
- `TitleBarBundle`：完整的标题栏
- `TitleBarButtonBundle`：控制按钮
- `TitleBarTextBundle`：标题文本
- `TitleBarLogoBundle`：应用 Logo
- `TitleBarPlaceholderBundle`：占位区域

## 资源管理

### 1. 左键行为配置
```rust
/// 左键点击行为
enum LeftClickAction {
    Move,  // 移动窗口
}
```

### 2. 防连点资源
```rust
/// 按钮点击冷却资源
struct TitleBarButtonCooldown {
    timer: Timer,  // 0.3 秒冷却计时器
}
```

### 3. 窗口状态资源
```rust
/// 窗口状态跟踪资源
struct WindowState {
    was_minimized: bool,      // 窗口是否被最小化
}
```

## 系统逻辑

### 1. 窗口拖动系统 (`drag_and_move_window`)
**功能**：处理标题栏区域的窗口拖动操作

**关键逻辑**：
- 使用 `just_pressed(MouseButton::Left)` 检测左键点击
- 排除按钮区域的点击（防止与按钮功能冲突）
- 当窗口处于最大化状态时拖动，自动切换为恢复按钮状态
- 调用 `window.start_drag_move()` 启动系统级拖动

### 2. 按钮点击处理系统 (`handle_button_clicks`)
**功能**：处理控制按钮的点击事件

**按钮行为**：
- **关闭**：发送 `AppExit::Success` 事件
- **最小化**：调用 `window.set_minimized(true)`，记录状态
- **最大化**：调用 `window.set_maximized(true)`，切换为恢复按钮
- **恢复**：调用 `window.set_maximized(false)`，切换为最大化按钮

### 3. 窗口可见性处理系统 (`handle_window_visibility`)
**功能**：管理窗口最小化恢复后的焦点和输入状态

**关键逻辑**：
- 监听 `WindowFocused` 事件
- 当窗口获得焦点且之前被最小化时，自动重置鼠标左键状态

## 防连点机制

### 双重保护设计

#### 1. 全局冷却计时器
- 所有按钮共享 0.3 秒冷却时间
- 计时器运行期间跳过所有按钮处理
- 防止物理层面的快速连续点击

#### 2. 按钮状态跟踪
- 每个按钮跟踪前一次的交互状态
- 只有从"非按下"变为"按下"才视为有效点击
- 防止同一帧内的重复触发

### 实现代码
```rust
// 检查是否为新的按下事件
let is_new_press = *interaction == Interaction::Pressed
    && prev_interaction.interaction != Some(Interaction::Pressed);

// 更新前次交互状态
prev_interaction.interaction = Some(*interaction);

if is_new_press {
    cooldown.timer.reset();  // 重置冷却计时器
    // 处理按钮点击...
}
```

### 窗口焦点管理

### 问题背景
窗口从最小化恢复后，第一次鼠标点击可能被操作系统用于激活窗口而不是传递给应用程序，导致拖动功能失效。

### 解决方案
1. **状态跟踪**：通过 `WindowState` 资源记录窗口最小化状态
2. **输入状态清理**：恢复窗口时重置鼠标左键输入状态

### 关键实现
```rust
// 在 handle_window_visibility 中
if window_state.was_minimized {
    window_state.was_minimized = false;
    mouse_input.reset(MouseButton::Left);  // 清理输入状态
}
```
