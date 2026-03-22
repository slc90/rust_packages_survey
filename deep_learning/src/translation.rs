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

/// 翻译源语言。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TranslationSourceLanguage {
	/// 英文。
	English,

	/// 日文。
	Japanese,
}

impl TranslationSourceLanguage {
	/// 获取源语言标签。
	pub fn as_label(self) -> &'static str {
		match self {
			Self::English => "English",
			Self::Japanese => "Japanese",
		}
	}
}

/// 翻译请求。
#[derive(Debug, Clone)]
pub struct TranslationRequest {
	/// 输入文本文件路径。
	pub input_path: PathBuf,

	/// 源语言。
	pub source_language: TranslationSourceLanguage,
}

/// GlotMAX-17-8B 模型描述。
pub fn glotmax_17_8b_descriptor() -> ModelDescriptor {
	ModelDescriptor {
		id: "LLaMAX/GlotMAX-17-8B",
		capability: ModelCapability::Translation,
		model_subdir: "glotmax-17-8b",
		weights_relative_path: "model.safetensors",
	}
}

/// 校验翻译模型目录和主权重文件。
pub fn ensure_translation_model_ready() -> Result<ModelDescriptor, DeepLearningError> {
	let descriptor = glotmax_17_8b_descriptor();
	let directory = model_dir(&descriptor);
	let weights = model_weights_path(&descriptor);
	ensure_model_directory_exists(&directory)?;
	ensure_model_weights_exist(&weights)?;
	Ok(descriptor)
}

/// 构建翻译请求快照路径。
pub fn build_translation_request_snapshot_path() -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	output_root_dir()
		.join("translation")
		.join(format!("translation_request_{timestamp}.txt"))
}

/// 保存翻译请求快照。
pub fn save_translation_request_snapshot(
	request: &TranslationRequest,
) -> Result<PathBuf, DeepLearningError> {
	let output_path = build_translation_request_snapshot_path();
	let content = format!(
		"Translation Phase 3 任务快照\ninput={}\nsource_language={}\ntarget_language=Chinese\n",
		request.input_path.display(),
		request.source_language.as_label(),
	);
	std::fs::write(&output_path, content)?;
	Ok(output_path)
}
