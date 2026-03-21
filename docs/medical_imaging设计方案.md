# 医学影像设计方案

## 1. 概述

医学影像功能用于加载和浏览三维医学数据，覆盖 `NIfTI`、`DICOM`、表面重建、体渲染、切片三视图五部分能力。整体方案继续沿用当前项目的 `Bevy 0.18 + ECS + Plugin` 架构，将“文件解析”“体数据处理”“渲染显示”“交互控制”拆开，避免把 I/O、算法和 UI 耦合在一起。

该功能的目标不是一次性做成完整工作站，而是优先完成一个可以稳定演进的最小闭环：

1. 读入单个 `NIfTI` 文件和单个 `DICOM Series`
2. 在界面中显示轴状 / 冠状 / 矢状三视图
3. 基于阈值生成表面网格并在 3D 视口显示
4. 提供基础体渲染能力

## 2. 推荐依赖

### 2.1 优先采用的现成库

| 依赖 | 用途 | 说明 |
|------|------|------|
| `nifti` | 读取 `NIfTI` / `nii.gz` | Rust 生态里较成熟的 `NIfTI` 库，支持 `ndarray` |
| `dicom-object` | 读取 `DICOM` 元数据与对象 | `dicom-rs` 生态高层 API |
| `dicom-pixeldata` | 解码 `DICOM Pixel Data` | 支持转成平面像素、`ndarray` 或 `image` |
| `ndarray` | 统一三维体数据表示 | 便于切片、重采样、阈值分割 |
| `image` | 2D 切片转纹理前的中间格式 | 便于灰度图、RGBA 图转换 |
| `fast_surface_nets` | 从体素提取等值面 | 适合表面重建，性能和实现复杂度都比自写 Marching Cubes 更稳 |

### 2.2 不建议一开始引入的方向

- 暂不建议优先找“Bevy 医学影像一体化插件”。目前更现实的做法是使用成熟的 Rust I/O 库读取数据，再在 Bevy 里自建显示和渲染流程。
- 暂不建议首版依赖 `VTK`、`ITK` 这类大型 C/C++ 绑定，集成成本、构建复杂度和跨平台风险都偏高。

### 2.3 方案里的库选型结论

- `NIfTI`：直接使用 `nifti`
- `DICOM`：使用 `dicom-object + dicom-pixeldata`
- 表面重建：优先使用 `fast_surface_nets`
- 体渲染：不依赖现成 Bevy 医学影像库，基于 `Bevy 0.18` 的自定义材质 / 自定义渲染通道实现 GPU Ray Marching
- 模态限制：首版只支持 `CT`、`MR`、`Segmentation`，不是这三类就直接报错

## 3. 架构设计

### 3.1 模块结构

建议新增一个内部 crate，专门负责医学影像数据读取与预处理：

```
medical_image/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── volume.rs
    ├── nifti_loader.rs
    ├── dicom_loader.rs
    ├── slice.rs
    ├── surface.rs
    └── windowing.rs

ui/src/homepage/medical_image/
├── components.rs
├── resources.rs
├── systems.rs
└── plugin.rs
```

对应主页模块导出：

```
ui/src/homepage/
├── medical_image.rs
└── common.rs
```

### 3.2 分层职责

| 层级 | 职责 |
|------|------|
| `medical_image` crate | 文件读取、体数据组织、切片和表面算法 |
| `ui::homepage::medical_image` | 页面生命周期、交互状态、纹理刷新、3D 视图控制 |
| Bevy 渲染层 | 切片纹理显示、3D 网格显示、体渲染 shader |

## 4. 核心数据结构

### 4.1 统一体数据结构

无论输入来自 `NIfTI` 还是 `DICOM`，都先统一为内部体数据格式。这里建议显式保留仿射矩阵，而不是只保留 `origin + spacing + direction`。

这样做的原因：

