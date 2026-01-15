use bevy::{ecs::resource::Resource, log::error};
use i18n::data_structure::Language;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Setting {
	pub language: Language,
}

impl Default for Setting {
	fn default() -> Self {
		Self {
			language: Language::Chinese,
		}
	}
}

/// 从文件中读取设置，如果文件不存在或格式错误，则返回默认设置
pub fn read_from_file_or_default(config_file_path: &str) -> Setting {
	let file = std::fs::File::open(config_file_path);
	match file {
		Ok(file) => match serde_json::from_reader(file) {
			Ok(setting) => setting,
			Err(_) => {
				error!("解析配置文件失败: {}, 使用默认配置", config_file_path);
				Setting::default()
			}
		},
		Err(_) => {
			error!("打开配置文件失败: {}, 使用默认配置", config_file_path);
			Setting::default()
		}
	}
}
