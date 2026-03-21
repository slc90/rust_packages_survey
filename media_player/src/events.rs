use std::path::PathBuf;

use crate::frame::VideoFrame;

/// 播放状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackStatus {
	#[default]
	Idle,
	Loading,
	Playing,
	Paused,
	Ended,
	Error,
}

/// 播放线程命令
#[derive(Debug)]
pub enum PlayerCommand {
	Load(PathBuf),
	Play,
	Pause,
	Close,
	Shutdown,
}

/// 播放线程事件
#[derive(Debug, Clone)]
pub enum PlayerEvent {
	StatusChanged(PlaybackStatus),
	Loaded(PathBuf),
	FrameReady(VideoFrame),
	PositionUpdated {
		position_ms: u64,
		duration_ms: Option<u64>,
	},
	Error(String),
	Closed,
}
