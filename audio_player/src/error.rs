use std::path::PathBuf;

use thiserror::Error;

/// 音频播放统一错误
#[derive(Debug, Error)]
pub enum AudioPlayerError {
	#[error("音频输出初始化失败: {0}")]
	Initialization(String),
	#[error("文件不存在: {0}")]
	FileNotFound(PathBuf),
	#[error("文件打开失败: {0}")]
	FileOpen(String),
	#[error("音频解码失败: {0}")]
	Decode(String),
	#[error("播放命令发送失败: {0}")]
	Command(String),
	#[error("IO 错误: {0}")]
	Io(#[from] std::io::Error),
}
