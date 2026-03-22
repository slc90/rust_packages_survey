use bevy::prelude::*;
use image::DynamicImage;
use screenshot::request::CaptureRegion;
use std::path::PathBuf;

/// 截图测试页主状态。
#[derive(Resource, Debug, Clone)]
pub struct ScreenshotPageState {
	/// 页面状态文本。
	pub status_text: String,

	/// 整窗裁剪测试使用的固定区域。
	pub crop_region: CaptureRegion,

	/// 独立 Render 测试区的渲染目标。
	pub render_target: Option<Handle<Image>>,
}

impl Default for ScreenshotPageState {
	fn default() -> Self {
		Self {
			status_text: "等待截图操作".to_string(),
			crop_region: CaptureRegion {
				x: 96,
				y: 360,
				width: 420,
				height: 180,
			},
			render_target: None,
		}
	}
}

/// 桌面框选覆盖层状态。
#[derive(Resource, Debug)]
pub struct ScreenRegionOverlayState {
	/// 覆盖层窗口实体。
	pub window_entity: Entity,

	/// 覆盖层相机实体。
	pub camera_entity: Entity,

	/// 覆盖层根节点实体。
	pub root_entity: Entity,

	/// 选框节点实体。
	pub selection_entity: Entity,

	/// 抓到的桌面图像，用于最终裁剪。
	pub capture_image: DynamicImage,

	/// 输出路径。
	pub output_path: PathBuf,

	/// 拖拽起点。
	pub drag_start: Option<Vec2>,

	/// 拖拽当前点。
	pub drag_current: Option<Vec2>,
}

/// 页面状态文本消息。
#[derive(Message, Debug, Clone)]
pub struct ScreenshotStatusMessage(pub String);
