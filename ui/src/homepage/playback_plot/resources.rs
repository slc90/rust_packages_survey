//! 回放波形资源定义
//!
//! 定义回放波形的数据结构、播放状态和配置

use bevy::prelude::*;

// ============================================================================
// PLAYBACK DATA - 回放数据资源
// ============================================================================

/// 回放数据资源
///
/// 存储从 EDF 文件加载的波形数据
#[derive(Resource, Debug, Default)]
pub struct PlaybackData {
	/// 源文件路径
	pub file_path: String,
	/// 各通道数据
	pub channels: Vec<Vec<f32>>,
	/// 通道数量（从 EDF 文件头读取）
	pub channel_count: usize,
	/// 采样率（从 EDF 文件头读取）
	pub sample_rate: u32,
	/// 总数据点数
	pub total_points: usize,
}

impl PlaybackData {
	/// 创建新的 PlaybackData
	pub fn new(
		file_path: String,
		channel_count: usize,
		sample_rate: u32,
		total_points: usize,
	) -> Self {
		Self {
			file_path,
			channels: vec![Vec::new(); channel_count],
			channel_count,
			sample_rate,
			total_points,
		}
	}

	/// 获取通道数量
	pub fn channel_count(&self) -> usize {
		self.channel_count
	}

	/// 获取采样率
	pub fn sample_rate(&self) -> u32 {
		self.sample_rate
	}

	/// 获取总数据点数
	pub fn total_points(&self) -> usize {
		self.total_points
	}
}

// ============================================================================
// PLAYBACK STATUS - 播放状态枚举
// ============================================================================

/// 回放播放状态
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PlaybackStatus {
	/// 播放中
	Playing,
	/// 暂停
	#[default]
	Paused,
}

// ============================================================================
// PLAYBACK CONTROL - 回放控制状态
// ============================================================================

/// 回放控制状态
#[derive(Resource, Debug)]
pub struct PlaybackControl {
	/// 播放状态
	pub status: PlaybackStatus,
	/// 当前播放位置（采样点索引）
	pub position: usize,
	/// 播放速度倍率
	pub speed: f32,
	/// 当前页码
	pub current_page: usize,
	/// 总页数
	pub total_pages: usize,
	/// 每页数据点数
	pub page_size: usize,
}

impl Default for PlaybackControl {
	fn default() -> Self {
		Self {
			status: PlaybackStatus::Paused,
			position: 0,
			speed: 1.0,
			current_page: 0,
			total_pages: 0,
			page_size: 4096,
		}
	}
}

impl PlaybackControl {
	/// 创建新的 PlaybackControl
	pub fn new(page_size: usize) -> Self {
		Self {
			status: PlaybackStatus::Paused,
			position: 0,
			speed: 1.0,
			current_page: 0,
			total_pages: 0,
			page_size,
		}
	}

	/// 切换播放/暂停状态
	pub fn toggle(&mut self) {
		self.status = match self.status {
			PlaybackStatus::Playing => PlaybackStatus::Paused,
			PlaybackStatus::Paused => PlaybackStatus::Playing,
		};
	}

	/// 是否正在播放
	pub fn is_playing(&self) -> bool {
		self.status == PlaybackStatus::Playing
	}
}

// ============================================================================
// PLAYBACK SPEED - 播放速度
// ============================================================================

/// 播放速度选项
pub const PLAYBACK_SPEED_OPTIONS: [f32; 3] = [1.0, 2.0, 4.0];

/// 播放速度
#[derive(Resource, Debug)]
pub struct PlaybackSpeed {
	pub multiplier: f32,
}

impl Default for PlaybackSpeed {
	fn default() -> Self {
		Self { multiplier: 1.0 }
	}
}

impl PlaybackSpeed {
	/// 切换到下一个速度档位
	pub fn next_speed(&mut self) {
		let current_idx = PLAYBACK_SPEED_OPTIONS
			.iter()
			.position(|&s| s == self.multiplier)
			.unwrap_or(0);
		let next_idx = (current_idx + 1) % PLAYBACK_SPEED_OPTIONS.len();
		self.multiplier = PLAYBACK_SPEED_OPTIONS[next_idx];
	}
}