- `NIfTI` 原生就以 affine 表达 voxel 到世界坐标的变换
- `DICOM Series` 也可以从方向余弦、位置和 spacing 重建等价的 4x4 变换
- 三视图联动、表面重建、后续配准都需要稳定的物理空间坐标
- `origin + spacing + direction` 适合日常使用，`affine` 适合做统一坐标变换，两者都保留更稳

建议结构如下：

```rust
/// 统一的三维体数据结构
pub struct VolumeData {
    /// 体数据尺寸，顺序为 x、y、z
    pub dims: [usize; 3],
    /// 每个体素在三个方向上的物理间距，单位 mm
    pub spacing: [f32; 3],
    /// 体数据原点，对应世界坐标系中的起点
    pub origin: [f32; 3],
    /// 方向余弦矩阵，用于表示体数据朝向
    pub direction: [[f32; 3]; 3],
    /// 从 voxel 坐标到病人/世界坐标的 4x4 仿射矩阵
    pub affine: [[f32; 4]; 4],
    /// 连续存储的体素数据，统一转换为 f32
    pub voxels: Vec<f32>,
    /// 当前体数据的最小值和最大值
    pub value_range: [f32; 2],
    /// 医学影像模态，只保留当前明确支持的类型
    pub modality: VolumeModality,
}

pub enum VolumeModality {
    /// CT 体数据
    Ct,
    /// MR 体数据
    Mr,
    /// 分割结果体数据
    Segmentation,
}
```

这样做的好处：

- UI 和渲染层不再关心底层文件格式
- `NIfTI` 与 `DICOM` 共用切片、表面和体渲染逻辑
- 便于后续加入重采样、缓存和分块加载

### 4.2 UI 资源设计

```rust
#[derive(Resource)]
pub struct MedicalImageState {
    /// 当前加载的体数据；未加载时为 None
    pub volume: Option<VolumeData>,
    /// 当前 DICOM 序列的唯一标识；NIfTI 场景可为空
    pub current_series_uid: Option<String>,
    /// 三视图当前切片索引，顺序为轴状、冠状、矢状
    pub slice_index: [usize; 3],
    /// 窗位
    pub window_center: f32,
    /// 窗宽
    pub window_width: f32,
    /// 表面重建使用的阈值
    pub surface_threshold: f32,
    /// 当前显示模式
    pub render_mode: RenderMode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// 仅显示三视图切片
    SliceOnly,
    /// 显示表面重建三维结果
    Surface3d,
    /// 显示体渲染三维结果
    Volume3d,
}
```

## 5. 功能设计

### 5.1 读取 NIfTI

**实现方案**：使用 `nifti` crate 读取 `.nii` / `.nii.gz`，优先支持单体数据，不先做四维时序。

**功能**：

- 读取图像维度、spacing、仿射矩阵或方向信息
- 将原始数值统一转换为 `f32`
- 记录体数据最小值 / 最大值
- 为三视图和体渲染准备连续内存
- 若模态不是 `CT`、`MR`、`Segmentation`，直接返回错误

**首版边界**：

- 优先支持 3D 标量体
- 暂不支持 4D fMRI、DTI tensor、复杂多通道体

### 5.2 读取 DICOM

**实现方案**：使用 `dicom-object` 读取标签，使用 `dicom-pixeldata` 解码像素，再按 `SeriesInstanceUID` 组装三维序列。

**关键处理**：

- 从目录读取同一序列的所有切片
- 依据 `ImagePositionPatient` 优先排序
- 无法排序时退回 `InstanceNumber`
- 根据 `RescaleSlope / RescaleIntercept` 还原 CT HU 值
- 读取 `PixelSpacing` 与层厚 / 层间距
- 从 `ImageOrientationPatient`、`ImagePositionPatient`、spacing 重建 affine
- 若模态不是 `CT`、`MR`、`Segmentation`，直接返回错误

**首版边界**：

- 优先支持单帧灰度 CT / MR 序列
- 暂不支持增强对象、RTStruct、分割对象、超声多帧 cine

### 5.3 切片三视图

**实现方案**：基于统一 `VolumeData` 做三向重切片，生成三个 2D 灰度纹理并显示在 UI 中。

