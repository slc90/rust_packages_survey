use std::path::PathBuf;

/// 音频播放状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioPlaybackStatus {
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
	StatusChanged(AudioPlaybackStatus),
	Loaded {
		path: PathBuf,
		duration_ms: Option<u64>,
	},
	PositionUpdated {
		position_ms: u64,
		duration_ms: Option<u64>,
	},
	Error(String),
	Closed,
}
