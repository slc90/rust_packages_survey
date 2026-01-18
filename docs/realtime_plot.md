# 实时波形绘制方案

本文档介绍如何使用 Bevy 引擎中的 `Polyline2d`、`Mesh2D` 和 `MeshMaterial2d` 实现实时波形绘制功能。

## 基本用法

以下是一个使用 `Polyline2d`、`Mesh2D` 和 `MeshMaterial2d` 绘制简单折线的基本示例：

```rust
use bevy::prelude::*;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // 创建 2D 相机
    commands.spawn(Camera2d);
    
    // 创建折线顶点数据
    let points = vec![
        Vec2::new(-100.0, 0.0),
        Vec2::new(-50.0, 50.0),
        Vec2::new(0.0, -30.0),
        Vec2::new(50.0, 80.0),
        Vec2::new(100.0, 0.0),
    ];
    
    // 创建 Polyline2d 并将其转换为 Mesh
    let polyline = Polyline2d::new(points);
    let mesh_handle = meshes.add(polyline);
    
    // 创建颜色材质
    let color = Color::rgb(0.2, 0.6, 0.9);
    let material_handle = materials.add(color);
    
    // 创建折线实体
    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}
```

### 关键步骤说明

1. **导入必要类型**：需要导入 `bevy::prelude::*` 以使用核心类型
2. **创建顶点数据**：`Vec2` 数组定义折线的各个点
3. **创建 Polyline2d**：使用 `Polyline2d::new()` 创建折线图元
4. **添加到 Mesh 资产**：通过 `meshes.add()` 将图元转换为 Mesh 并获取句柄
5. **创建颜色材质**：使用 `Color` 创建材质并添加到材质资产
6. **创建实体**：组合 `Mesh2d`、`MeshMaterial2d` 和 `Transform` 组件创建可渲染实体

## 方案概述

实时波形绘制需要动态更新折线图以显示随时间变化的数据。本方案基于 Bevy 的 2D 渲染系统，使用 `Polyline2d` 图元创建折线网格，并通过 `Mesh2d` 和 `MeshMaterial2d` 组件进行渲染。数据更新通过修改 `Assets<Mesh>` 中存储的网格数据实现。

## 核心组件

1. **Polyline2d**：2D 多段线图元，可转换为 `Mesh`
2. **Mesh2d**：组件，包装 `Handle<Mesh>`，用于渲染 2D 网格
3. **MeshMaterial2d**：组件，包装 `Handle<ColorMaterial>`，用于设置网格材质
4. **Assets<Mesh>**：资源，存储和管理网格数据

## 系统设计

### 数据结构

```rust
/// 波形数据管理器
#[derive(Resource)]
struct WaveformData {
    /// 当前波形数据点 (归一化到 [0, 1] 范围)
    points: Vec<f32>,
    /// 数据点数量
    capacity: usize,
    /// 当前数据索引
    index: usize,
}

impl WaveformData {
    /// 添加新的数据点
    fn push(&mut self, value: f32) {
        self.points[self.index] = value;
        self.index = (self.index + 1) % self.capacity;
    }
}

/// 波形实体组件
#[derive(Component)]
struct Waveform {
    /// 网格句柄
    mesh_handle: Handle<Mesh>,
    /// 材质句柄
    material_handle: Handle<ColorMaterial>,
    /// 波形显示区域大小
    size: Vec2,
    /// 波形颜色
    color: Color,
}
```

### 初始化系统

初始化系统负责创建波形实体，包括：
1. 创建 `Polyline2d` 网格并添加到 `Assets<Mesh>`
2. 创建颜色材质并添加到 `Assets<ColorMaterial>`
3. 创建包含 `Mesh2d` 和 `MeshMaterial2d` 组件的实体

```rust
fn setup_waveform(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // 创建初始波形数据（水平线）
    let initial_points = vec![Vec2::ZERO; WAVEFORM_POINTS];
    let polyline = Polyline2d::new(initial_points);
    
    // 创建网格和材质
    let mesh_handle = meshes.add(polyline);
    let material_handle = materials.add(Color::BLUE);
    
    // 创建波形实体
    commands.spawn((
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(material_handle.clone()),
        Waveform {
            mesh_handle,
            material_handle,
            size: Vec2::new(800.0, 200.0),
            color: Color::BLUE,
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    // 初始化波形数据资源
    commands.insert_resource(WaveformData {
        points: vec![0.0; WAVEFORM_POINTS],
        capacity: WAVEFORM_POINTS,
        index: 0,
    });
}
```

