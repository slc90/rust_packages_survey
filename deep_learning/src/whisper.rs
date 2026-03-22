use std::{
	path::PathBuf,
	time::{SystemTime, UNIX_EPOCH},
};

use crate::{
	error::DeepLearningError,
	model::{
		ModelCapability, ModelDescriptor, ensure_model_directory_exists,
		ensure_model_weights_exist, model_dir, model_weights_path,
	},
	output::output_root_dir,
};

/// Whisper 语言提示。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhisperLanguageHint {
	/// 自动检测语言。
	Auto,

	/// 中文提示。
	Chinese,

	/// 日文提示。
	Japanese,

	/// 英文提示。
	English,
}

impl WhisperLanguageHint {
	/// 获取语言提示文本。
	pub fn as_label(self) -> &'static str {
		match self {
			Self::Auto => "Auto",
			Self::Chinese => "Chinese",
			Self::Japanese => "Japanese",
			Self::English => "English",
		}
	}
}

/// Whisper 请求参数。
#[derive(Debug, Clone)]
pub struct WhisperRequest {
	/// 输入音频或视频文件路径。
	pub input_path: PathBuf,

	/// 语言提示。
	pub language_hint: WhisperLanguageHint,

	/// 是否输出时间戳。
	pub with_timestamps: bool,
}

/// Whisper Large v3 模型描述。
pub fn whisper_large_v3_descriptor() -> ModelDescriptor {
	ModelDescriptor {
		id: "openai/whisper-large-v3",
		capability: ModelCapability::Whisper,
		model_subdir: "whisper-large-v3",
		weights_relative_path: "model.safetensors",
	}
}

/// 校验 Whisper 模型目录和主权重文件。
pub fn ensure_whisper_model_ready() -> Result<ModelDescriptor, DeepLearningError> {
	let descriptor = whisper_large_v3_descriptor();
	let directory = model_dir(&descriptor);
	let weights = model_weights_path(&descriptor);
	ensure_model_directory_exists(&directory)?;
	ensure_model_weights_exist(&weights)?;
	Ok(descriptor)
}

/// 构建 Whisper 任务快照输出路径。
pub fn build_whisper_request_snapshot_path() -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	output_root_dir()
		.join("whisper")
		.join(format!("whisper_request_{timestamp}.txt"))
}

/// 将 Whisper 请求写出为任务快照文件。
pub fn save_whisper_request_snapshot(
	request: &WhisperRequest,
) -> Result<PathBuf, DeepLearningError> {
	let output_path = build_whisper_request_snapshot_path();
	let content = format!(
		"Whisper Phase 2 任务快照\ninput={}\nlanguage_hint={}\nwith_timestamps={}\n",
		request.input_path.display(),
		request.language_hint.as_label(),
		request.with_timestamps
	);
	std::fs::write(&output_path, content)?;
	Ok(output_path)
}
