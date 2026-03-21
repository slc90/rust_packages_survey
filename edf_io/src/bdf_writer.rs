//! BDF (BioSemi Data Format) 文件写入器
//!
//! BDF 是 24 位版本的 EDF 格式，用于 BioSemi 设备采集的 EEG 数据

use rand::Rng;
use std::fs::File;
use std::io::{BufWriter, Seek, Write};
use std::path::Path;
use thiserror::Error;

/// BDF 文件写入错误
#[derive(Error, Debug)]
pub enum BdfWriterError {
	#[error("文件创建失败: {0}")]
	FileCreateError(String),

	#[error("写入失败: {0}")]
	WriteError(String),

	#[error("数据大小不匹配: 期望 {expected}, 实际 {actual}")]
	DataSizeMismatch { expected: usize, actual: usize },
}

/// BDF 信号参数
#[derive(Debug, Clone)]
pub struct BdfSignalParam {
	/// 信号标签 (如 "EEG Fp1")
	pub label: String,
	/// 物理最大值
	pub physical_max: f64,
	/// 物理最小值
	pub physical_min: f64,
	/// 数字最大值 (24-bit: 8388607)
	pub digital_max: i32,
	/// 数字最小值 (-24-bit: -8388608)
	pub digital_min: i32,
	/// 采样率
	pub sample_rate: i32,
	/// 物理单位
	pub physical_dimension: String,
}

/// BDF 文件写入器
pub struct BdfWriter {
	/// 信号参数列表
	signals: Vec<BdfSignalParam>,
	/// 通道数量
	channel_count: usize,
	/// 已写入的采样点数量
	samples_written: usize,
	/// 文件头大小
	header_size: usize,
	/// 数据记录大小 (字节)
	record_size: usize,
	/// 数据记录数量
	num_records: usize,
	/// 文件
	file: Option<BufWriter<File>>,
}

impl BdfWriter {
	/// 创建新的 BDF 文件
	///
	/// # Arguments
	/// * `path` - 输出文件路径
	/// * `signals` - 信号参数列表
	/// * `sample_rate` - 采样率 (Hz)
	/// * `total_samples` - 总采样点数
	pub fn create(
		path: &Path,
		signals: Vec<BdfSignalParam>,
		sample_rate: i32,
		total_samples: usize,
	) -> Result<Self, BdfWriterError> {
		let channel_count = signals.len();
		let header_size = 256 + (256 * channel_count);
		let num_records = total_samples.div_ceil(sample_rate as usize);

		// BDF 每个数据记录内，按通道顺序连续存放每个通道的一整段样本。
		let record_size = channel_count * sample_rate as usize * 3;

		let file =
			File::create(path).map_err(|e| BdfWriterError::FileCreateError(e.to_string()))?;

		let mut writer = Self {
			signals,
			channel_count,
			samples_written: 0,
			header_size,
			record_size,
			num_records,
			file: Some(BufWriter::new(file)),
		};

		// 写入 BDF 头
		writer.write_header()?;

		Ok(writer)
	}