### 数据更新系统

数据更新系统负责接收新的波形数据并更新数据缓冲区：

```rust
fn update_waveform_data(
    mut waveform_data: ResMut<WaveformData>,
    // 假设从外部数据源获取新数据点
    new_data: Res<WaveformDataSource>,
) {
    for &value in &new_data.values {
        waveform_data.points[waveform_data.index] = value;
        waveform_data.index = (waveform_data.index + 1) % waveform_data.capacity;
    }
}
```

### 网格更新系统

网格更新系统根据最新的波形数据更新 `Polyline2d` 网格：

```rust
fn update_waveform_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    waveform_data: Res<WaveformData>,
    mut query: Query<(&Waveform, &Mesh2d)>,
) {
    if waveform_data.is_changed() {
        for (waveform, mesh2d) in query.iter_mut() {
            if let Some(mesh) = meshes.get_mut(&waveform.mesh_handle) {
                // 创建新的顶点数据
                let mut points = Vec::with_capacity(waveform_data.capacity);
                let x_step = waveform.size.x / (waveform_data.capacity as f32 - 1.0);
                
                for (i, &value) in waveform_data.points.iter().enumerate() {
                    let x = i as f32 * x_step - waveform.size.x / 2.0;
                    let y = (value * 2.0 - 1.0) * waveform.size.y / 2.0;
                    points.push(Vec2::new(x, y));
                }
                
                // 创建新的 Polyline2d 并替换原网格
                let new_polyline = Polyline2d::new(points);
                *mesh = Mesh::from(new_polyline);
                
                // 更新 Mesh2d 组件（如果需要重新创建句柄）
                // 注意：直接修改 Assets 中的网格数据即可，不需要更新句柄
            }
        }
    }
}
```

### 性能优化方案

#### 1. 环形缓冲区
使用环形缓冲区存储波形数据，避免频繁的内存分配：

```rust
struct CircularBuffer {
    data: Vec<f32>,
    capacity: usize,
    head: usize,
    tail: usize,
    size: usize,
}

impl CircularBuffer {
    fn push(&mut self, value: f32) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % self.capacity;
        if self.size < self.capacity {
            self.size += 1;
        } else {
            self.tail = (self.tail + 1) % self.capacity;
        }
    }
    
    fn iter(&self) -> impl Iterator<Item = &f32> {
        (0..self.size).map(move |i| {
            &self.data[(self.tail + i) % self.capacity]
        })
    }
}
```

#### 2. 增量更新
只更新变化的部分网格数据，而非整个网格：

```rust
fn partial_mesh_update(
    mesh: &mut Mesh,
    new_points: &[Vec2],
    start_index: usize,
) {
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for (i, point) in new_points.iter().enumerate() {
            let idx = start_index + i;
            if idx < positions.len() {
                positions[idx] = [point.x, point.y, 0.0];
            }
        }
    }
}
```

#### 3. 批处理更新
积累一定数量的数据点后批量更新网格，减少更新频率：

```rust
#[derive(Resource)]
struct WaveformUpdateAccumulator {
    pending_points: Vec<f32>,
    update_threshold: usize,
}

fn accumulate_and_update(
    mut accumulator: ResMut<WaveformUpdateAccumulator>,
    new_data: Res<WaveformDataSource>,
    mut waveform_data: ResMut<WaveformData>,
) {
    accumulator.pending_points.extend(&new_data.values);
    
    if accumulator.pending_points.len() >= accumulator.update_threshold {
        for &value in &accumulator.pending_points {
            waveform_data.push(value);
        }
        accumulator.pending_points.clear();
    }
}
```

### 高级功能

#### 1. 多通道波形
支持同时显示多个波形通道：

```rust
struct MultiChannelWaveform {
    channels: Vec<WaveformChannel>,
}

struct WaveformChannel {
    mesh_handle: Handle<Mesh>,
    material_handle: Handle<ColorMaterial>,
    data: Vec<f32>,
    color: Color,
    offset: f32, // Y轴偏移
    scale: f32,  // 振幅缩放
}
```

#### 2. 可配置样式
支持动态调整波形样式：

```rust
#[derive(Component)]
struct WaveformStyle {
    line_width: f32,
    line_color: Color,
    fill_color: Option<Color>,
    antialias: bool,
}

fn update_waveform_style(
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(&Waveform, &WaveformStyle), Changed<WaveformStyle>>,
) {
    for (waveform, style) in query.iter() {
        if let Some(material) = materials.get_mut(&waveform.material_handle) {
            material.color = style.line_color;
        }
    }
}
```

