use std::{path::PathBuf, time::Instant};

use bevy::prelude::*;
use bevy::tasks::Task;
use deep_learning::{
	error::DeepLearningError,
	image_generation::{
		ImageGenerationModelKind, ImageGenerationRequest, ImageGenerationResolution,
	},
	runtime::InferenceOutput,
	runtime::RuntimeDirectories,
	separation::SeparationRequest,
	task::DlTaskId,
	translation::{TranslationRequest, TranslationSourceLanguage},
	tts::{TtsLanguage, TtsRequest},
	whisper::{WhisperLanguageHint, WhisperModelKind, WhisperRequest},
};

/// 深度学习测试页状态。
#[derive(Resource, Debug, Clone)]
pub struct DeepLearningPageState {
	/// 模型根目录。
	pub model_root: String,

	/// 输出根目录。
	pub output_root: String,

	/// 当前状态文本。
	pub status_text: String,

	/// 当前结果文本。
	pub result_text: String,

	/// 下一个任务 ID。
	pub next_task_id: u64,

	/// Whisper 选中的输入文件。
	pub whisper_input_file: Option<PathBuf>,

	/// Whisper 语言提示。
	pub whisper_language_hint: WhisperLanguageHint,

	/// Whisper 模型类型。
	pub whisper_model: WhisperModelKind,

	/// Whisper 是否输出时间戳。
	pub whisper_with_timestamps: bool,

	/// Whisper 当前进度。
	pub whisper_progress: f32,

	/// Whisper 当前状态文本。
	pub whisper_status_text: String,

	/// 翻译选中的输入文件。
	pub translation_input_file: Option<PathBuf>,

	/// 翻译源语言。
	pub translation_source_language: TranslationSourceLanguage,

	/// TTS 选中的输入文件。
	pub tts_input_file: Option<PathBuf>,

	/// TTS 输出语言。
	pub tts_language: TtsLanguage,

	/// TTS 说话人。
	pub tts_speaker: String,

	/// TTS 语速倍率。
	pub tts_speed: f32,

	/// 人声分离选中的输入文件。
	pub separation_input_file: Option<PathBuf>,

	/// 图片生成 Prompt 文件。
	pub image_generation_prompt_file: Option<PathBuf>,

	/// 图片生成分辨率。
	pub image_generation_resolution: ImageGenerationResolution,

	/// 图片生成随机种子。
	pub image_generation_seed: u64,

	/// 图片生成采样步数。
	pub image_generation_steps: u32,

	/// 图片生成模型模式。
	pub image_generation_model: ImageGenerationModelKind,

	/// 图片预览纹理句柄。
	pub image_generation_preview_texture: Handle<Image>,

	/// 图片预览输出路径。
	pub image_generation_preview_path: Option<PathBuf>,
}

impl DeepLearningPageState {
	/// 根据运行时目录创建页面状态。
	pub fn new(directories: &RuntimeDirectories, preview_texture: Handle<Image>) -> Self {
		Self {
			model_root: directories.model_root.display().to_string(),
			output_root: directories.output_root.display().to_string(),
			status_text: "等待任务".to_string(),
			result_text: "暂无结果".to_string(),
			next_task_id: 1,
			whisper_input_file: None,
			whisper_language_hint: WhisperLanguageHint::Auto,
			whisper_model: WhisperModelKind::Base,
			whisper_with_timestamps: true,
			whisper_progress: 0.0,
			whisper_status_text: "Whisper 进度：等待任务".to_string(),
			translation_input_file: None,
			translation_source_language: TranslationSourceLanguage::English,
			tts_input_file: None,
			tts_language: TtsLanguage::Chinese,
			tts_speaker: "default".to_string(),
			tts_speed: 1.0,
			separation_input_file: None,
			image_generation_prompt_file: None,
			image_generation_resolution: ImageGenerationResolution::Size1024x768,
			image_generation_seed: 20260322,
			image_generation_steps: 4,
			image_generation_model: ImageGenerationModelKind::SdxlTurbo,
			image_generation_preview_texture: preview_texture,
			image_generation_preview_path: None,
		}
	}

	/// 生成新的任务 ID。
	pub fn allocate_task_id(&mut self) -> DlTaskId {
		let task_id = DlTaskId(self.next_task_id);
		self.next_task_id += 1;
		task_id
	}

	/// 获取 Whisper 请求。
	pub fn build_whisper_request(&self) -> Option<WhisperRequest> {
		let input_path = self.whisper_input_file.clone()?;
		Some(WhisperRequest {
			input_path,
			model: self.whisper_model,
			language_hint: self.whisper_language_hint,
			with_timestamps: self.whisper_with_timestamps,
		})
	}

	/// 获取翻译请求。
	pub fn build_translation_request(&self) -> Option<TranslationRequest> {
		let input_path = self.translation_input_file.clone()?;
		Some(TranslationRequest {
			input_path,
			source_language: self.translation_source_language,
		})
	}

	/// 获取 TTS 请求。
	pub fn build_tts_request(&self) -> Option<TtsRequest> {
		let input_path = self.tts_input_file.clone()?;
		Some(TtsRequest {
			input_path,
			language: self.tts_language,
			speaker: self.tts_speaker.clone(),
			speed: self.tts_speed,
		})
	}

	/// 获取人声分离请求。
	pub fn build_separation_request(&self) -> Option<SeparationRequest> {
		let input_path = self.separation_input_file.clone()?;
		Some(SeparationRequest { input_path })
	}

	/// 获取图像生成请求。
	pub fn build_image_generation_request(&self) -> Option<ImageGenerationRequest> {
		let prompt_path = self.image_generation_prompt_file.clone()?;
		Some(ImageGenerationRequest {
			prompt_path,
			resolution: self.image_generation_resolution,
			seed: self.image_generation_seed,
			steps: self.image_generation_steps,
			model: self.image_generation_model,
		})
	}
}

/// 后台推理任务。
pub struct PendingInferenceTask {
	/// 任务 ID。
	pub id: DlTaskId,

	/// 任务类型。
	pub kind: deep_learning::task::DlTaskKind,

	/// 异步任务句柄。
	pub task: Task<Result<InferenceOutput, DeepLearningError>>,

	/// 任务启动时间。
	pub started_at: Instant,
}

/// 后台推理任务队列资源。
#[derive(Resource, Default)]
pub struct DeepLearningPendingTasks {
	/// 当前待完成任务列表。
	pub tasks: Vec<PendingInferenceTask>,
}
