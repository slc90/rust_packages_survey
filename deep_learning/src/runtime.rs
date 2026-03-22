use std::path::PathBuf;

use crate::{
	error::DeepLearningError, model::ensure_model_directories, output::ensure_output_directories,
};

/// 深度学习运行时目录信息。
#[derive(Debug, Clone)]
pub struct RuntimeDirectories {
	/// 模型根目录。
	pub model_root: PathBuf,

	/// 输出根目录。
	pub output_root: PathBuf,
}

/// 初始化 Phase 1 所需的运行时目录。
pub fn initialize_runtime_directories() -> Result<RuntimeDirectories, DeepLearningError> {
	ensure_model_directories()?;
	ensure_output_directories()?;

	Ok(RuntimeDirectories {
		model_root: PathBuf::from("deepl_models"),
		output_root: PathBuf::from("deep_learning_output"),
	})
}
