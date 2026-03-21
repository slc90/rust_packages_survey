//! EDF+ 文件读取器
//!
//! 使用 edfplus 库读取 EDF+ 格式文件

use edfplus::EdfReader;
use thiserror::Error;

/// EDF+ 加载错误
#[derive(Error, Debug)]
pub enum EdfLoaderError {
	#[error("文件打开失败: {0}")]
	FileOpenError(String),

	#[error("文件读取失败: {0}")]
	ReadError(String),

	#[error("无效的通道索引: {0}")]
	InvalidChannel(usize),
}

/// EDF+ 文件读取器
///
/// 封装 edfplus 库的 EdfReader，提供更简洁的 API
pub struct EdfLoader {
	/// 文件路径
	path: String,
	/// 通道数量
	channel_count: usize,
	/// 采样率
	sample_rate: u32,
	/// 总数据点数
	total_points: usize,
	/// 各通道物理值数据
	channels: Vec<Vec<f32>>,
}

impl EdfLoader {
	/// 从文件加载 EDF+ 数据
	///
	/// # Arguments
	/// * `path` - EDF+ 文件路径
	///
	/// # Returns
	/// 成功返回 EdfLoader 实例
	pub fn from_file(path: &str) -> Result<Self, EdfLoaderError> {
		let mut reader =
			EdfReader::open(path).map_err(|e| EdfLoaderError::FileOpenError(e.to_string()))?;
		let header = reader.header();
		let channel_count = header.signals.len();
		let sample_rate = header
			.signals
			.first()
			.map(|signal| signal.samples_per_record.max(0) as u32)
			.unwrap_or(0);

		let mut channels: Vec<Vec<f32>> = Vec::with_capacity(channel_count);
		for signal_idx in 0..channel_count {
			let samples = reader
				.read_physical_samples(signal_idx, usize::MAX)
				.map_err(|e| EdfLoaderError::ReadError(e.to_string()))?;
			channels.push(samples.into_iter().map(|value| value as f32).collect());
		}

		let total_points = channels.first().map(|channel| channel.len()).unwrap_or(0);

		Ok(Self {
			path: path.to_string(),
			channel_count,
			sample_rate,
			total_points,
			channels,
		})
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

	/// 获取文件路径
	pub fn path(&self) -> &str {
		&self.path
	}

	/// 获取全部通道数据
	pub fn channels(&self) -> &[Vec<f32>] {
		&self.channels
	}
}