**功能**：

- 轴状 / 冠状 / 矢状三视图同步显示
- 鼠标滚轮或滑条切换层面
- 十字光标联动，点击一个视图时更新另外两个视图的交点
- 调整窗宽窗位

**渲染策略**：

- CPU 端根据当前切片索引提取 2D 数据
- 应用窗宽窗位后映射到 `u8`
- 上传为 Bevy `Image`

这种方式首版最稳，逻辑简单，便于先把交互做通。

### 5.4 表面重建

**实现方案**：对体数据做阈值化，调用 `fast_surface_nets` 提取等值面，再转换成 Bevy `Mesh`。

**处理步骤**：

1. 根据阈值生成标量场或二值体
2. 执行 Surface Nets 提取三角网格
3. 按 voxel spacing 把网格坐标映射到真实物理空间
4. 生成法线，创建 `Mesh3d`

**为什么优先 Surface Nets**：

- Rust 现成库可用
- 对规则体素数据适配好
- 比自行实现 Marching Cubes 更省时间
- 对首版“骨骼 / 皮层 / 器官粗表面”已经足够

### 5.5 体渲染

**实现方案**：基于 GPU Ray Marching 做体渲染，不建议首版走 CPU 切片堆叠。

**为什么不建议首版走 CPU 切片堆叠**：

- CPU 切片堆叠本质上是在 CPU 侧准备大量切片结果，再交给 GPU 混合；对体渲染这种高采样任务来说，CPU 很容易成为瓶颈
- 视角变化、窗宽窗位变化、裁剪平面变化时，往往需要频繁重建切片或重新上传大量纹理，CPU 到 GPU 的传输成本高
- CPU 路线更适合做“伪体渲染”或演示版，做 MIP、透明度累计、传递函数时，灵活性和画质都不如 shader 内直接采样 3D 纹理
- 后续如果要加剪切平面、步长调节、早停、空域跳跃，CPU 路线大概率需要推倒重来

**为什么虽然 GPU 编程更困难，仍然建议走 GPU**：

- 难点主要集中在一次 3D 纹理上传和 shader 管线接入，边界是清晰的
- 如果首版只做单通道 3D texture + MIP，shader 复杂度是可控的
- 一旦最小 ray marching 跑通，后续增强都能在同一条技术路径上演进
- 当前项目已经基于 `Bevy 0.18`，本身就站在 GPU 渲染框架上，长期成本反而更低

所以这里的取舍不是“GPU 更简单”，而是“GPU 首版更难一点，但路线正确；CPU 首版看起来轻，后面更容易返工”。

**渲染思路**：

1. 把体数据上传成 3D 纹理
2. 在包围盒立方体上执行 fragment shader
3. 射线步进采样 3D texture
4. 通过 transfer function 映射颜色与透明度
5. 支持 MIP 或 alpha compositing

**建议阶段**：

- 第一阶段：MIP，适合 CTA、血管、骨结构
- 第二阶段：Alpha Compositing + 1D transfer function
- 第三阶段：剪切平面、采样步长调节、空域跳跃优化

**结论**：

- 体渲染不建议依赖单独第三方 Bevy 医学库
- 更稳妥的路线是自己实现一个最小可控的 Bevy 自定义材质 / 渲染 pass

## 6. 系统设计

### 6.1 生命周期系统

| 系统 | 触发条件 | 功能 |
|------|----------|------|
| `on_enter` | 进入医学影像页面 | 初始化资源、默认状态、相机和视口 |
| `on_exit` | 离开医学影像页面 | 清理纹理、Mesh、临时缓存 |

### 6.2 数据加载系统

| 系统 | 功能 |
|------|------|
| `load_nifti_file` | 读取 `NIfTI` 并填充 `MedicalImageState.volume` |
| `load_dicom_series` | 扫描目录、组装 `DICOM Series` |
| `normalize_volume_metadata` | 把输入格式转换成统一坐标和 spacing |

### 6.3 交互系统

