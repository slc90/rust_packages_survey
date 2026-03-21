use bevy::prelude::*;
use medical_image::{SurfaceMeshStats, VolumeData, VolumeModality};

/// 医学影像显示模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
	/// 仅显示三视图切片
	SliceOnly,
	/// 显示表面重建三维结果
	Surface3d,
	/// 显示体渲染三维结果
	Volume3d,
}

/// 医学影像页面数据状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MedicalImageLoadState {
	/// 尚未加载数据
	Empty,
	/// 当前有可用数据
	Ready,
	/// 当前处于处理阶段
	Busy,
	/// 上一次处理失败
	Error,
}

/// 医学影像页面状态
#[derive(Resource, Debug)]
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
	/// 当前表面网格统计信息
	pub surface_mesh_stats: Option<SurfaceMeshStats>,
	/// 是否请求重建表面
	pub surface_dirty: bool,
	/// 三维预览聚焦中心
	pub surface_focus_center: [f32; 3],
	/// 三维预览相机距离
	pub surface_camera_distance: f32,
	/// 三维预览相机偏航角
	pub surface_camera_yaw: f32,
	/// 三维预览相机俯仰角
	pub surface_camera_pitch: f32,
	/// 体渲染步长，按包围盒最大边长的比例表示
	pub volume_step_size: f32,
	/// 是否请求重建体渲染资源
	pub volume_dirty: bool,
	/// 当前显示模式
	pub render_mode: RenderMode,
	/// 当前状态文本
	pub status_text: String,
	/// 当前数据源描述
	pub source_text: String,
	/// 当前模态
	pub modality: Option<VolumeModality>,
	/// 当前页面数据状态
	pub load_state: MedicalImageLoadState,
	/// 当前数据版本号；每次加载新体数据后递增
	pub volume_revision: u64,
}

impl Default for MedicalImageState {
	fn default() -> Self {
		Self {
			volume: None,
			current_series_uid: None,
			slice_index: [0, 0, 0],
			window_center: 40.0,
			window_width: 400.0,
			surface_threshold: 300.0,
			surface_mesh_stats: None,
			surface_dirty: false,
			surface_focus_center: [0.0, 0.0, 0.0],
			surface_camera_distance: 400.0,
			surface_camera_yaw: 0.75,
			surface_camera_pitch: 0.45,
			volume_step_size: 1.0 / 256.0,
			volume_dirty: false,
			render_mode: RenderMode::SliceOnly,
			status_text: "尚未加载医学影像数据".to_string(),
			source_text: "文件: -".to_string(),
			modality: None,
			load_state: MedicalImageLoadState::Empty,
			volume_revision: 0,
		}
	}
}

impl MedicalImageState {
	/// 根据体数据尺寸重置切片索引
	pub fn reset_slice_index(&mut self) {
		if let Some(volume) = &self.volume {
			self.slice_index = [volume.dims[2] / 2, volume.dims[1] / 2, volume.dims[0] / 2];
		}
	}

	/// 根据模态设置默认窗宽窗位
	pub fn apply_default_windowing(&mut self) {
		match self.modality {
			Some(VolumeModality::Ct) => {
				self.window_center = 40.0;
				self.window_width = 400.0;
				self.surface_threshold = 300.0;
			}
			Some(VolumeModality::Mr) => {
				if let Some(volume) = &self.volume {
					let [min_value, max_value] = volume.value_range;
					self.window_center = (min_value + max_value) / 2.0;
					self.window_width = (max_value - min_value).max(1.0);
					self.surface_threshold = self.window_center;
				}
			}
			Some(VolumeModality::Segmentation) => {
				self.window_center = 0.5;
				self.window_width = 1.0;
				self.surface_threshold = 0.5;
			}
			None => {}
		}
	}
}

/// 三视图纹理资源
#[derive(Resource, Debug)]
pub struct MedicalImageTextures {
	/// 轴状视图纹理
	pub axial: Handle<Image>,
	/// 冠状视图纹理
	pub coronal: Handle<Image>,
	/// 矢状视图纹理
	pub sagittal: Handle<Image>,
}

/// 医学影像三维场景资源
#[derive(Resource, Debug)]
pub struct MedicalImageSceneResources {
	/// 表面网格材质
	pub surface_material: Handle<StandardMaterial>,
	/// 最近一次缓存的表面网格
	pub cached_surface_mesh: Option<Handle<Mesh>>,
	/// 最近一次缓存表面网格的阈值
	pub cached_surface_threshold: Option<f32>,
	/// 最近一次缓存表面网格对应的数据版本
	pub cached_surface_revision: Option<u64>,
	/// 最近一次缓存的体纹理
	pub cached_volume_texture: Option<Handle<Image>>,
	/// 最近一次缓存的体渲染材质
	pub cached_volume_material:
		Option<Handle<crate::homepage::medical_image::volume_render::VolumeRenderMaterial>>,
	/// 最近一次缓存体纹理对应的数据版本
	pub cached_volume_revision: Option<u64>,
}
