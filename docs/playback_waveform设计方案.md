# 回放波形方案

## 1. 概述

回放波形功能用于从文件加载已记录的生理信号数据（如脑电、心电等）并进行播放浏览。采用 Bevy 的 ECS 架构，与现有实时波形功能共享渲染系统，但数据来源和控制逻辑独立。

## 2. 架构设计

### 2.1 模块结构

```
// 独立 crate：负责 EDF+ 文件读写
edf_io/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── loader.rs      # EDF+ 文件读取
│   └── generator.rs   # 测试数据生成器
└── tools/
    └── generate_test_edf.rs  # 测试数据生成工具

// 项目根目录 data/：测试数据目录
data/
└── test_64ch_4000hz_10min.edf

// UI crate：负责回放界面
ui/src/homepage/playback_plot/
├── components.rs    # UI组件标记（播放控制、分页按钮等）
├── resources.rs     # 回放数据资源和状态管理
├── systems.rs       # 生命周期、播放控制逻辑
└── plugin.rs        # 插件定义
```

**外部库**：`edfplus` - 纯Rust实现的EDF+文件读写库，位于 `edf_io` crate 中

### 2.2 与实时波形的对比

| 特性 | 实时波形 | 回放波形 |
|------|----------|----------|
| 数据来源 | 内存随机生成 | 文件读取 |
| 通道数量 | 由配置决定 | 由文件头决定 |
| 播放控制 | 连续生成 | 播放/暂停/速度控制 |
| 分页浏览 | 不支持 | 上一页/下一页 |
| 状态 | `Functions::RealtimePlot` | `Functions::PlaybackPlot`（新增） |

### 2.3 核心组件/资源

| 组件/资源 | 用途 |
|-----------|------|
| `PlaybackData` | 回放数据资源，存储文件中的波形数据 |
| `PlaybackState` | 播放状态（播放中/暂停） |
| `PlaybackSpeed` | 播放速度倍率 |
| `CurrentPage` | 当前页码 |
| `PageSize` | 每页数据点数 |
| `EdfLoader` | EDF+文件读取器（位于 `edf_io` crate） |

## 3. 功能设计

### 3.1 独立测试数据生成工具

**实现方案**：编写独立的 Rust 程序生成 EDF+ 测试文件

**文件位置**：`edf_io/tools/generate_test_edf.rs`

**生成参数**：
- 通道数：64
- 采样率：4000 Hz
- 时长：10 分钟
- 输出路径：`data/test_64ch_4000hz_10min.edf`（项目根目录下）

**功能**：
- 生成多通道（64通道）模拟 EEG 数据
- 叠加正弦波和高斯噪声模拟真实生理信号
- 写入 EDF+ 格式文件

**运行方式**：
```bash
cargo run --bin generate_test_edf -p edf_io
```

**生成器数据结构**：
```rust
// edf_io/tools/generate_test_edf.rs
struct TestEdfGenerator {
    channel_count: usize,   // 64
    sample_rate: u32,       // 4000
    duration_secs: u32,    // 600 (10分钟)
}

impl TestEdfGenerator {
    /// 生成测试用 EDF+ 文件
    pub fn generate(&self, output_path: &Path) -> Result<(), Box<dyn Error>>;
}
```

**文件格式**：EDF+ (European Data Format Plus)
- 16-bit 数据精度
- 支持多通道
- 标准的生物信号存储格式

### 3.2 读取 EDF+ 文件

**实现方案**：使用 `edfplus` crate 读取 EDF+ 文件

**功能**：
- 解析 EDF+ 文件头（通道信息、采样率、数据长度）
- 读取各通道数据到内存
- 支持大文件分块读取

**API 使用示例**：
```rust
use edfplus::{EdfReader, SignalParam};

// 读取
let mut reader = EdfReader::open("data.edf")?;
let samples = reader.read_physical_samples(0, 1000)?;

// 获取信号参数
let signal_params = reader.signal_params();
```

**封装为 Bevy 资源**：
```rust
pub struct EdfLoader {
    reader: Option<EdfReader>,
    file_path: String,
}

impl EdfLoader {
    pub fn load(&mut self, path: &str) -> Result<(), Box<dyn Error>>;
    pub fn get_channel_data(&self, channel: usize) -> Vec<f32>;
    pub fn get_page(&self, page: usize, page_size: usize) -> PlaybackData;
}
```

### 3.3 播放/暂停功能

**UI控件**：播放/暂停按钮