	/// 写入 BDF 头信息
	fn write_header(&mut self) -> Result<(), BdfWriterError> {
		let file = self
			.file
			.as_mut()
			.ok_or_else(|| BdfWriterError::WriteError("File not opened".to_string()))?;

		let mut header = vec![0u8; self.header_size];
		// ===== 全局头 (256 字节) =====
		// BDF 版本标识: 0xFF + "BIOSEMI"
		header[0] = 0xFF;
		header[1..8].copy_from_slice(b"BIOSEMI");

		// 8-88 字节: 患者信息 (80 字节)
		write_ascii_field(&mut header[8..88], "X X X X");

		// 88-168 字节: 记录信息 (80 字节)
		write_ascii_field(&mut header[88..168], "Startdate 21-MAR-2026 Test EEG Data");

		// 168-176 字节: 开始日期 (8 字节) "dd.mm.yy"
		write_ascii_field(&mut header[168..176], "21.03.26");

		// 176-184 字节: 开始时间 (8 字节) "hh.mm.ss"
		write_ascii_field(&mut header[176..184], "00.00.00");

		// 184-192 字节: 头部长度 (8 字节 ASCII)
		write_ascii_field(&mut header[184..192], &format!("{:<8}", self.header_size));

		// 192-236 字节: 保留 (44 字节)
		write_ascii_field(&mut header[192..236], "24BIT");

		// 236-244 字节: 数据记录数量 (8 字节 ASCII)
		write_ascii_field(&mut header[236..244], &format!("{:<8}", self.num_records));

		// 244-252 字节: 每条记录的持续时间 (8 字节 ASCII, 秒)
		write_ascii_field(&mut header[244..252], &format!("{:<8}", 1));

		// 252-256 字节: 信号数量 (4 字节 ASCII)
		write_ascii_field(&mut header[252..256], &format!("{:<4}", self.channel_count));

		// ===== 信号头 =====
		// EDF/BDF 头部中的各字段需要“按字段分组”为所有通道依次写入，
		// 不是每个通道一个独立的 256 字节块。
		let mut offset = 256;
		offset = write_signal_field(&mut header, offset, 16, &self.signals, |signal| {
			signal.label.clone()
		});
		offset = write_signal_field(&mut header, offset, 80, &self.signals, |_| {
			"AgAgCl electrodes".to_string()
		});
		offset = write_signal_field(&mut header, offset, 8, &self.signals, |signal| {
			signal.physical_dimension.clone()
		});
		offset = write_signal_field(&mut header, offset, 8, &self.signals, |signal| {
			format!("{:<8}", signal.physical_min)
		});
		offset = write_signal_field(&mut header, offset, 8, &self.signals, |signal| {
			format!("{:<8}", signal.physical_max)
		});
		offset = write_signal_field(&mut header, offset, 8, &self.signals, |signal| {
			format!("{:<8}", signal.digital_min)
		});
		offset = write_signal_field(&mut header, offset, 8, &self.signals, |signal| {
			format!("{:<8}", signal.digital_max)
		});
		offset = write_signal_field(&mut header, offset, 80, &self.signals, |_| {
			"HP:0.1Hz LP:70Hz".to_string()
		});
		offset = write_signal_field(&mut header, offset, 8, &self.signals, |signal| {
			format!("{:<8}", signal.sample_rate)
		});
		let _ = write_signal_field(&mut header, offset, 32, &self.signals, |_| String::new());

		file.write_all(&header)
			.map_err(|e| BdfWriterError::WriteError(e.to_string()))?;

		Ok(())
	}

	/// 写入多通道数据
	///
	/// # Arguments
	/// * `data` - 数据，格式为 [channel][samples]，每样本点 24-bit signed
	pub fn write_samples(&mut self, data: &[Vec<f64>]) -> Result<(), BdfWriterError> {
		if data.len() != self.channel_count {
			return Err(BdfWriterError::DataSizeMismatch {
				expected: self.channel_count,
				actual: data.len(),
			});
		}

		let samples_to_write = data[0].len();
		let file = self
			.file
			.as_mut()
			.ok_or_else(|| BdfWriterError::WriteError("File not opened".to_string()))?;

		// 转换并写入每通道数据
		for (ch, channel_samples) in data.iter().enumerate().take(self.channel_count) {
			for value in channel_samples.iter().take(samples_to_write) {
				let value = *value;

				// 转换为 24-bit signed integer
				// 物理范围 [physical_min, physical_max] -> 数字范围 [digital_min, digital_max]
				let signal = &self.signals[ch];
				let normalized =
					(value - signal.physical_min) / (signal.physical_max - signal.physical_min);
				let digital_value = (normalized
					* (signal.digital_max as f64 - signal.digital_min as f64)
					+ signal.digital_min as f64) as i32;

				// 限制在 24-bit 范围内
				let digital_value = digital_value.clamp(-8388608, 8388607);

				// 转换为 3 字节小端序 (24-bit)
				let bytes = digital_value.to_le_bytes();
				file.write_all(&bytes[0..3])
					.map_err(|e| BdfWriterError::WriteError(e.to_string()))?;
			}
		}

		self.samples_written += samples_to_write;
		Ok(())
	}

	/// 完成写入并关闭文件
	pub fn finalize(mut self) -> Result<(), BdfWriterError> {
		if let Some(mut file) = self.file.take() {
			file.flush()
				.map_err(|e| BdfWriterError::WriteError(e.to_string()))?;

			// 如果有未完成的采样点，用 0 填充
			// (BDF 格式要求文件大小固定)
			let expected_size = self.header_size + (self.num_records * self.record_size);
			let current_size =
				file.stream_position()
					.map_err(|e| BdfWriterError::WriteError(e.to_string()))? as usize;

			if current_size < expected_size {
				let padding = expected_size - current_size;
				let zeros = vec![0u8; padding];
				file.write_all(&zeros)
					.map_err(|e| BdfWriterError::WriteError(e.to_string()))?;
			}

			file.flush()
				.map_err(|e| BdfWriterError::WriteError(e.to_string()))?;
		}
		Ok(())
	}
}

fn write_ascii_field(buf: &mut [u8], value: &str) {
	buf.fill(b' ');
	let bytes = value.as_bytes();
	let len = bytes.len().min(buf.len());
	buf[..len].copy_from_slice(&bytes[..len]);
}

