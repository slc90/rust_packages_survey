use bevy::prelude::*;

/// 医学影像页面根节点
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct MedicalImageContentMarker;

/// 状态文本标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct MedicalImageStatusTextMarker;

/// 当前文件信息文本标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct MedicalImageSourceTextMarker;

/// 轴状视图图像标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct AxialSliceImageMarker;

/// 冠状视图图像标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct CoronalSliceImageMarker;

/// 矢状视图图像标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct SagittalSliceImageMarker;

/// 加载 CT 样例按钮标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct LoadCtSampleButtonMarker;

/// 加载 MR 样例按钮标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct LoadMrSampleButtonMarker;

/// 阈值减小按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct SurfaceThresholdDecreaseButtonMarker;

/// 阈值增大按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct SurfaceThresholdIncreaseButtonMarker;

/// 表面重建按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct RebuildSurfaceButtonMarker;

/// 切换到切片模式按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct SliceModeButtonMarker;

/// 切换到表面模式按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct SurfaceModeButtonMarker;

/// 切换到体渲染模式按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VolumeModeButtonMarker;

/// 体渲染步长减小按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VolumeStepDecreaseButtonMarker;

/// 体渲染步长增大按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VolumeStepIncreaseButtonMarker;

/// 窗位减小按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct WindowCenterDecreaseButtonMarker;

/// 窗位增大按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct WindowCenterIncreaseButtonMarker;

/// 窗宽减小按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct WindowWidthDecreaseButtonMarker;

/// 窗宽增大按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct WindowWidthIncreaseButtonMarker;

/// 三维预览区域标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct MedicalImageViewportMarker;

/// 三维相机标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct MedicalImageCamera3dMarker;

/// 三维灯光标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct MedicalImageLightMarker;

/// 表面网格实体标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct MedicalImageSurfaceMeshMarker;

/// 体渲染包围盒实体标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct MedicalImageVolumeBoxMarker;

/// 医学影像按钮 bundle
#[derive(Bundle)]
pub struct MedicalImageButtonBundle<T: Component> {
	/// 按钮组件
	pub button: Button,
	/// 按钮标记
	pub marker: T,
	/// 布局节点
	pub node: Node,
	/// 背景色
	pub background_color: BackgroundColor,
}

impl<T: Component> MedicalImageButtonBundle<T> {
	/// 创建标准按钮 bundle
	pub fn new(marker: T) -> Self {
		Self {
			button: Button,
			marker,
			node: Node {
				width: Val::Px(120.0),
				height: Val::Px(36.0),
				flex_shrink: 0.0,
				padding: UiRect::all(Val::Px(6.0)),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			background_color: BackgroundColor(Color::srgb(0.17, 0.44, 0.72)),
		}
	}
}

/// 医学影像切片面板 bundle
#[derive(Bundle)]
pub struct MedicalImagePanelBundle {
	/// 布局节点
	pub node: Node,
	/// 背景色
	pub background_color: BackgroundColor,
}

impl MedicalImagePanelBundle {
	/// 创建标准切片面板
	pub fn new(width: f32, height: f32) -> Self {
		Self {
			node: Node {
				width: Val::Px(width),
				height: Val::Px(height),
				min_width: Val::Px(width),
				min_height: Val::Px(height),
				flex_shrink: 0.0,
				flex_direction: FlexDirection::Column,
				padding: UiRect::all(Val::Px(8.0)),
				row_gap: Val::Px(8.0),
				..default()
			},
			background_color: BackgroundColor(Color::WHITE),
		}
	}

	/// 创建铺满父容器的响应式面板
	pub fn responsive(min_width: f32, height: f32) -> Self {
		Self {
			node: Node {
				width: Val::Percent(100.0),
				min_width: Val::Px(min_width),
				height: Val::Px(height),
				min_height: Val::Px(height),
				flex_grow: 1.0,
				flex_shrink: 1.0,
				flex_direction: FlexDirection::Column,
				padding: UiRect::all(Val::Px(8.0)),
				row_gap: Val::Px(8.0),
				..default()
			},
			background_color: BackgroundColor(Color::WHITE),
		}
	}
}

/// 切片图像 bundle
#[derive(Bundle)]
pub struct SliceImageBundle<T: Component> {
	/// 图像标记
	pub marker: T,
	/// 布局节点
	pub node: Node,
	/// 图像节点
	pub image_node: ImageNode,
}

impl<T: Component> SliceImageBundle<T> {
	/// 创建标准切片图像节点
	pub fn new(marker: T, texture: Handle<Image>, size: f32) -> Self {
		Self {
			marker,
			node: Node {
				width: Val::Px(size),
				height: Val::Px(size),
				flex_shrink: 0.0,
				align_self: AlignSelf::Center,
				..default()
			},
			image_node: ImageNode::new(texture),
		}
	}
}
