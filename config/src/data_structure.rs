use bevy::{
	ecs::resource::Resource,
	log::{debug, error, info},
};
use i18n::data_structure::Language;
use serde::{Deserialize, Serialize};

/// 波形配置
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct WaveformConfig {
	/// 通道数量
	pub channel_count: usize,
	/// 采样率 (Hz)
	pub sample_rate: u32,
	/// 最大显示点数
	pub buffer_size: usize,
}

impl Default for WaveformConfig {
	fn default() -> Self {
		Self {
			channel_count: 1,
			sample_rate: 1000,
			buffer_size: 4096,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Setting {
	pub language: Language,
	pub waveform: WaveformConfig,
}

impl Default for Setting {
	fn default() -> Self {
		Self {
			language: Language::Chinese,
			waveform: WaveformConfig::default(),
		}
	}
}

/// 从文件中读取设置，如果文件不存在或格式错误，则返回默认设置
pub fn read_from_file_or_default(config_file_path: &str) -> Setting {
	let file = std::fs::File::open(config_file_path);
	match file {
		Ok(file) => match serde_json::from_reader(file) {
			Ok(setting) => {
				debug!("读取配置文件成功:{}", config_file_path);
				setting
			}
			Err(_) => {
				error!("解析配置文件失败:{}, 使用默认配置", config_file_path);
				Setting::default()
			}
		},
		Err(_) => {
			error!("打开配置文件失败:{}, 使用默认配置", config_file_path);
			Setting::default()
		}
	}
}

/// 保存设置到文件
pub fn save_to_file(setting: &Setting, config_file_path: &str) -> Result<(), String> {
	let file = std::fs::File::create(config_file_path);
	match file {
		Ok(file) => match serde_json::to_writer_pretty(file, setting) {
			Ok(_) => {
				info!("保存配置文件成功:{}", config_file_path);
				Ok(())
			}
			Err(e) => {
				error!("保存配置文件失败:{}, 错误:{}", config_file_path, e);
				Err(format!("保存配置文件失败: {}", e))
			}
		},
		Err(e) => {
			error!("创建配置文件失败:{}, 错误:{}", config_file_path, e);
			Err(format!("创建配置文件失败: {}", e))
		}
	}
}

#[derive(Debug, Clone, Resource)]
pub struct ConfigPath(String);

impl ConfigPath {
	pub fn new(path: String) -> Self {
		Self(path)
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}
}