**功能**：
- 点击切换播放状态
- 播放状态显示按钮图标变化
- 暂停时保持当前位置

**数据结构**：
```rust
#[derive(Resource, Clone, Copy, PartialEq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
}

#[derive(Resource)]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub current_position: usize,  // 当前播放位置（采样点索引）
}
```

**播放逻辑**：
```
[Playing] --点击--> [Paused]
[Paused]  --点击--> [Playing]
```

### 3.4 设置播放速度

**UI控件**：速度选择按钮（1x / 2x / 4x）

**功能**：
- 点击循环切换速度档位
- 影响数据推进速率
- 显示当前速度

**数据结构**：
```rust
#[derive(Resource)]
pub struct PlaybackSpeed {
    pub multiplier: f32,  // 速度倍率
}

impl PlaybackSpeed {
    pub const OPTIONS: [f32; 3] = [1.0, 2.0, 4.0];
}
```

### 3.5 上一页/下一页

**UI控件**：上一页/下一页按钮

**功能**：
- 切换当前显示的数据页
- 页面大小可配置（默认 4096 点）
- 翻页时重置播放位置到页面起始
- 边界检查（首页禁用上一页，末页禁用下一页）

**数据结构**：
```rust
#[derive(Resource)]
pub struct PageInfo {
    pub current_page: usize,
    pub total_pages: usize,
    pub page_size: usize,
}
```

## 4. 回放数据结构设计

```rust
/// 回放数据资源
#[derive(Resource)]
pub struct PlaybackData {
    /// 各通道数据
    channels: Vec<Vec<f32>>,
    /// 通道数量（从EDF文件头读取）
    channel_count: usize,
    /// 采样率（从EDF文件头读取）
    sample_rate: u32,
    /// 总数据点数
    total_points: usize,
}

/// 回放播放状态
#[derive(Resource, Clone, Copy, PartialEq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
}

/// 回放控制状态
#[derive(Resource)]
pub struct PlaybackControl {
    /// 播放状态
    pub status: PlaybackStatus,
    /// 当前播放位置（采样点索引）
    pub position: usize,
    /// 播放速度倍率
    pub speed: f32,
    /// 当前页码
    pub current_page: usize,
    /// 总页数
    pub total_pages: usize,
    /// 每页数据点数
    pub page_size: usize,
}
```

**通道数量**：回放模式的通道数量从 EDF 文件头读取，不使用配置文件中的通道数设置。
```

## 5. 系统设计

### 5.1 生命周期系统

| 系统 | 触发条件 | 功能 |
|------|----------|------|
| `on_enter` | 进入回放页面 | 初始化资源、加载默认文件或显示选择界面 |
| `on_exit` | 离开回放页面 | 清理资源、保存播放位置 |

### 5.2 数据加载系统

| 系统 | 功能 |
|------|------|
| `load_edf_file` | 读取 EDF+ 文件并填充 `PlaybackData` |
| `generate_test_edf` | 生成测试用 EDF+ 文件 |
| `load_page_data` | 加载指定页的数据到显示缓存 |

### 5.3 播放控制系统

| 系统 | 功能 |
|------|------|
| `update_playback` | 根据播放状态和速度推进播放位置 |
| `handle_play_pause` | 处理播放/暂停按钮点击 |
| `handle_speed_change` | 处理速度切换按钮点击 |
| `handle_page_change` | 处理上一页/下一页按钮点击 |

### 5.4 渲染系统（复用）

| 系统 | 功能 |
|------|------|
| `init_waveform_rendering` | 初始化波形渲染资源（复用实时波形） |
| `update_waveform_display` | 更新波形显示（复用） |
| `spawn_axis_grid` | 绘制坐标轴和网格（复用） |

## 6. UI 布局设计

```
+------------------------------------------------------------------+
|  [主页] 回放波形                                                    |
+------------------------------------------------------------------+
|                                                                    |
|  +------------------+  +----------------------------------------+ |
|  | [文件] test.edf  |  |                                        | |
|  +------------------+  |         波形显示区域                     | |
|                        |                                        | |
|  [播放] [暂停]         |         - 多通道波形                     | |
|                        |         - 坐标轴和网格                   | |
|  速度: [1x]            |         - 滚动显示                       | |
|                        |                                        | |
|  位置: 12345 / 99999   |                                        | |
|                        |                                        | |
|  [<上一页] [下一页>]   |                                        | |
|                        |                                        | |
|  页码: 3 / 25          |                                        | |
|                        +----------------------------------------+ |
|                                                                    |
+------------------------------------------------------------------+
```

## 7. 配置文件扩展

在 `config.json` 中增加回放配置：

```json
{
    "waveform": {
        "channel_count": 8,
        "sample_rate": 1000,
        "buffer_size": 4096
    },
    "playback": {
        "last_file": "path/to/last_opened.edf",
        "page_size": 4096,
        "default_speed": 1.0
    }
}
```

**注意**：回放模式的通道数量从 EDF 文件头读取，不使用配置文件中 `waveform.channel_count`。

## 8. 实现步骤

### Phase 1: 创建 edf_io crate

- 创建 `edf_io/` 目录和 Cargo.toml
- 在 `edf_io/Cargo.toml` 中添加 `edfplus` 依赖
- 在 `edf_io/src/lib.rs` 中定义公开 API
- 在 `edf_io/src/loader.rs` 中实现 `EdfLoader`
- 在 `edf_io/src/generator.rs` 中实现测试数据生成器
- 在 `ui/Cargo.toml` 中添加 `edf_io` 依赖

### Phase 2: 独立测试工具

- 在 `edf_io/tools/` 下创建 `generate_test_edf.rs` 测试数据生成工具
- 运行生成 64 通道、4000 Hz、10 分钟的 EDF+ 测试文件
- 确认文件保存在 `data/test_64ch_4000hz_10min.edf`（项目根目录下）

**工具结构**：
```
edf_io/
└── tools/
    └── generate_test_edf.rs  # 测试数据生成工具
                              # 运行: cargo run --bin generate_test_edf -p edf_io

