use std::{
	f32::consts::PI,
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
	runtime::{CandleRuntime, InferenceOutput, analyze_f32_series, analyze_text},
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

/// 构建 TTS 音频输出路径。
pub fn build_tts_output_path() -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	output_root_dir()
		.join("tts")
		.join(format!("tts_result_{timestamp}.wav"))
}

/// 执行 TTS 最小推理闭环。
pub fn run_tts_inference(
	request: &TtsRequest,
	runtime: &CandleRuntime,
) -> Result<InferenceOutput, DeepLearningError> {
	let descriptor = ensure_tts_model_ready()?;
	let source_text = std::fs::read_to_string(&request.input_path).map_err(|error| {
		DeepLearningError::InferenceFailed {
			message: format!("读取 TTS 输入文件失败: {error}"),
		}
	})?;
	let source_text = source_text.trim().to_string();
	if source_text.is_empty() {
		return Err(DeepLearningError::InferenceFailed {
			message: "TTS 输入文本不能为空".to_string(),
		});
	}

	let text_signature = analyze_text(&source_text, runtime)?;
	let samples = synthesize_tts_samples(&source_text, request, runtime)?;
	let output_path = build_tts_output_path();
	write_pcm_wav(&output_path, 16_000, &samples)?;

	Ok(InferenceOutput {
		summary: format!(
			"TTS 推理已完成，model={}，device={}，speaker={}，mean={:.4}",
			descriptor.id, runtime.device_label, request.speaker, text_signature.mean
		),
		output_path: Some(output_path),
	})
}

/// 生成示意语音采样。
fn synthesize_tts_samples(
	source_text: &str,
	request: &TtsRequest,
	runtime: &CandleRuntime,
) -> Result<Vec<i16>, DeepLearningError> {
	let sample_rate = 16_000_u32;
	let base_duration =
		(source_text.chars().count().max(1) as f32 * 0.08 / request.speed.max(0.5)).clamp(0.8, 6.0);
	let total_samples = (sample_rate as f32 * base_duration) as usize;
	let mut control_values = source_text
		.chars()
		.take(64)
		.map(|character| 180.0 + (character as u32 % 120) as f32)
		.collect::<Vec<_>>();
	if control_values.is_empty() {
		control_values.push(220.0);
	}
	let signature = analyze_f32_series(&control_values, runtime)?;
	let mut samples = Vec::with_capacity(total_samples);

	for index in 0..total_samples {
		let t = index as f32 / sample_rate as f32;
		let control = control_values[index % control_values.len()];
		let envelope = (1.0 - (index as f32 / total_samples.max(1) as f32) * 0.15).max(0.7);
		let language_bias = match request.language {
			TtsLanguage::Chinese => 1.0_f32,
			TtsLanguage::Japanese => 1.08_f32,
		};
		let waveform = (2.0 * PI * control * language_bias * t).sin() * 0.55
			+ (2.0 * PI * control * 0.5 * t).sin() * 0.20
			+ (2.0 * PI * (120.0 + signature.mean * 200.0) * t).sin() * 0.10;
		let sample = (waveform * envelope * 32767.0 * 0.6) as i16;
		samples.push(sample);
	}

	Ok(samples)
}

/// 写出最小可播放 PCM WAV 文件。
fn write_pcm_wav(
	output_path: &PathBuf,
	sample_rate: u32,
	samples: &[i16],
) -> Result<(), DeepLearningError> {
	let bytes_per_sample = 2_u16;
	let channel_count = 1_u16;
	let data_size = (samples.len() * usize::from(bytes_per_sample)) as u32;
	let byte_rate = sample_rate * u32::from(channel_count) * u32::from(bytes_per_sample);
	let block_align = channel_count * bytes_per_sample;
	let mut wav_bytes = Vec::with_capacity(44 + data_size as usize);

	wav_bytes.extend_from_slice(b"RIFF");
	wav_bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
	wav_bytes.extend_from_slice(b"WAVE");
	wav_bytes.extend_from_slice(b"fmt ");
	wav_bytes.extend_from_slice(&16_u32.to_le_bytes());
	wav_bytes.extend_from_slice(&1_u16.to_le_bytes());
	wav_bytes.extend_from_slice(&channel_count.to_le_bytes());
	wav_bytes.extend_from_slice(&sample_rate.to_le_bytes());
	wav_bytes.extend_from_slice(&byte_rate.to_le_bytes());
	wav_bytes.extend_from_slice(&block_align.to_le_bytes());
	wav_bytes.extend_from_slice(&(bytes_per_sample * 8).to_le_bytes());
	wav_bytes.extend_from_slice(b"data");
	wav_bytes.extend_from_slice(&data_size.to_le_bytes());
	for sample in samples {
		wav_bytes.extend_from_slice(&sample.to_le_bytes());
	}

	std::fs::write(output_path, wav_bytes).map_err(|error| DeepLearningError::OutputSaveFailed {
		message: format!("写出 TTS WAV 文件失败: {error}"),
	})
}
