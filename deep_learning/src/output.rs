use std::path::PathBuf;

use crate::{
	error::DeepLearningError,
	model::{local_app_data_root_dir, workspace_root_dir, workspace_root_dir_if_available},
};

/// 文本输出结果。
#[derive(Debug, Clone)]
pub struct TextOutput {
	/// 输出文件路径。
	pub path: PathBuf,

	/// 输出文本内容。
	pub text: String,
}

/// 音频输出结果。
#[derive(Debug, Clone)]
pub struct AudioOutput {
	/// 输出文件路径。
	pub path: PathBuf,
}

/// 图片输出结果。
#[derive(Debug, Clone)]
pub struct ImageOutput {
	/// 输出文件路径。
	pub path: PathBuf,
}

/// 获取深度学习输出根目录。
pub fn output_root_dir() -> PathBuf {
	if let Some(workspace_root_dir) = workspace_root_dir_if_available() {
		return workspace_root_dir.join("deep_learning_output");
	}

	if let Some(local_app_data_dir) = local_app_data_root_dir() {
		return local_app_data_dir.join("deep_learning_output");
	}

	workspace_root_dir().join("deep_learning_output")
}

/// 创建所有输出目录。
pub fn ensure_output_directories() -> Result<(), DeepLearningError> {
	let root = output_root_dir();
	std::fs::create_dir_all(root.join("translation"))?;
	std::fs::create_dir_all(root.join("separation"))?;
	std::fs::create_dir_all(root.join("whisper"))?;
	std::fs::create_dir_all(root.join("image_generation"))?;
	std::fs::create_dir_all(root.join("tts"))?;
	Ok(())
}
