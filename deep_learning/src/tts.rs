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

/// TTS 语言。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TtsLanguage {
	/// 中文。
	Chinese,

	/// 日文。
	Japanese,
}

impl TtsLanguage {
	/// 获取语言标签。
	pub fn as_label(self) -> &'static str {
		match self {
			Self::Chinese => "Chinese",
			Self::Japanese => "Japanese",
		}
	}
}

/// TTS 请求。
#[derive(Debug, Clone)]
pub struct TtsRequest {
	/// 输入文本文件路径。
	pub input_path: PathBuf,

	/// 输出语言。
	pub language: TtsLanguage,

	/// 说话人。
	pub speaker: String,

	/// 语速倍率。
	pub speed: f32,
}

/// Qwen3-TTS 主模型描述。
pub fn qwen3_tts_descriptor() -> ModelDescriptor {
	ModelDescriptor {
		id: "Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice",
		capability: ModelCapability::Tts,
		model_subdir: "qwen3-tts-12hz-1.7b-customvoice",
		weights_relative_path: "model.safetensors",
	}
}

/// 校验 TTS 模型目录和主权重文件。
pub fn ensure_tts_model_ready() -> Result<ModelDescriptor, DeepLearningError> {
	let descriptor = qwen3_tts_descriptor();
	let directory = model_dir(&descriptor);
	let weights = model_weights_path(&descriptor);
	ensure_model_directory_exists(&directory)?;
	ensure_model_weights_exist(&weights)?;
	Ok(descriptor)
}

/// 构建 TTS 请求快照路径。
pub fn build_tts_request_snapshot_path() -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	output_root_dir()
		.join("tts")
		.join(format!("tts_request_{timestamp}.txt"))
}

/// 保存 TTS 请求快照。
pub fn save_tts_request_snapshot(request: &TtsRequest) -> Result<PathBuf, DeepLearningError> {
	let output_path = build_tts_request_snapshot_path();
	let content = format!(
		"TTS Phase 3 任务快照\ninput={}\nlanguage={}\nspeaker={}\nspeed={}\n",
		request.input_path.display(),
		request.language.as_label(),
		request.speaker,
		request.speed
	);
	std::fs::write(&output_path, content)?;
	Ok(output_path)
}