| 系统 | 功能 |
|------|------|
| `handle_slice_scroll` | 处理三视图切片滚动 |
| `handle_crosshair_pick` | 处理视图点击联动 |
| `handle_window_level_change` | 调整窗宽窗位 |
| `handle_render_mode_change` | 切换三视图 / 表面 / 体渲染 |
| `handle_surface_threshold_change` | 修改表面重建阈值 |

### 6.4 渲染系统

| 系统 | 功能 |
|------|------|
| `update_slice_textures` | 更新三张切片纹理 |
| `rebuild_surface_mesh` | 阈值变化后重建表面网格 |
| `upload_volume_texture` | 把体数据上传到 3D 纹理 |
| `update_volume_render_params` | 更新 shader 参数，如步长和窗宽窗位 |

## 7. UI 布局设计

```
+--------------------------------------------------------------------------------+
|  [主页] 医学影像                                                               |
+--------------------------------------------------------------------------------+
| [文件: xxx.nii.gz / Series_xxx] [加载NIfTI] [加载DICOM目录] [模式切换]          |
+--------------------------------------------------------------------------------+
|                                                                                |
|  +--------------------+ +--------------------+ +--------------------+          |
|  | 轴状视图            | | 冠状视图            | | 矢状视图            |          |
|  |                    | |                    | |                    |          |
|  |      slice         | |      slice         | |      slice         |          |
|  |                    | |                    | |                    |          |
|  +--------------------+ +--------------------+ +--------------------+          |
|                                                                                |
|  窗位: [40]   窗宽: [400]   阈值: [300]   当前层面: [120, 88, 76]             |
|                                                                                |
|  +--------------------------------------------------------------------------+  |
|  |                              3D 视口                                      |  |
|  |                     - 表面重建 / 体渲染结果                               |  |
|  +--------------------------------------------------------------------------+  |
|                                                                                |
+--------------------------------------------------------------------------------+
```

## 8. 实现步骤

### Phase 1: 创建 `medical_image` crate

- 1.1 创建 `medical_image/` 目录和 `Cargo.toml`
- 1.2 将 `medical_image` 加入 workspace 成员
- 1.3 在 `medical_image/Cargo.toml` 中添加依赖
  - `nifti`
  - `dicom-object`
  - `dicom-pixeldata`
  - `ndarray`
  - `image`
  - `fast_surface_nets`
- 1.4 创建 `medical_image/src/lib.rs`
- 1.5 创建 `medical_image/src/volume.rs`
- 1.6 在 `volume.rs` 中定义 `VolumeData`
- 1.7 在 `volume.rs` 中定义 `VolumeModality`
- 1.8 在 `volume.rs` 中定义医学影像统一错误类型
- 1.9 在 `lib.rs` 中导出 `VolumeData`、`VolumeModality` 和错误类型
- 1.10 创建 `medical_image/src/windowing.rs`
- 1.11 在 `windowing.rs` 中实现窗宽窗位映射函数
- 1.12 在 `windowing.rs` 中实现归一化到 `u8` 灰度图的工具函数
- 1.13 创建 `medical_image/src/slice.rs`
- 1.14 在 `slice.rs` 中定义轴状 / 冠状 / 矢状三种切片方向枚举
- 1.15 在 `slice.rs` 中实现从 `VolumeData` 提取 2D 切片的基础函数

### Phase 2: 三视图基础能力

