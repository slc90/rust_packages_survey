use std::path::PathBuf;

use bevy::prelude::*;
use media_player::{PlaybackStatus, PlayerHandle};

/// 单个播放器槽位状态
#[derive(Default)]
pub struct VideoPlayerSlotState {
	pub player: Option<PlayerHandle>,
	pub texture: Option<Handle<Image>>,
	pub current_file: Option<PathBuf>,
	pub status: PlaybackStatus,
	pub status_text: String,
	pub position_ms: u64,
	pub duration_ms: Option<u64>,
}

/// 主窗口视频页面状态
#[derive(Resource, Default)]
pub struct MainVideoPlayerState {
	pub slot: VideoPlayerSlotState,
	pub initial_directory: PathBuf,
}

/// 弹窗播放器状态
#[derive(Resource, Default)]
pub struct PopupVideoPlayerState {
	pub window_entity: Option<Entity>,
	pub camera_entity: Option<Entity>,
	pub root_entity: Option<Entity>,
	pub slot: VideoPlayerSlotState,
}
