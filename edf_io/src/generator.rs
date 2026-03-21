//! 测试数据生成器
//!
//! 生成 EDF+ 格式的测试数据文件

use edfplus::{EdfWriter, SignalParam};
use rand::Rng;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use thiserror::Error;

/// 测试数据生成错误
#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum GeneratorError {
	#[error("文件创建失败: {0}")]
	FileCreateError(String),

	#[error("数据写入失败: {0}")]
	WriteError(String),

	#[error("EDF+ 错误: {0}")]
	EdfError(String),
}

impl From<edfplus::EdfError> for GeneratorError {
	fn from(err: edfplus::EdfError) -> Self {
		GeneratorError::EdfError(err.to_string())
	}
}

/// 测试 EDF+ 数据生成器
///
/// 生成模拟 EEG 数据的 EDF+ 测试文件
pub struct TestEdfGenerator {
	/// 通道数量
	channel_count: usize,
	/// 采样率 (Hz)
	sample_rate: u32,
	/// 时长 (秒)
	duration_secs: u32,
}

impl TestEdfGenerator {
	/// 创建新的生成器
	///
	/// # Arguments
	/// * `channel_count` - 通道数量
	/// * `sample_rate` - 采样率 (Hz)
	/// * `duration_secs` - 时长 (秒)
	pub fn new(channel_count: usize, sample_rate: u32, duration_secs: u32) -> Self {
		Self {
			channel_count,
			sample_rate,
			duration_secs,
		}
	}

	/// 生成测试 EDF+ 文件
	///
	/// # Arguments
	/// * `output_path` - 输出文件路径
	///
	/// # Returns
	/// 成功返回 Ok(())，失败返回错误信息
	pub fn generate(&self, output_path: &Path) -> Result<(), GeneratorError> {
		let samples_per_channel = (self.sample_rate as usize) * (self.duration_secs as usize);

		eprintln!(
			"生成测试 EDF+ 文件: {} 通道, {} Hz, {} 秒, 共 {} 采样点/通道",
			self.channel_count, self.sample_rate, self.duration_secs, samples_per_channel
		);

		// 创建 EDF+ 文件写入器
		let mut writer = EdfWriter::create(output_path)?;

		// 设置患者信息
		writer.set_patient_info("TEST001", "M", "01-JAN-2024", "Test EEG Data")?;

		// 添加信号通道
		for ch in 0..self.channel_count {
			let signal = SignalParam {
				label: format!("EEG CH{}", ch),
				samples_in_file: 0,
				physical_max: 200.0,
				physical_min: -200.0,
				digital_max: 32767,
				digital_min: -32768,
				samples_per_record: self.sample_rate as i32,
				physical_dimension: "uV".to_string(),
				prefilter: "HP:0.1Hz LP:70Hz".to_string(),
				transducer: "AgAgCl cup electrodes".to_string(),
			};
			writer.add_signal(signal)?;
		}

		// 生成并写入数据
		let samples_per_record = self.sample_rate as usize;
		let record_count = samples_per_channel / samples_per_record;

		eprintln!("写入 {} 个数据记录...", record_count);

		let mut rng = rand::rng();
		let base_freq = 10.0; // 基础频率 10 Hz

		for record in 0..record_count {
			let mut channel_data: Vec<Vec<f64>> =
				vec![Vec::with_capacity(samples_per_record); self.channel_count];

			for sample_idx in 0..samples_per_record {
				let t = (record * samples_per_record + sample_idx) as f64 / self.sample_rate as f64;

				for channel_samples in channel_data.iter_mut().take(self.channel_count) {
					// 生成模拟 EEG 信号：正弦波 + 噪声
					let signal_val = (t * base_freq * 2.0 * std::f64::consts::PI).sin() * 50.0;
					let noise: f64 = (rng.random::<f64>() - 0.5) * 20.0;
					let value = signal_val + noise;
					channel_samples.push(value);
				}
			}

			// 写入这一秒的数据
			writer.write_samples(&channel_data)?;

			if (record + 1) % 10 == 0 || record == record_count - 1 {
				eprintln!("进度: {}/{} 记录", record + 1, record_count);
			}
		}

		writer.finalize()?;
		sanitize_edf_header(output_path)?;

		eprintln!("文件生成完成: {:?}", output_path);

		Ok(())
	}
}

impl Default for TestEdfGenerator {
	fn default() -> Self {
		Self::new(64, 4000, 600) // 64 通道, 4000 Hz, 10 分钟
	}
}

