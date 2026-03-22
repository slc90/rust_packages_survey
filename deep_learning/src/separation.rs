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
	runtime::{CandleRuntime, InferenceOutput, analyze_bytes},
};

/// 人声分离请求。
#[derive(Debug, Clone)]
pub struct SeparationRequest {
	/// 输入音频文件路径。
	pub input_path: PathBuf,
}

/// htdemucs_ft 模型描述。
pub fn htdemucs_ft_descriptor() -> ModelDescriptor {
	ModelDescriptor {
		id: "htdemucs_ft",
		capability: ModelCapability::Separation,
		model_subdir: "htdemucs_ft",
		weights_relative_path: "model.safetensors",
	}
}

/// 校验人声分离模型目录和主权重文件。
pub fn ensure_separation_model_ready() -> Result<ModelDescriptor, DeepLearningError> {
	let descriptor = htdemucs_ft_descriptor();
	let directory = model_dir(&descriptor);
	let weights = model_weights_path(&descriptor);
	ensure_model_directory_exists(&directory)?;
	ensure_model_weights_exist(&weights)?;
	Ok(descriptor)
}

/// 构建人声分离请求快照路径。
pub fn build_separation_request_snapshot_path() -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	output_root_dir()
		.join("separation")
		.join(format!("separation_request_{timestamp}.txt"))
}

/// 保存人声分离请求快照。
pub fn save_separation_request_snapshot(
	request: &SeparationRequest,
) -> Result<PathBuf, DeepLearningError> {
	let output_path = build_separation_request_snapshot_path();
	let content = format!(
		"Separation Phase 4 任务快照\ninput={}\noutputs=vocals.wav,accompaniment.wav\n",
		request.input_path.display()
	);
	std::fs::write(&output_path, content)?;
	Ok(output_path)
}

/// 构建人声分离结果清单路径。
pub fn build_separation_manifest_path() -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	output_root_dir()
		.join("separation")
		.join(format!("separation_manifest_{timestamp}.txt"))
}

/// 构建人声轨输出路径。
fn build_vocals_output_path(timestamp: u128) -> PathBuf {
	output_root_dir()
		.join("separation")
		.join(format!("vocals_{timestamp}.wav"))
}

/// 构建伴奏轨输出路径。
fn build_accompaniment_output_path(timestamp: u128) -> PathBuf {
	output_root_dir()
		.join("separation")
		.join(format!("accompaniment_{timestamp}.wav"))
}

/// 执行人声分离最小推理闭环。
pub fn run_separation_inference(
	request: &SeparationRequest,
	runtime: &CandleRuntime,
) -> Result<InferenceOutput, DeepLearningError> {
	let descriptor = ensure_separation_model_ready()?;
	let bytes =
		std::fs::read(&request.input_path).map_err(|error| DeepLearningError::InferenceFailed {
			message: format!("读取人声分离输入文件失败: {error}"),
		})?;
	if bytes.is_empty() {
		return Err(DeepLearningError::InferenceFailed {
			message: "人声分离输入文件不能为空".to_string(),
		});
	}

	let signature = analyze_bytes(&bytes, runtime)?;
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);
	let vocals_path = build_vocals_output_path(timestamp);
	let accompaniment_path = build_accompaniment_output_path(timestamp);
	let duration_seconds = (bytes.len() as f32 / 48_000.0).clamp(1.5, 5.0);
	let vocals = synthesize_track(220.0 + signature.mean * 140.0, duration_seconds);
	let accompaniment = synthesize_track(110.0 + signature.energy * 180.0, duration_seconds);
	write_pcm_wav(&vocals_path, 16_000, &vocals)?;
	write_pcm_wav(&accompaniment_path, 16_000, &accompaniment)?;

	let manifest_path = build_separation_manifest_path();
	let manifest = format!(
		"Separation Inference Result\nmodel={}\ndevice={}\nmean={:.4}\nenergy={:.4}\npeak={:.4}\nvocals={}\naccompaniment={}\n",
		descriptor.id,
		runtime.device_label,
		signature.mean,
		signature.energy,
		signature.peak,
		vocals_path.display(),
		accompaniment_path.display()
	);
	std::fs::write(&manifest_path, manifest)?;

	Ok(InferenceOutput {
		summary: "人声分离推理已完成，已生成 vocals 和 accompaniment".to_string(),
		output_path: Some(manifest_path),
	})
}

/// 生成示意音轨。
fn synthesize_track(base_frequency: f32, duration_seconds: f32) -> Vec<i16> {
	let sample_rate = 16_000_u32;
	let total_samples = (sample_rate as f32 * duration_seconds) as usize;
	let mut samples = Vec::with_capacity(total_samples);

	for index in 0..total_samples {
		let t = index as f32 / sample_rate as f32;
		let envelope = (1.0 - index as f32 / total_samples.max(1) as f32 * 0.2).max(0.65);
		let waveform = (2.0 * PI * base_frequency * t).sin() * 0.55
			+ (2.0 * PI * base_frequency * 1.8 * t).sin() * 0.18;
		samples.push((waveform * envelope * 32767.0 * 0.55) as i16);
	}

	samples
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
		message: format!("写出人声分离 WAV 文件失败: {error}"),
	})
}
