use std::path::PathBuf;

use crate::events::AudioPlaybackStatus;

/// 当前播放快照
#[derive(Debug, Clone, Default)]
pub struct PlaybackSnapshot {
	pub current_file: Option<PathBuf>,
	pub status: AudioPlaybackStatus,
	pub position_ms: u64,
	pub duration_ms: Option<u64>,
	pub status_text: String,
}
