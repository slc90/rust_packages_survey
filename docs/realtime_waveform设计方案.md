# 实时波形方案

## 1. 概述

实时波形功能用于在界面上实时显示多通道生理信号波形（如脑电、心电等）。采用Bevy的ECS架构，数据生成与渲染分离，通过异步消息机制实现前后台通信。

## 2. 架构设计

### 2.1 模块结构

```
ui/src/homepage/realtime_plot/
├── components.rs    # UI组件定义
├── systems.rs       # 生命周期系统和渲染逻辑
├── plugin.rs        # 插件定义
├── resources.rs     # 波形数据资源
└── data_generator.rs # 后台数据生成器（未来扩展）
```

### 2.2 核心组件

| 组件/资源 | 用途 |
|-----------|------|
| `WaveformData` | 波形数据资源，存储各通道数据 |
| `ChannelCount` | 通道数量配置 |
| `SampleRate` | 采样率配置 |
| `WaveformSettings` | 波形显示设置（颜色、线宽等） |
| `WaveformMeshMarker` | 波形网格实体标记 |
| `ControlPanelMarker` | 控制面板标记 |

### 2.3 消息通信

| 消息类型 | 方向 | 用途 |
|----------|------|------|
| `WaveformDataMessage` | 后台→前台 | 传递新生成的波形数据 |
| `UpdateSettingsMessage` | 前台→后台 | 更新通道数、采样率等设置 |

## 3. 功能设计

### 3.1 画图功能

**实现方案**：基于Bevy的2D渲染系统，使用`Polyline2d`绑定实时更新Mesh

- 使用`Mesh2d` + `MeshMaterial2d`渲染波形曲线
- 每个通道独立一个实体，使用不同颜色区分
- 采用环形缓冲区存储数据，控制内存占用
- 支持滚动显示（新数据从右移入，左侧数据移出）

**数据结构**：
```rust
// 波形数据资源
struct WaveformData {
    channels: Vec<Vec<f32>>,  // 各通道数据
    max_points: usize,         // 最大显示点数
}

// 环形缓冲区实现
struct RingBuffer<T> {
    data: Vec<T>,
    write_pos: usize,
    capacity: usize,
}
```

### 3.2 设置通道

**UI控件**：数字输入框 + 滑动条（范围1-32）

**功能**：
- 动态添加/删除通道对应的波形实体
- 通道堆叠显示，用不同颜色区分
- 配置保存到 `config.json`

**UI布局**：
```
[通道数: [1] ====○==== 16]
```

### 3.3 设置采样率

**UI控件**：下拉选择框（500, 1000, 2000, 4000 Hz）

**功能**：
- 控制数据生成速度
- 影响时间轴刻度显示

**UI布局**：
```
[采样率: [▼ 1000 Hz]]
```

### 3.4 后台随机数生成

**实现方案**：使用Bevy的异步任务系统

- 使用 `AsyncTask` 在后台线程生成数据
- 生成高斯分布的随机数模拟生理信号
- 可配置的幅度和噪声级别

**数据生成逻辑**：
```rust
// 伪随机数生成（可替换为真实数据源）
fn generate_random_waveform(sample_count: usize) -> Vec<f32> {
    // 使用_rand crate 生成高斯噪声
    // 叠加正弦波模拟真实信号
}
```

### 3.5 前后台异步消息通信

**机制**：使用Bevy的 `MessageReader` / `MessageWriter`

**数据流**：
```
[后台任务] --WaveformDataMessage--> [前台系统] --> [更新Mesh]
```

**实现**：
1. 后台任务通过 `MessageWriter<WaveformDataMessage>` 发送数据
2. 前台系统通过 `MessageReader<WaveformDataMessage>` 读取并更新渲染

## 4. 文件清单

| 文件 | 职责 |
|------|------|
| `components.rs` | 定义UI组件标记（控制面板、设置项等） |
| `resources.rs` | 定义 `WaveformData` 资源和消息类型 |
| `systems.rs` | 实现波形生成、数据更新、UI渲染系统 |
| `plugin.rs` | 插件入口，注册所有系统和资源 |

## 5. 实现步骤

### Phase 1: 基础框架

- [x] 1.1 在 `resources.rs` 中定义 `WaveformData` 结构体
  - 包含 `channels: Vec<Vec<f32>>` 存储各通道数据
  - 包含 `max_points: usize` 最大显示点数
