# 设置页面实现方案

## 架构设计

### 模块结构

```
ui/src/homepage/setting/
├── components.rs     # ECS 组件定义
├── systems.rs        # 系统实现
└── plugin.rs         # 插件定义
```

### 系统架构

```
设置页面插件 (SettingPlugin)
├── 生命周期系统
│   ├── on_enter: 创建设置界面
│   └── on_exit: 清理资源
├── 单选按钮样式系统
│   ├── update_radio_button_border_color
│   ├── update_radio_button_border_color2
│   ├── update_radio_button_mark_color
│   └── update_radio_button_mark_color2
├── 语言同步系统
│   └── sync_radio_buttons_to_language

## 单选按钮样式系统详细设计思路

### 设计背景与问题分析

在 Bevy 的单选按钮组件设计中，视觉状态同步是一个关键挑战。当用户点击单选按钮时，系统需要：

1. **边框颜色更新**：选中状态显示深绿色边框，未选中状态显示灰色边框
2. **内部标记颜色更新**：选中状态显示亮绿色实心圆，未选中状态标记不可见
3. **状态变化检测**：需要处理 `Checked` 组件的添加、改变和移除事件
4. **实时响应**：确保视觉反馈与用户操作同步，无延迟或闪烁

### 函数分工与协作策略

四个样式系统函数采用"主辅配对"的设计模式：

#### 1. `update_radio_button_border_color` - 主边框颜色更新器
**职责**：处理 `Checked` 组件的添加(`Added<Checked>`)和改变(`Changed<Checked>`)事件
**工作原理**：
- 查询所有包含 `Checked` 组件且该组件最近发生变化的中文/英文单选按钮
- 遍历查询结果，根据 `Has<Checked>` 的状态决定边框颜色
- 通过 `border_color.set_all()` 方法更新边框实体的视觉状态

#### 2. `update_radio_button_border_color2` - 辅助边框颜色更新器  
**职责**：专门处理 `Checked` 组件被移除(`RemovedComponents<Checked>`)的情况
**设计动机**：
- Bevy 的 `Changed<T>` 过滤器无法检测到组件移除事件
- 当单选按钮从选中变为未选中时，`Checked` 组件被移除而非改变
- 需要专门的系统来捕获 `RemovedComponents<Checked>` 事件
**工作原理**：
- 监听 `RemovedComponents<Checked>`，获取被移除 `Checked` 组件的实体列表
- 对每个被移除的实体查询其当前 `Has<Checked>` 状态（应为 false）
- 更新边框颜色为未选中状态（灰色）

#### 3. `update_radio_button_mark_color` - 主标记颜色更新器
**职责**：处理选中状态变化时内部实心圆的显示/隐藏
**实现特点**：
- 与边框颜色更新器使用相同的查询过滤器（`Added<Checked>` + `Changed<Checked>`）
- 需要两级实体导航：单选按钮 → 边框实体 → 标记实体
- 通过修改 `BackgroundColor` 组件实现：选中时为亮绿色(`ELEMENT_FILL`)，未选中时为透明(`Color::NONE`)

#### 4. `update_radio_button_mark_color2` - 辅助标记颜色更新器
**职责**：在 `Checked` 组件被移除时同步更新标记颜色
**设计一致性**：
- 与 `update_radio_button_border_color2` 采用相同的监听策略
- 确保边框和标记的颜色变化完全同步
- 避免视觉不一致（如边框已变灰但标记仍显示的情况）

#### 实体导航策略
每个单选按钮采用三层实体结构：
```
RadioButton实体 (包含Checked组件)
├── Border实体 (包含BorderColor组件)
│   └── Mark实体 (包含BackgroundColor组件)
└── Label实体 (包含Text组件)
```
样式系统通过 `Children` 组件逐级导航，确保精确更新目标实体。

#### 颜色常量设计
- `CHECKED_BORDER_COLOR`: 深绿色(0.2, 0.6, 0.2)，提供明显的选中状态视觉提示
- `UNCHECKED_BORDER_COLOR`: 灰色(0.45, 0.45, 0.45)，保持低调的未选中状态
- `ELEMENT_FILL`: 亮绿色(0.35, 0.75, 0.35)，与边框形成和谐的视觉层次

### 协同工作机制

四个系统在 Bevy 的调度框架中并行运行，但通过事件类型自然分离工作负载：

1. **用户点击未选中按钮** → `Checked` 组件被添加 → 触发 `update_radio_button_border_color` 和 `update_radio_button_mark_color`
2. **用户点击已选中按钮** → `Checked` 组件被移除 → 触发 `update_radio_button_border_color2` 和 `update_radio_button_mark_color2`
3. **其他单选按钮自动取消选中** → `Checked` 组件被移除 → 同样触发两个"2"后缀的系统
```
