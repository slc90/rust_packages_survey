use std::path::PathBuf;

use thiserror::Error;

/// 深度学习模块统一错误类型。
#[derive(Debug, Error)]
pub enum DeepLearningError {
	/// 模型目录不存在。
	#[error("模型目录不存在: {path}")]
	ModelDirectoryMissing {
		/// 缺失的模型目录路径。
		path: PathBuf,
	},

	/// 模型文件不存在。
	#[error("模型文件不存在: {path}")]
	ModelFileMissing {
		/// 缺失的模型文件路径。
		path: PathBuf,
	},

	/// 模型加载失败。
	#[error("模型加载失败: {message}")]
	ModelLoadFailed {
		/// 错误信息。
		message: String,
	},

	/// 推理执行失败。
	#[error("推理失败: {message}")]
	InferenceFailed {
		/// 错误信息。
		message: String,
	},

	/// 输出保存失败。
	#[error("输出保存失败: {message}")]
	OutputSaveFailed {
		/// 错误信息。
		message: String,
	},

	/// IO 错误。
	#[error("IO错误: {0}")]
	Io(#[from] std::io::Error),
}