fn write_signal_field<F>(
	header: &mut [u8],
	mut offset: usize,
	field_width: usize,
	signals: &[BdfSignalParam],
	formatter: F,
) -> usize
where
	F: Fn(&BdfSignalParam) -> String,
{
	for signal in signals {
		write_ascii_field(
			&mut header[offset..offset + field_width],
			&formatter(signal),
		);
		offset += field_width;
	}
	offset
}

/// 生成测试 BDF 文件
pub fn generate_test_bdf(
	output_path: &Path,
	channel_count: usize,
	sample_rate: u32,
	duration_secs: u32,
) -> Result<(), BdfWriterError> {
	let samples_per_channel = (sample_rate as usize) * (duration_secs as usize);

	eprintln!(
		"生成测试 BDF 文件: {} 通道, {} Hz, {} 秒, 共 {} 采样点/通道",
		channel_count, sample_rate, duration_secs, samples_per_channel
	);

	// 创建信号参数
	let signals: Vec<BdfSignalParam> = (0..channel_count)
		.map(|ch| BdfSignalParam {
			label: format!("EEG CH{}", ch),
			physical_max: 200.0,
			physical_min: -200.0,
			digital_max: 8388607,
			digital_min: -8388608,
			sample_rate: sample_rate as i32,
			physical_dimension: "uV".to_string(),
		})
		.collect();

	// 创建 BDF 写入器
	let mut writer = BdfWriter::create(
		output_path,
		signals,
		sample_rate as i32,
		samples_per_channel,
	)?;

	// 生成并写入数据
	let samples_per_record = sample_rate as usize;
	let record_count = samples_per_channel / samples_per_record;

	eprintln!("写入 {} 个数据记录...", record_count);

	let mut rng = rand::rng();
	let base_freq = 10.0; // 基础频率 10 Hz

	for record in 0..record_count {
		let mut channel_data: Vec<Vec<f64>> =
			vec![Vec::with_capacity(samples_per_record); channel_count];

		for sample_idx in 0..samples_per_record {
			let t = (record * samples_per_record + sample_idx) as f64 / sample_rate as f64;

			for channel_samples in channel_data.iter_mut().take(channel_count) {
				// 生成模拟 EEG 信号：正弦波 + 噪声
				let signal_val = (t * base_freq * 2.0 * std::f64::consts::PI).sin() * 50.0;
				let noise: f64 = (rng.random::<f64>() - 0.5) * 20.0;
				let value = signal_val + noise;
				channel_samples.push(value);
			}
		}

		writer.write_samples(&channel_data)?;

		if (record + 1) % 10 == 0 || record == record_count - 1 {
			eprintln!("进度: {}/{} 记录", record + 1, record_count);
		}
	}

	writer.finalize()?;

	eprintln!("文件生成完成: {:?}", output_path);

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;

	fn test_signals() -> Vec<BdfSignalParam> {
		vec![
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
				physical_max: 100.0,
				physical_min: -100.0,
				digital_max: 8_388_607,
				digital_min: -8_388_608,
				sample_rate: 4,
				physical_dimension: "uV".to_string(),
			},
		]
	}

	#[test]
	fn writes_bdf_header_in_field_major_layout() {
		let path = std::env::temp_dir().join("codex_bdf_header_test.bdf");
		let writer = BdfWriter::create(&path, test_signals(), 4, 4).unwrap();
		writer.finalize().unwrap();

		let data = fs::read(&path).unwrap();
		fs::remove_file(&path).ok();

		assert_eq!(data[0], 0xFF);
		assert_eq!(&data[1..8], b"BIOSEMI");
		assert_eq!(&data[184..192], b"768     ");
		assert_eq!(&data[236..244], b"1       ");
		assert_eq!(&data[252..256], b"2   ");

		assert_eq!(&data[256..272], b"EEG CH0         ");
		assert_eq!(&data[272..288], b"EEG CH1         ");
		assert_eq!(&data[288..305], b"AgAgCl electrodes");
		assert!(data[305..368].iter().all(|byte| *byte == b' '));
	}

	#[test]
	fn pads_file_to_full_record_size() {
		let path = std::env::temp_dir().join("codex_bdf_size_test.bdf");
		let mut writer = BdfWriter::create(&path, test_signals(), 4, 5).unwrap();
		let data = vec![vec![0.0; 4], vec![0.0; 4]];
		writer.write_samples(&data).unwrap();
		writer.finalize().unwrap();

		let metadata = fs::metadata(&path).unwrap();
		fs::remove_file(&path).ok();

		// 2 个记录，2 通道，4 样本/记录，24-bit => 768 + 2 * (2 * 4 * 3)
		assert_eq!(metadata.len(), 816);
	}
}