data/
└── test_64ch_4000hz_10min.edf  # 生成的测试文件（项目根目录）
```

### Phase 3: 基础框架

- 在 `common.rs` 中添加 `Functions::PlaybackPlot` 状态
- 创建 `playback_plot/` 目录结构
- 在 `resources.rs` 中定义 `PlaybackData` 结构体（通道数从文件读取）
- 在 `resources.rs` 中定义 `PlaybackControl` 资源
- 在 `resources.rs` 中定义 `PlaybackStatus` 枚举
- 在 `components.rs` 中添加 UI 组件标记
  - `PlayButtonMarker`
  - `PauseButtonMarker`
  - `SpeedButtonMarker`
  - `PrevPageButtonMarker`
  - `NextPageButtonMarker`
  - `FilePathDisplayMarker`
  - `PositionDisplayMarker`
  - `PageDisplayMarker`

### Phase 4: 播放控制

- 在 `systems.rs` 中实现 `on_enter()` 生命周期
- 在 `systems.rs` 中实现 `on_exit()` 生命周期
- 实现 `update_playback()` 播放更新系统
- 实现 `handle_play_pause()` 播放/暂停处理
- 实现 `handle_speed_change()` 速度切换处理
- 实现 `handle_page_change()` 翻页处理

### Phase 5: UI 生成

- 实现 `spawn_playback_control_ui()` 控制面板 UI
- 实现 `update_playback_position_display()` 更新位置显示

### Phase 6: 集成测试

- 注册 `PlaybackPlotPlugin` 到 `HomepagePlugin`
- 添加菜单栏入口（复用现有菜单架构）
- 端到端测试：读取 EDF+ -> 播放 -> 暂停 -> 翻页

## 9. 依赖清单

### edf_io crate

| 依赖 | 用途 |
|------|------|
| `edfplus` | EDF+ 文件读写 |
| `rand` | 测试数据生成 |

### ui crate

| 依赖 | 用途 |
|------|------|
| `edf_io` | EDF+ 文件读写（新增内部 crate） |
| 已有 `bevy` | 渲染和 ECS |
| 已有 `serde_json` | 配置序列化 |

**edfplus 库信息**：
- 版本: 0.1.0
- 许可证: BSD-3-Clause
- 特性: 完整的 EDF+ 读写支持、类型安全 API、流式读取

## 10. 与实时波形共享的组件

以下组件/系统可从 `realtime_plot` 复用：

- `generate_waveform_points()` 函数
- `get_channel_color()` 函数
- `CHANNEL_COLORS` 常量
- 波形渲染逻辑（`update_waveform_display`）
- 坐标轴和网格绘制（`spawn_axis_grid`）

**共享策略**：
1. 将通用渲染逻辑提取到 `common.rs` 或共享模块
2. 或在 `playback_plot` 中直接复用，不做抽象

**注意**：`PlaybackData` 与 `WaveformData` 结构不同，PlaybackData 的通道数从 EDF 文件头读取。