- 2.1 创建 `medical_image/src/nifti_loader.rs`
- 2.2 在 `nifti_loader.rs` 中实现 `NIfTI` 文件读取入口
- 2.3 解析 `NIfTI` 维度、spacing、方向信息和 affine
- 2.4 将 `NIfTI` 原始数据统一转换为 `VolumeData`
- 2.5 检查模态是否属于 `CT`、`MR`、`Segmentation`
- 2.6 若模态不支持，返回明确错误
- 2.7 创建 `medical_image/src/dicom_loader.rs`
- 2.8 在 `dicom_loader.rs` 中实现扫描目录并收集 DICOM 文件
- 2.9 按 `SeriesInstanceUID` 对切片分组
- 2.10 选择目标序列并按 `ImagePositionPatient` 排序
- 2.11 排序信息缺失时退回 `InstanceNumber`
- 2.12 使用 `dicom-pixeldata` 解码像素数据
- 2.13 根据 `RescaleSlope / RescaleIntercept` 还原体素值
- 2.14 解析 `PixelSpacing`、层厚、方向余弦
- 2.15 重建 affine 并转换为统一 `VolumeData`
- 2.16 检查模态是否属于 `CT`、`MR`、`Segmentation`
- 2.17 若模态不支持，返回明确错误
- 2.18 在 `ui/src/homepage/common.rs` 中新增 `Functions::MedicalImage`
- 2.19 创建 `ui/src/homepage/medical_image/` 目录
- 2.20 创建 `ui/src/homepage/medical_image/components.rs`
- 2.21 创建 `ui/src/homepage/medical_image/resources.rs`
- 2.22 创建 `ui/src/homepage/medical_image/systems.rs`
- 2.23 创建 `ui/src/homepage/medical_image/plugin.rs`
- 2.24 在 `resources.rs` 中定义 `MedicalImageState`
- 2.25 在 `resources.rs` 中定义 `RenderMode`
- 2.26 在 `components.rs` 中定义三视图区域、控制面板、3D 视口等组件标记
- 2.27 在 `systems.rs` 中实现 `on_enter()` 生命周期初始化
- 2.28 在 `systems.rs` 中实现 `on_exit()` 生命周期清理
- 2.29 实现加载 `NIfTI` 文件的系统入口
- 2.30 实现加载 `DICOM` 目录的系统入口
- 2.31 实现三视图切片纹理初始化
- 2.32 实现轴状视图切片刷新
- 2.33 实现冠状视图切片刷新
- 2.34 实现矢状视图切片刷新
- 2.35 实现切片索引滑动或滚轮切换
- 2.36 实现窗宽窗位调整
- 2.37 实现三视图十字线联动
- 2.38 在 `plugin.rs` 中注册资源和系统
- 2.39 在 `ui/src/homepage/plugin.rs` 中注册 `MedicalImagePlugin`
- 2.40 在菜单栏中增加“医学影像”入口
- 2.41 完成 `NIfTI -> 三视图显示` 闭环测试
- 2.42 完成 `DICOM Series -> 三视图显示` 闭环测试

### Phase 3: 表面重建

- 3.1 创建 `medical_image/src/surface.rs`
- 3.2 在 `surface.rs` 中定义表面重建输入参数结构
- 3.3 在 `surface.rs` 中实现阈值转二值体或标量场的预处理
- 3.4 接入 `fast_surface_nets` 提取等值面
- 3.5 将输出顶点坐标映射到物理空间
- 3.6 生成法线数据
- 3.7 生成索引缓冲
- 3.8 封装为可转换到 Bevy `Mesh` 的中间结构
- 3.9 在 `resources.rs` 中增加表面重建阈值和缓存句柄
- 3.10 在 `systems.rs` 中实现“重建表面”触发逻辑
- 3.11 在 `systems.rs` 中实现 Bevy `Mesh` 创建与替换
- 3.12 在 3D 视口中显示表面网格
- 3.13 增加基础灯光
- 3.14 增加基础 3D 相机
- 3.15 增加简单轨道相机控制
- 3.16 实现表面模式与三视图模式切换
- 3.17 完成阈值调整 -> 网格刷新闭环测试

### Phase 4: 体渲染 MVP