fn sanitize_edf_header(path: &Path) -> Result<(), GeneratorError> {
	let mut file = OpenOptions::new()
		.read(true)
		.write(true)
		.open(path)
		.map_err(|e| GeneratorError::FileCreateError(e.to_string()))?;

	let mut fixed_header = vec![0u8; 256];
	file.read_exact(&mut fixed_header)
		.map_err(|e| GeneratorError::WriteError(e.to_string()))?;

	for byte in &mut fixed_header {
		if *byte == 0 {
			*byte = b' ';
		}
	}
	normalize_recording_field(&mut fixed_header)?;

	let header_len_str = std::str::from_utf8(&fixed_header[184..192])
		.map_err(|e| GeneratorError::EdfError(e.to_string()))?;
	let header_len = header_len_str
		.trim()
		.parse::<usize>()
		.map_err(|e| GeneratorError::EdfError(e.to_string()))?;

	let mut full_header = vec![0u8; header_len];
	full_header[..256].copy_from_slice(&fixed_header);
	file.read_exact(&mut full_header[256..])
		.map_err(|e| GeneratorError::WriteError(e.to_string()))?;

	for byte in &mut full_header[256..] {
		if *byte == 0 {
			*byte = b' ';
		}
	}

	file.seek(SeekFrom::Start(0))
		.map_err(|e| GeneratorError::WriteError(e.to_string()))?;
	file.write_all(&full_header)
		.map_err(|e| GeneratorError::WriteError(e.to_string()))?;
	file.flush()
		.map_err(|e| GeneratorError::WriteError(e.to_string()))?;

	Ok(())
}

fn normalize_recording_field(main_header: &mut [u8]) -> Result<(), GeneratorError> {
	let start_date = std::str::from_utf8(&main_header[168..176])
		.map_err(|e| GeneratorError::EdfError(e.to_string()))?;
	let parts: Vec<&str> = start_date.trim().split('.').collect();
	if parts.len() != 3 {
		return Err(GeneratorError::EdfError(format!(
			"无法解析 EDF 开始日期字段: {start_date}"
		)));
	}

	let day = parts[0];
	let month = match parts[1] {
		"01" => "JAN",
		"02" => "FEB",
		"03" => "MAR",
		"04" => "APR",
		"05" => "MAY",
		"06" => "JUN",
		"07" => "JUL",
		"08" => "AUG",
		"09" => "SEP",
		"10" => "OCT",
		"11" => "NOV",
		"12" => "DEC",
		other => {
			return Err(GeneratorError::EdfError(format!(
				"无法解析 EDF 开始日期月份: {other}"
			)));
		}
	};
	let year_suffix = parts[2]
		.parse::<u32>()
		.map_err(|e| GeneratorError::EdfError(e.to_string()))?;
	let year = if year_suffix >= 85 {
		1900 + year_suffix
	} else {
		2000 + year_suffix
	};

	let recording_field = format!("Startdate {day}-{month}-{year} X X X X");
	write_ascii_field(&mut main_header[88..168], &recording_field);
	Ok(())
}

fn write_ascii_field(buf: &mut [u8], value: &str) {
	buf.fill(b' ');
	let bytes = value.as_bytes();
	let len = bytes.len().min(buf.len());
	buf[..len].copy_from_slice(&bytes[..len]);
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;

	#[test]
	fn sanitize_header_replaces_nul_bytes_with_spaces() {
		let path = std::env::temp_dir().join("codex_edf_header_sanitize_test.edf");
		let mut bytes = vec![b' '; 512];
		bytes[0..8].copy_from_slice(b"0       ");
		bytes[184..192].copy_from_slice(b"     512");
		bytes[168..176].copy_from_slice(b"21.03.26");
		bytes[20] = 0;
		bytes[260] = 0;
		fs::write(&path, &bytes).unwrap();

		sanitize_edf_header(&path).unwrap();

		let rewritten = fs::read(&path).unwrap();
		fs::remove_file(&path).ok();

		assert_eq!(rewritten[20], b' ');
		assert_eq!(rewritten[260], b' ');
	}

	#[test]
	fn normalize_recording_field_writes_edf_plus_compliant_date() {
		let mut header = vec![b' '; 256];
		header[168..176].copy_from_slice(b"21.03.26");

		normalize_recording_field(&mut header).unwrap();

		let recording = std::str::from_utf8(&header[88..168]).unwrap().trim_end();
		assert_eq!(recording, "Startdate 21-MAR-2026 X X X X");
	}
}
