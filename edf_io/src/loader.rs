//! EDF+ 文件读取器
//!
//! 使用 edfplus 库读取 EDF+ 格式文件

use edfplus::EdfReader;
use std::fs;
use thiserror::Error;

/// EDF+ 加载错误
#[derive(Error, Debug)]
pub enum EdfLoaderError {
	#[error("文件打开失败: {0}")]
	FileOpenError(String),

	#[error("文件读取失败: {0}")]
	ReadError(String),

	#[error("无效的文件格式: {0}")]
	InvalidFormat(String),

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
		if is_bdf_file(path)? {
			return Self::from_bdf_file(path);
		}

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

	/// 从 BDF 文件加载数据
	fn from_bdf_file(path: &str) -> Result<Self, EdfLoaderError> {
		let bytes =
			fs::read(path).map_err(|error| EdfLoaderError::FileOpenError(error.to_string()))?;
		if bytes.len() < 256 {
			return Err(EdfLoaderError::InvalidFormat("BDF 头长度不足".to_string()));
		}

		let header_bytes = parse_ascii_usize(&bytes[184..192], "头长度")?;
		let channel_count = parse_ascii_usize(&bytes[252..256], "通道数")?;
		if header_bytes < 256 || channel_count == 0 {
			return Err(EdfLoaderError::InvalidFormat(
				"BDF 头字段不合法".to_string(),
			));
		}
		if bytes.len() < header_bytes {
			return Err(EdfLoaderError::InvalidFormat(
				"BDF 文件长度小于头长度".to_string(),
			));
		}

		let num_records = parse_ascii_usize(&bytes[236..244], "记录数")?;
		let record_duration = parse_ascii_f32(&bytes[244..252], "记录时长")?;
		if record_duration <= 0.0 {
			return Err(EdfLoaderError::InvalidFormat(
				"BDF 记录时长必须大于 0".to_string(),
			));
		}

		let signal_offset = 256;
		let labels_offset = signal_offset;
		let physical_min_offset =
			labels_offset + 16 * channel_count + 80 * channel_count + 8 * channel_count;
		let physical_max_offset = physical_min_offset + 8 * channel_count;
		let digital_min_offset = physical_max_offset + 8 * channel_count;
		let digital_max_offset = digital_min_offset + 8 * channel_count;
		let samples_per_record_offset = digital_max_offset + 8 * channel_count + 80 * channel_count;
		let reserved_offset = samples_per_record_offset + 8 * channel_count;
		let expected_header_end = reserved_offset + 32 * channel_count;
		if expected_header_end > header_bytes {
			return Err(EdfLoaderError::InvalidFormat(
				"BDF 信号头长度不完整".to_string(),
			));
		}

		let mut channels = Vec::with_capacity(channel_count);
		let mut sample_rate = 0u32;
		let mut record_size = 0usize;

		let mut signal_headers = Vec::with_capacity(channel_count);
		for index in 0..channel_count {
			let physical_min = parse_ascii_f32(
				&bytes[physical_min_offset + index * 8..physical_min_offset + (index + 1) * 8],
				"物理最小值",
			)?;
			let physical_max = parse_ascii_f32(
				&bytes[physical_max_offset + index * 8..physical_max_offset + (index + 1) * 8],
				"物理最大值",
			)?;
			let digital_min = parse_ascii_i32(
				&bytes[digital_min_offset + index * 8..digital_min_offset + (index + 1) * 8],
				"数字最小值",
			)?;
			let digital_max = parse_ascii_i32(
				&bytes[digital_max_offset + index * 8..digital_max_offset + (index + 1) * 8],
				"数字最大值",
			)?;
			let samples_per_record = parse_ascii_usize(
				&bytes[samples_per_record_offset + index * 8
					..samples_per_record_offset + (index + 1) * 8],
				"每记录采样数",
			)?;

			if index == 0 {
				sample_rate = (samples_per_record as f32 / record_duration).round() as u32;
			}
			record_size = record_size.saturating_add(samples_per_record.saturating_mul(3));
			channels.push(Vec::with_capacity(
				num_records.saturating_mul(samples_per_record),
			));
			signal_headers.push(BdfSignalHeader {
				physical_min,
				physical_max,
				digital_min,
				digital_max,
				samples_per_record,
			});
		}

		let data_bytes = &bytes[header_bytes..];
		let expected_data_size = num_records.saturating_mul(record_size);
		if data_bytes.len() < expected_data_size {
			return Err(EdfLoaderError::InvalidFormat(format!(
				"BDF 数据长度不足: 期望 {expected_data_size} 字节，实际 {} 字节",
				data_bytes.len()
			)));
		}

		for record_index in 0..num_records {
			let mut record_offset = record_index * record_size;
			for (channel_index, signal_header) in signal_headers.iter().enumerate() {
				for _ in 0..signal_header.samples_per_record {
					let sample_bytes = &data_bytes[record_offset..record_offset + 3];
					let digital_value = decode_bdf_sample(sample_bytes);
					let physical_value = digital_to_physical(digital_value, signal_header);
					channels[channel_index].push(physical_value);
					record_offset += 3;
				}
			}
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

#[derive(Debug, Clone, Copy)]
struct BdfSignalHeader {
	physical_min: f32,
	physical_max: f32,
	digital_min: i32,
	digital_max: i32,
	samples_per_record: usize,
}

fn is_bdf_file(path: &str) -> Result<bool, EdfLoaderError> {
	let bytes = fs::read(path).map_err(|error| EdfLoaderError::FileOpenError(error.to_string()))?;
	if bytes.len() < 8 {
		return Ok(false);
	}
	Ok(bytes[0] == 0xFF && &bytes[1..8] == b"BIOSEMI")
}

fn parse_ascii_usize(bytes: &[u8], field_name: &str) -> Result<usize, EdfLoaderError> {
	let text = parse_ascii_field(bytes);
	text.parse::<usize>()
		.map_err(|error| EdfLoaderError::InvalidFormat(format!("{field_name} 解析失败: {error}")))
}

fn parse_ascii_i32(bytes: &[u8], field_name: &str) -> Result<i32, EdfLoaderError> {
	let text = parse_ascii_field(bytes);
	text.parse::<i32>()
		.map_err(|error| EdfLoaderError::InvalidFormat(format!("{field_name} 解析失败: {error}")))
}

fn parse_ascii_f32(bytes: &[u8], field_name: &str) -> Result<f32, EdfLoaderError> {
	let text = parse_ascii_field(bytes);
	text.parse::<f32>()
		.map_err(|error| EdfLoaderError::InvalidFormat(format!("{field_name} 解析失败: {error}")))
}

fn parse_ascii_field(bytes: &[u8]) -> String {
	String::from_utf8_lossy(bytes).trim().to_string()
}

fn decode_bdf_sample(bytes: &[u8]) -> i32 {
	let sign_byte = if bytes[2] & 0x80 == 0 { 0x00 } else { 0xFF };
	i32::from_le_bytes([bytes[0], bytes[1], bytes[2], sign_byte])
}

fn digital_to_physical(digital_value: i32, signal_header: &BdfSignalHeader) -> f32 {
	let digital_range = (signal_header.digital_max - signal_header.digital_min) as f32;
	if digital_range.abs() <= f32::EPSILON {
		return signal_header.physical_min;
	}

	let physical_range = signal_header.physical_max - signal_header.physical_min;
	((digital_value - signal_header.digital_min) as f32 / digital_range) * physical_range
		+ signal_header.physical_min
}

#[cfg(test)]
mod tests {
	use super::EdfLoader;
	use crate::{BdfSignalParam, BdfWriter};

	#[test]
	fn can_read_generated_bdf_file() {
		let path = std::env::temp_dir().join("codex_loader_test.bdf");
		let signals = vec![
			BdfSignalParam {
				label: "EEG CH0".to_string(),
				physical_max: 200.0,
				physical_min: -200.0,
				digital_max: 8_388_607,
				digital_min: -8_388_608,
				sample_rate: 4,
				physical_dimension: "uV".to_string(),
			},
			BdfSignalParam {
				label: "EEG CH1".to_string(),
				physical_max: 200.0,
				physical_min: -200.0,
				digital_max: 8_388_607,
				digital_min: -8_388_608,
				sample_rate: 4,
				physical_dimension: "uV".to_string(),
			},
		];

		let mut writer = BdfWriter::create(&path, signals, 4, 4).unwrap();
		writer
			.write_samples(&[vec![0.0, 10.0, 20.0, 30.0], vec![5.0, 15.0, 25.0, 35.0]])
			.unwrap();
		writer.finalize().unwrap();

		let path_text = path.to_string_lossy().to_string();
		let loader = EdfLoader::from_file(&path_text).unwrap();
		std::fs::remove_file(&path).ok();

		assert_eq!(loader.channel_count(), 2);
		assert_eq!(loader.sample_rate(), 4);
		assert_eq!(loader.total_points(), 4);
		assert_eq!(loader.channels().len(), 2);
	}
}
