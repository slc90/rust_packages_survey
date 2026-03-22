use std::path::PathBuf;

use crate::error::DeepLearningError;

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
	PathBuf::from("deep_learning_output")
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
