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
	runtime::{CandleRuntime, InferenceOutput, analyze_text},
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

/// 构建翻译结果输出路径。
pub fn build_translation_output_path() -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	output_root_dir()
		.join("translation")
		.join(format!("translation_result_{timestamp}.txt"))
}

/// 执行翻译最小推理闭环。
pub fn run_translation_inference(
	request: &TranslationRequest,
	runtime: &CandleRuntime,
) -> Result<InferenceOutput, DeepLearningError> {
	let descriptor = ensure_translation_model_ready()?;
	let source_text = std::fs::read_to_string(&request.input_path).map_err(|error| {
		DeepLearningError::InferenceFailed {
			message: format!("读取翻译输入文件失败: {error}"),
		}
	})?;
	let segments = split_translation_segments(&source_text);
	if segments.is_empty() {
		return Err(DeepLearningError::InferenceFailed {
			message: "翻译输入文本不能为空".to_string(),
		});
	}

	let global_signature = analyze_text(&source_text, runtime)?;
	let mut translated_segments = Vec::with_capacity(segments.len());
	for (index, segment) in segments.iter().enumerate() {
		let signature = analyze_text(segment, runtime)?;
		translated_segments.push(format!(
			"第{}段候选输出（{} -> Chinese，mean={:.4}，energy={:.4}）：\n{}",
			index + 1,
			request.source_language.as_label(),
			signature.mean,
			signature.energy,
			build_translation_placeholder(segment, request.source_language)
		));
	}

	let output_path = build_translation_output_path();
	let content = format!(
		"Translation Inference Result\nmodel={}\ndevice={}\nsegments={}\nmean={:.4}\nenergy={:.4}\npeak={:.4}\n\n{}\n",
		descriptor.id,
		runtime.device_label,
		segments.len(),
		global_signature.mean,
		global_signature.energy,
		global_signature.peak,
		translated_segments.join("\n\n")
	);
	std::fs::write(&output_path, content)?;

	Ok(InferenceOutput {
		summary: format!("翻译推理已完成，{} 段文本已生成结果文件", segments.len()),
		output_path: Some(output_path),
	})
}

/// 对长文本做分段，避免单段过长。
fn split_translation_segments(source_text: &str) -> Vec<String> {
	source_text
		.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty())
		.flat_map(|line| {
			line.split(['.', '!', '?', '。', '！', '？'])
				.map(str::trim)
				.filter(|segment| !segment.is_empty())
				.map(ToString::to_string)
				.collect::<Vec<_>>()
		})
		.collect()
}

/// 生成翻译占位输出文本。
fn build_translation_placeholder(
	segment: &str,
	source_language: TranslationSourceLanguage,
) -> String {
	let prefix = match source_language {
		TranslationSourceLanguage::English => "英译中最小推理占位输出",
		TranslationSourceLanguage::Japanese => "日译中最小推理占位输出",
	};
	format!("{prefix}：{}", segment.trim())
}
