use std::path::PathBuf;

use bevy::prelude::*;

/// 范式页面中的 GIF 帧数据。
pub struct ParadigmGifFrame {
	/// 当前帧纹理句柄。
	pub image: Handle<Image>,

	/// 当前帧持续时长，单位秒。
	pub duration_seconds: f32,
}

/// 可选的显示器信息。
#[derive(Clone)]
pub struct ParadigmMonitorOption {
	/// Bevy 里的显示器实体。
	pub entity: Entity,

	/// 显示器名称。
	pub name: String,

	/// 显示器刷新率，单位 Hz。
	pub refresh_rate_hz: f64,
}

/// GIF 预览状态。
#[derive(Default)]
pub struct ParadigmGifPreviewState {
	/// GIF 文件路径。
	pub path: PathBuf,

	/// 文件名称。
	pub file_name: String,

	/// 预览帧序列。
	pub frames: Vec<ParadigmGifFrame>,

	/// 当前帧索引。
	pub current_frame_index: usize,

	/// 当前帧已累计时间，单位秒。
	pub accumulated_seconds: f32,

	/// 是否已经成功加载。
	pub is_loaded: bool,

	/// 当前状态说明。
	pub status_text: String,

	/// GIF 尺寸。
	pub size: UVec2,
}

/// 范式页面状态。
#[derive(Resource, Default)]
pub struct ParadigmPageState {
	/// GIF 预览状态。
	pub gif_preview: ParadigmGifPreviewState,

	/// 当前可用显示器列表。
	pub monitor_options: Vec<ParadigmMonitorOption>,

	/// 当前选中的显示器索引。
	pub selected_monitor_index: usize,

	/// 当前目标字符。
	pub target_symbol: char,

	/// 页面状态文本。
	pub status_text: String,
}

/// 当前高亮目标。
#[derive(Clone, Copy, Default)]
pub enum ParadigmStimulusTarget {
	/// 当前不高亮。
	#[default]
	None,

	/// 高亮某一行。
	Row(usize),

	/// 高亮某一列。
	Column(usize),
}

/// P300 播放状态。
#[derive(Resource, Default)]
pub struct ParadigmPlaybackState {
	/// 当前是否处于播放中。
	pub is_running: bool,

	/// 当前是否处于暂停态。
	pub is_paused: bool,

	/// 当前刺激序列。
	pub sequence: Vec<ParadigmStimulusTarget>,

	/// 当前刺激索引。
	pub sequence_index: usize,

	/// 当前刺激剩余帧数。
	pub remaining_frames: u32,

	/// flash 阶段持续帧数。
	pub flash_frames: u32,

	/// interval 阶段持续帧数。
	pub interval_frames: u32,

	/// 当前是否处于 flash 阶段。
	pub is_flash_phase: bool,

	/// 当前高亮目标。
	pub active_target: ParadigmStimulusTarget,

	/// 当前 block 数量。
	pub block_count: u32,
}

/// 范式播放窗口状态。
#[derive(Resource, Default)]
pub struct ParadigmPresentationWindowState {
	/// 播放窗口实体。
	pub window_entity: Option<Entity>,

	/// 播放相机实体。
	pub camera_entity: Option<Entity>,

	/// 播放根节点实体。
	pub root_entity: Option<Entity>,

	/// 当前目标显示器索引。
	pub target_monitor_index: usize,

	/// 当前窗口是否已经创建。
	pub is_open: bool,
}