- 4.1 明确 `VolumeData` 到 3D 纹理的数据格式
- 4.2 实现体数据归一化和上传缓冲准备
- 4.3 创建体渲染使用的 3D 纹理资源
- 4.4 在 Bevy 中创建包围盒几何体
- 4.5 创建体渲染材质参数结构
- 4.6 编写最小体渲染 shader 文件
- 4.7 在 shader 中实现射线进入 / 退出包围盒求交
- 4.8 在 shader 中实现 3D 纹理步进采样
- 4.9 先实现 `MIP`
- 4.10 在 UI 中增加体渲染模式切换
- 4.11 在 UI 中增加采样步长参数
- 4.12 在 UI 中增加窗口参数传递到 shader
- 4.13 打通 `VolumeData -> 3D 纹理 -> MIP 显示` 闭环
- 4.14 在 `MIP` 跑通后补 `alpha compositing`
- 4.15 在 `alpha compositing` 中接入基础 transfer function
- 4.16 实现三视图模式、表面模式、体渲染模式三者互斥切换
- 4.17 完成体渲染 MVP 的可视化验收

### Phase 5: 性能与工程化

- 5.1 将 `NIfTI` / `DICOM` 加载改为异步任务
- 5.2 增加加载中状态和错误提示状态
- 5.3 对切片纹理改为复用已有 `Image`，避免频繁创建
- 5.4 增加表面网格缓存，阈值未变时不重复生成
- 5.5 增加 DICOM 目录扫描结果缓存
- 5.6 对大体积数据增加下采样选项
- 5.7 评估是否保留原始 `i16` 缓存而不是始终转 `f32`
- 5.8 增加体渲染采样步长和最大步数限制
- 5.9 增加异常数据日志输出
- 5.10 增加最小测试数据集
  - `NIfTI` 示例
  - `DICOM Series` 示例
  - `Segmentation` 示例
- 5.11 增加单元测试
  - affine 重建测试
  - 切片提取测试
  - 窗宽窗位映射测试
  - 模态检查测试
- 5.12 增加集成测试
  - `NIfTI -> 三视图`
  - `DICOM -> 三视图`
  - `阈值变化 -> 表面刷新`
- 5.13 更新 `structure.md`
- 5.14 根据完成情况回写 `docs/requirements.md`

## 9. 风险与应对

### 9.1 DICOM 兼容性复杂

风险：

- 医院导出的 `DICOM` 差异大
- 压缩传输语法、层间距、方向信息不一定统一

应对：

- 首版只承诺支持单序列灰度体数据
- 先支持常见未压缩或纯 Rust 能解码的语法
- 对异常序列给出明确错误信息，不静默失败

### 9.2 Bevy 体渲染实现成本较高

风险：

- 需要理解 `Bevy 0.18` 渲染管线和 shader 集成
- 首次接入 3D 纹理和 ray marching 调试成本不低

应对：

- 三视图和表面重建先落地
- 体渲染单独作为第二阶段
- 首版只做单通道 3D texture + MIP，避免一开始做复杂传递函数
- 明确不走 CPU 切片堆叠，避免先做一套过渡方案再整体推翻

### 9.3 大体积数据占用内存

风险：

- CT / MR 体数据很容易达到数百 MB

应对：

- 统一转 `f32` 前先评估原始位深
- 视情况保留 `i16` 原始缓存 + 渲染时归一化
- 后续支持降采样和分块上传

## 10. 依赖清单

### `medical_image` crate

| 依赖 | 用途 |
|------|------|
| `nifti` | 读取 `.nii` / `.nii.gz` |
| `dicom-object` | 读取 DICOM 标签与对象 |
| `dicom-pixeldata` | 解码 DICOM 像素数据 |
| `ndarray` | 三维数组和切片处理 |
| `image` | 2D 图像中间表示 |
| `fast_surface_nets` | 表面重建 |

### `ui` crate

| 依赖 | 用途 |
|------|------|
| `medical_image` | 医学影像 I/O 与预处理 |
| 已有 `bevy` | UI、ECS、纹理、Mesh、渲染集成 |

## 11. 推荐落地顺序

建议按下面顺序推进，而不是五项并行：

1. `NIfTI` 读取
2. `DICOM Series` 读取
3. 三视图
4. 表面重建
5. 体渲染

这样可以尽快得到可见结果，并把最重的渲染风险留到后面单独收敛。
