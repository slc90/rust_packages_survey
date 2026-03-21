use std::path::PathBuf;

use audio_player::{AudioPlaybackStatus, PlayerHandle};
use bevy::prelude::*;

/// 页面级音频播放状态
#[derive(Resource)]
pub struct AudioPlaybackState {
	pub player: Option<PlayerHandle>,
	pub current_file: Option<PathBuf>,
	pub status: AudioPlaybackStatus,
	pub position_ms: u64,
	pub duration_ms: Option<u64>,
	pub status_text: String,
	pub initial_directory: PathBuf,
}

impl Default for AudioPlaybackState {
	fn default() -> Self {
		Self {
			player: None,
			current_file: None,
			status: AudioPlaybackStatus::Idle,
			position_ms: 0,
			duration_ms: None,
			status_text: "请点击打开文件选择音频".to_string(),
			initial_directory: PathBuf::new(),
		}
	}
}
