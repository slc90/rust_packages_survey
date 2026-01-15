# 主页内容区域切换功能实现方案

## 概述

本方案提供了一个基于 Bevy 状态机和消息系统的动态内容区域切换机制。该机制允许应用程序在标题栏下方的同一个内容区域内显示不同的功能界面（如关于页面、测试页面等），通过菜单栏选择进行无缝切换。

## 实现状态
- 状态管理：使用 Bevy 状态机 (`States`) 管理当前显示的功能
- 消息系统：通过消息 (`Message`) 驱动状态切换
- 组件化：每个功能有独立的组件和生命周期管理
- 插件化架构：每个功能作为独立插件集成到主系统中

## 架构设计

### 模块结构
```
ui/src/homepage/
├── common.rs            # 公共组件和状态定义
├── plugin.rs            # 主页主插件（状态管理）
├── about/               # 关于功能
│   ├── components.rs    # 关于页面组件
│   ├── systems.rs       # 关于页面系统（on_enter/on_exit）
│   └── plugin.rs        # 关于页面插件
└── test/                # 测试功能（示例）
    ├── components.rs    # 测试页面组件
    ├── systems.rs       # 测试页面系统
    └── plugin.rs        # 测试页面插件
```

### 数据流
```
菜单栏点击 → 发送ChangeFunctionMessage → 状态机处理消息 → 切换Functions状态
↓
新状态触发OnEnter系统 → 清理旧内容 → 创建新功能界面
```

## 核心组件

### 1. 状态枚举 (`Functions`)
```rust
/// 所有功能的枚举，用来切换ContentArea
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default, Reflect)]
pub enum Functions {
    #[default]
    About,  // 关于页面
    Test,   // 测试页面
}
```

### 2. 内容区域标记组件
```rust
/// Marker component for the content area container
/// Used to identify the main content area entity in queries
/// This area appears below the title bar and displays different UI content based on state
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ContentAreaMarker;
```

### 3. 功能切换消息
```rust
/// 用于触发功能切换的消息
#[derive(Message, Clone)]
pub struct ChangeFunctionMessage(pub Functions);
```

### 4. 功能页面标记组件（示例）
```rust
/// About页面标记组件
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct AboutContentMarker;

/// Test页面标记组件
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct TestContentMarker;
```

## 状态切换机制

### 1. 消息处理系统
```rust
/// 接收切换ContentArea内容的消息
fn change_function(
    mut messages: MessageReader<ChangeFunctionMessage>,
    current_state: Res<State<Functions>>,
    mut next_state: ResMut<NextState<Functions>>,
) {
    if messages.is_empty() {
        return;
    }
    for message in messages.read() {
        let current_function = current_state.get();
        let new_function = &message.0;
        if current_function != new_function {
            next_state.set(new_function.clone());
        } else {
            debug!("当前功能和新功能相同:{:?},无需切换", current_function);
        }
    }
}
```

### 2. 状态生命周期管理
- **进入状态**：`OnEnter(Functions::About)` 触发 `on_enter` 系统，创建对应界面
- **离开状态**：`OnExit(Functions::About)` 触发 `on_exit` 系统，清理界面组件
- **状态不变**：相同状态切换被忽略，避免不必要的重新渲染

## 消息系统

### 消息发送（菜单栏事件处理）
```rust
// 在菜单栏系统中发送切换消息
writer.write(ChangeFunctionMessage(Functions::About));
```

### 消息接收（状态切换处理）
```rust
// 在主插件中注册消息并添加处理系统
app.add_message::<ChangeFunctionMessage>();
app.add_systems(Update, change_function);
```

## 插件架构

### 1. 主页主插件 (`HomepagePlugin`)
```rust
/// Plugin for the homepage system
pub struct HomepagePlugin;

impl Plugin for HomepagePlugin {
    fn build(&self, app: &mut App) {
        // 注册公共组件
        app.register_type::<ContentAreaMarker>();
        // 初始化状态
        app.init_state::<Functions>();
        // 注册切换功能的消息
        app.add_message::<ChangeFunctionMessage>();
        // 添加子功能插件
        app.add_plugins(AboutPlugin);
        app.add_plugins(TestPlugin);
        // 注册消息处理系统
        app.add_systems(Update, change_function);
    }
}
```

### 2. 功能子插件（以 AboutPlugin 为例）
```rust
/// Plugin for the About state
pub struct AboutPlugin;

impl Plugin for AboutPlugin {
    fn build(&self, app: &mut App) {
        // 注册About组件
        app.register_type::<AboutContentMarker>();
        // 添加生命周期系统
        app.add_systems(OnEnter(Functions::About), on_enter)
            .add_systems(OnExit(Functions::About), on_exit);
    }
}
```

## 系统流程

### 1. 初始化流程
```
App启动 → HomepagePlugin初始化
  ↓
注册Functions状态 → 注册ChangeFunctionMessage消息
  ↓
添加AboutPlugin和TestPlugin → 注册各自组件和系统
  ↓
默认进入Functions::About状态 → 触发on_enter系统
```

### 2. 切换流程
```
用户点击菜单栏"About" → 发送ChangeFunctionMessage(Functions::About)
  ↓
change_function系统接收消息
  ↓
检查当前状态 != 新状态 → 设置新状态
  ↓
触发Functions::Test的on_exit → 清理Test内容
  ↓
触发Functions::About的on_enter → 创建About内容
```

### 3. 内容管理流程
```rust
// 进入About页面
pub fn on_enter(mut commands: Commands, query: Query<Entity, With<ContentAreaMarker>>) {
    // 获取内容区域的实体
    if let Ok(content_area) = query.single() {
        // 在内容区域中创建About界面
        commands.entity(content_area).with_children(|parent| {
            parent.spawn((
                AboutContentMarker,
                Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                Text::new("About Page\nThis is the About section"),
                TextColor::BLACK,
            ));
        });
    }
}

// 离开About页面
pub fn on_exit(mut commands: Commands, query: Query<Entity, With<AboutContentMarker>>) {
    // 清理所有About内容实体
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
```

## 扩展新功能

### 步骤1：扩展Functions枚举
```rust
// 在common.rs中
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default, Reflect)]
pub enum Functions {
    #[default]
    About,
    Test,
    NewFeature,  // 新增功能
}
```

### 步骤2：创建新功能模块
```
ui/src/homepage/
└── new_feature/          # 新功能模块
    ├── components.rs     # 组件定义
    ├── systems.rs       # 系统定义（on_enter/on_exit）
    └── plugin.rs        # 插件定义
```

### 步骤3：实现插件
```rust
// new_feature/plugin.rs
pub struct NewFeaturePlugin;

impl Plugin for NewFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NewFeatureContentMarker>()
            .add_systems(OnEnter(Functions::NewFeature), on_enter)
            .add_systems(OnExit(Functions::NewFeature), on_exit);
    }
}
```

### 步骤4：在主插件中注册
```rust
// 在HomepagePlugin中
app.add_plugins(NewFeaturePlugin);
```

### 步骤5：在菜单栏中添加入口
```rust
// 在menu_bar/systems.rs中
(
    FunctionMenuItemBundle::default(),
    observe(|_activated: On<Activate>, mut writer: MessageWriter<ChangeFunctionMessage>| {
        writer.write(ChangeFunctionMessage(Functions::NewFeature));
    }),
    children![(Text::new("New Feature"), TextColor(Color::BLACK),)],
)
```
