use std::path::PathBuf;

use thiserror::Error;

/// 媒体播放器统一错误
#[derive(Debug, Error)]
pub enum MediaPlayerError {
	#[error("GStreamer 初始化失败: {0}")]
	Initialization(String),
	#[error("文件不存在: {0}")]
	FileNotFound(PathBuf),
	#[error("构建播放管线失败: {0}")]
	Pipeline(String),
	#[error("播放器状态切换失败: {0}")]
	StateChange(String),
	#[error("IO 错误: {0}")]
	Io(#[from] std::io::Error),
}