- [x] 1.2 在 `resources.rs` 中定义 `WaveformDataMessage` 消息类型
- [x] 1.3 在 `resources.rs` 中定义 `WaveformSettingsMessage` 消息类型（用于前台→后台设置更新）
- [x] 1.4 实现 `RingBuffer<T>` 数据结构
  - 实现 `new(capacity: usize)` 构造方法
  - 实现 `push(&mut self, item: T)` 写入方法
  - 实现 `get_all(&self) -> &[T]` 获取全部数据方法
- [x] 1.5 在 `components.rs` 中添加组件标记
  - `WaveformMeshMarker` 波形网格实体标记
  - `ControlPanelMarker` 控制面板标记
  - `ChannelSliderMarker` 通道滑块标记
  - `SampleRateDropdownMarker` 采样率下拉框标记
- [x] 1.6 在 `systems.rs` 中实现 `on_enter()` 生命周期函数
  - 初始化 `WaveformData` 资源
- [x] 1.7 在 `systems.rs` 中实现 `on_exit()` 生命周期函数
  - 清理波形相关实体

### Phase 2: 数据生成

- [x] 2.1 在 `Cargo.toml` 中添加 `rand` 依赖
- [x] 2.2 在 `resources.rs` 中实现 `WaveformGenerator` 结构体
  - 包含采样率、幅度、噪声级别等参数
- [x] 2.3 实现 `WaveformGenerator::generate()` 方法
  - 生成均匀分布随机数（替代高斯分布）
  - 叠加正弦波模拟真实信号
- [x] 2.4 实现 `WaveformGenerator::generate_single()` 方法
- [ ] 2.5 在 `systems.rs` 中实现数据生成系统 `generate_waveform_data`
  - 使用 `Commands` 发送 `WaveformDataMessage`
- [ ] 2.6 在 `systems.rs` 中实现数据接收系统 `receive_waveform_data`
  - 使用 `MessageReader<WaveformDataMessage>` 读取数据
  - 更新 `WaveformData` 资源
- [ ] 2.7 配置定时器系统
  - 根据采样率设置定时触发间隔

### Phase 3: 渲染绑定

- [ ] 3.1 在 `systems.rs` 中实现 `spawn_waveform_mesh()` 系统
  - 为每个通道创建独立的 `Mesh2d` 实体
  - 绑定不同颜色材质
- [ ] 3.2 在 `systems.rs` 中实现 `update_waveform_mesh()` 系统
  - 从 `WaveformData` 读取数据
  - 更新 `Polyline2d` Mesh
- [ ] 3.3 在 `systems.rs` 中实现多通道颜色映射
  - 定义颜色数组：`["#3498db", "#e74c3c", "#2ecc71", "#f1c40f", ...]`
  - 根据通道索引选择颜色
- [ ] 3.4 在 `systems.rs` 中实现滚动显示逻辑
  - 新数据从右侧移入
  - 旧数据从左侧移出
- [ ] 3.5 在 `systems.rs` 中实现坐标轴和网格绘制
  - X轴时间刻度
  - Y轴幅度刻度

### Phase 4: UI控制

- [ ] 4.1 在 `components.rs` 中添加控制面板组件标记
- [ ] 4.2 在 `systems.rs` 中实现控制面板 UI 生成系统
  - 创建控制面板容器
- [ ] 4.3 在 `systems.rs` 中实现通道数滑动条
  - 范围 1-32
  - 实时响应通道数变化
  - 动态增删波形实体
- [ ] 4.4 在 `systems.rs` 中实现采样率下拉选择框
  - 选项：500, 1000, 2000, 4000 Hz
- [ ] 4.5 实现前台→后台设置更新
  - 滑动条/下拉框变化时发送 `WaveformSettingsMessage`
- [ ] 4.6 在 `plugin.rs` 中注册所有新系统
- [ ] 4.7 更新 `config.json` 配置结构
  - 添加 `waveform` 配置项
- [ ] 4.8 实现设置持久化
  - 启动时加载配置
  - 修改时保存配置

## 6. 配置扩展

在 `config.json` 中增加波形配置：

```json
{
    "waveform": {
        "channel_count": 8,
        "sample_rate": 1000,
        "buffer_size": 4096,
        "colors": ["#3498db", "#e74c3c", "#2ecc71", "#f1c40f"]
    }
}
```

## 7. 依赖清单

| 依赖 | 用途 |
|------|------|
| `rand` | 随机数生成 |
| `rand_distr` | 高斯分布随机数 |
| 已有的 `bevy` | 渲染和ECS |
| 已有的 `serde_json` | 配置序列化 |