#### 3. 交互功能
支持波形缩放和平移：

```rust
fn handle_waveform_interaction(
    mut query: Query<(&mut Waveform, &mut Transform)>,
    mouse_wheel: Res<ButtonInput<MouseWheel>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for (mut waveform, mut transform) in query.iter_mut() {
        // 鼠标滚轮缩放
        if mouse_wheel.just_pressed(MouseWheel::Up) {
            waveform.size *= 1.1;
        } else if mouse_wheel.just_pressed(MouseWheel::Down) {
            waveform.size *= 0.9;
        }
        
        // 键盘平移
        if keyboard.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= 10.0;
        } else if keyboard.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 10.0;
        }
    }
}
```

## 完整示例

```rust
use bevy::prelude::*;

const WAVEFORM_POINTS: usize = 1024;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, setup_waveform))
        .add_systems(
            Update,
            (
                update_waveform_data,
                update_waveform_mesh,
                simulate_waveform_data,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn simulate_waveform_data(time: Res<Time>, mut data_source: ResMut<WaveformDataSource>) {
    // 生成模拟波形数据（正弦波）
    let t = time.elapsed_secs();
    data_source.values.clear();
    
    for i in 0..10 {
        let value = (t * 2.0 + i as f32 * 0.1).sin() * 0.5 + 0.5;
        data_source.values.push(value);
    }
}

#[derive(Resource)]
struct WaveformDataSource {
    values: Vec<f32>,
}

impl Default for WaveformDataSource {
    fn default() -> Self {
        Self {
            values: Vec::new(),
        }
    }
}
```

## 注意事项

1. **线程安全**：确保波形数据的更新和渲染在不同系统间同步
2. **内存管理**：避免频繁创建新的网格句柄，重用现有资源
3. **渲染顺序**：合理设置 `Transform` 的 z 坐标以确保正确的渲染顺序
4. **性能监控**：在性能敏感的应用中监控网格更新频率和内存使用
5. **WebAssembly 支持**：注意 WebAssembly 环境下某些图形功能可能受限

## 扩展建议

1. **使用计算着色器**：对于超大规模波形数据，考虑使用计算着色器进行数据处理
2. **GPU 加速**：利用 GPU 实例化技术同时渲染多个波形
3. **LOD 系统**：根据缩放级别动态调整波形细节层次
4. **离线渲染**：支持将波形导出为图片或视频

## 官方示例参考

以下是 helpers.md 中提到的官方示例和文档，直接关联到实时波形绘制：

### 1. Bevy 2D 形状示例
- **URL**: https://bevy.org/examples/2d-rendering/2d-shapes/
- **内容**: 展示了如何使用各种 2D 图元（包括 `Polyline2d`）创建网格
- **关键特性**:
  - 使用 `Polyline2d::new()` 创建多段线
  - 通过 `meshes.add()` 将图元转换为 `Mesh`
  - 使用 `Mesh2d` 和 `MeshMaterial2d` 组件渲染
  - 支持实时更新网格数据

### 2. Mesh 结构体文档
- **URL**: https://docs.rs/bevy/latest/bevy/prelude/struct.Mesh.html
- **内容**: `Mesh` 结构体的完整 API 文档
- **关键功能**:
  - 手动创建和修改网格数据
  - 使用 `with_inserted_attribute()` 添加顶点属性
  - 动态更新顶点位置数据
  - 与 `Polyline2d` 等图元的转换接口

### 使用建议
1. **实时更新**: 通过 `meshes.get_mut()` 获取网格的可变引用，直接修改顶点数据
2. **性能优化**: 避免每帧创建新网格，重用现有网格资源
3. **数据映射**: 将波形数据归一化到合适的坐标空间（如 [-1, 1] 或基于显示区域大小）
4. **渲染配置**: 通过 `MeshMaterial2d` 设置线条颜色和样式

## 参考资源

1. [Bevy 2D 形状示例](https://bevy.org/examples/2d-rendering/2d-shapes/)
2. [Mesh 结构体文档](https://docs.rs/bevy/latest/bevy/prelude/struct.Mesh.html)
3. [Bevy 资产系统](https://bevy-cheatbook.github.io/features/assets.html)
4. [Bevy ECS 系统](https://bevy-cheatbook.github.io/programming/systems.html)

---
*最后更新：根据 Bevy 0.18 版本编写*
