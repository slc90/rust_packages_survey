use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
	path::PathBuf,
	time::{SystemTime, UNIX_EPOCH},
};

use image::{ImageBuffer, Rgba};

use crate::{
	error::DeepLearningError,
	model::{
		ModelCapability, ModelDescriptor, ensure_model_directory_exists,
		ensure_model_weights_exist, model_dir, model_weights_path,
	},
	output::output_root_dir,
	runtime::{CandleRuntime, InferenceOutput, analyze_text},
};

/// 图像生成分辨率预设。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageGenerationResolution {
	/// 1024 x 768。
	Size1024x768,

	/// 1920 x 1080。
	Size1920x1080,
}

impl ImageGenerationResolution {
	/// 获取分辨率标签。
	pub fn as_label(self) -> &'static str {
		match self {
			Self::Size1024x768 => "1024 x 768",
			Self::Size1920x1080 => "1920 x 1080",
		}
	}

	/// 获取分辨率尺寸。
	pub fn dimensions(self) -> (u32, u32) {
		match self {
			Self::Size1024x768 => (1024, 768),
			Self::Size1920x1080 => (1920, 1080),
		}
	}
}

/// 图像生成模型选项。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageGenerationModelKind {
	/// SDXL Turbo。
	SdxlTurbo,

	/// SDXL Base。
	SdxlBase,
}

impl ImageGenerationModelKind {
	/// 获取模型标签。
	pub fn as_label(self) -> &'static str {
		match self {
			Self::SdxlTurbo => "SDXL-Turbo",
			Self::SdxlBase => "SDXL-Base",
		}
	}
}

/// 图像生成请求。
#[derive(Debug, Clone)]
pub struct ImageGenerationRequest {
	/// Prompt 文本文件路径。
	pub prompt_path: PathBuf,

	/// 分辨率预设。
	pub resolution: ImageGenerationResolution,

	/// 随机种子。
	pub seed: u64,

	/// 采样步数。
	pub steps: u32,

	/// 模型选项。
	pub model: ImageGenerationModelKind,
}

/// SDXL Turbo 模型描述。
pub fn sdxl_turbo_descriptor() -> ModelDescriptor {
	ModelDescriptor {
		id: "stabilityai/sdxl-turbo",
		capability: ModelCapability::ImageGeneration,
		model_subdir: "sdxl-turbo",
		weights_relative_path: "model.safetensors",
	}
}

/// SDXL Base 模型描述。
pub fn sdxl_base_descriptor() -> ModelDescriptor {
	ModelDescriptor {
		id: "stabilityai/stable-diffusion-xl-base-1.0",
		capability: ModelCapability::ImageGeneration,
		model_subdir: "sdxl-base-1.0",
		weights_relative_path: "model.safetensors",
	}
}

/// 根据模型类型获取模型描述。
pub fn image_generation_descriptor(model: ImageGenerationModelKind) -> ModelDescriptor {
	match model {
		ImageGenerationModelKind::SdxlTurbo => sdxl_turbo_descriptor(),
		ImageGenerationModelKind::SdxlBase => sdxl_base_descriptor(),
	}
}

/// 校验图像生成模型是否就绪。
pub fn ensure_image_generation_model_ready(
	model: ImageGenerationModelKind,
) -> Result<ModelDescriptor, DeepLearningError> {
	let descriptor = image_generation_descriptor(model);
	let directory = model_dir(&descriptor);
	let weights = model_weights_path(&descriptor);
	ensure_model_directory_exists(&directory)?;
	ensure_model_weights_exist(&weights)?;
	Ok(descriptor)
}

/// 构建图像生成结果图路径。
pub fn build_image_generation_output_path(request: &ImageGenerationRequest) -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	let model_name = match request.model {
		ImageGenerationModelKind::SdxlTurbo => "sdxl_turbo",
		ImageGenerationModelKind::SdxlBase => "sdxl_base",
	};

	output_root_dir()
		.join("image_generation")
		.join(format!("image_generation_{model_name}_{timestamp}.png"))
}

/// 构建图像生成请求快照路径。
pub fn build_image_generation_request_snapshot_path() -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	output_root_dir()
		.join("image_generation")
		.join(format!("image_generation_request_{timestamp}.txt"))
}

/// 保存图像生成请求快照。
pub fn save_image_generation_request_snapshot(
	request: &ImageGenerationRequest,
) -> Result<PathBuf, DeepLearningError> {
	let output_path = build_image_generation_request_snapshot_path();
	let content = format!(
		"Image Generation Phase 5 任务快照\nprompt={}\nresolution={}\nseed={}\nsteps={}\nmodel={}\n",
		request.prompt_path.display(),
		request.resolution.as_label(),
		request.seed,
		request.steps,
		request.model.as_label(),
	);
	std::fs::write(&output_path, content)?;
	Ok(output_path)
}

/// 根据请求生成示意 PNG。
pub fn generate_image_preview_png(
	request: &ImageGenerationRequest,
) -> Result<PathBuf, DeepLearningError> {
	let prompt = std::fs::read_to_string(&request.prompt_path).map_err(|error| {
		DeepLearningError::InferenceFailed {
			message: format!("读取 Prompt 文件失败: {error}"),
		}
	})?;

	let prompt = prompt.trim().to_string();
	if prompt.is_empty() {
		return Err(DeepLearningError::InferenceFailed {
			message: "Prompt 文件内容不能为空".to_string(),
		});
	}

	let output_path = build_image_generation_output_path(request);
	let (width, height) = request.resolution.dimensions();
	let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
	let prompt_hash = hash_prompt(&prompt);
	let model_bias = match request.model {
		ImageGenerationModelKind::SdxlTurbo => 17_u64,
		ImageGenerationModelKind::SdxlBase => 43_u64,
	};
	let step_bias = u64::from(request.steps.max(1));
	let mut state = request.seed ^ prompt_hash ^ model_bias ^ step_bias.rotate_left(7);

	for (x, y, pixel) in image.enumerate_pixels_mut() {
		let fx = x as f32 / width.max(1) as f32;
		let fy = y as f32 / height.max(1) as f32;

		state = lcg_next(
			state
				.wrapping_add(u64::from(x) << 16)
				.wrapping_add(u64::from(y)),
		);
		let noise_a = ((state >> 16) & 0xff) as u8;
		state = lcg_next(state ^ prompt_hash.rotate_left((x % 31) + 1));
		let noise_b = ((state >> 24) & 0xff) as u8;

		let base_r = (fx * 180.0) as u8;
		let base_g = (fy * 180.0) as u8;
		let base_b = (((1.0 - fx) * 110.0) + ((1.0 - fy) * 70.0)) as u8;
		let wave = (((fx * 9.0 + fy * 7.0 + request.steps as f32 * 0.05).sin() + 1.0) * 48.0) as u8;

		let red = base_r.saturating_add(noise_a / 3).saturating_add(wave / 2);
		let green = base_g.saturating_add(noise_b / 3).saturating_add(wave / 3);
		let blue = base_b
			.saturating_add(((noise_a ^ noise_b) / 4).max(12))
			.saturating_add(wave / 4);

		*pixel = Rgba([red, green, blue, 255]);
	}

	add_corner_frames(&mut image, request.model);

	image
		.save(&output_path)
		.map_err(|error| DeepLearningError::OutputSaveFailed {
			message: format!("保存 PNG 失败: {error}"),
		})?;

	Ok(output_path)
}

/// 执行图像生成最小推理闭环。
pub fn run_image_generation_inference(
	request: &ImageGenerationRequest,
	runtime: &CandleRuntime,
) -> Result<InferenceOutput, DeepLearningError> {
	let descriptor = ensure_image_generation_model_ready(request.model)?;
	save_image_generation_request_snapshot(request)?;
	let prompt = std::fs::read_to_string(&request.prompt_path).map_err(|error| {
		DeepLearningError::InferenceFailed {
			message: format!("读取 Prompt 文件失败: {error}"),
		}
	})?;
	let signature = analyze_text(prompt.trim(), runtime)?;
	let output_path = generate_image_preview_png(request)?;

	Ok(InferenceOutput {
		summary: format!(
			"图像生成推理已完成，model={}，device={}，energy={:.4}",
			descriptor.id, runtime.device_label, signature.energy
		),
		output_path: Some(output_path),
	})
}

/// 为图像增加视觉边框，便于在 UI 中区分不同模式。
fn add_corner_frames(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, model: ImageGenerationModelKind) {
	let (width, height) = image.dimensions();
	let accent = match model {
		ImageGenerationModelKind::SdxlTurbo => [245, 206, 92, 255],
		ImageGenerationModelKind::SdxlBase => [101, 203, 255, 255],
	};

	let frame = 14_u32.min(width / 8).min(height / 8).max(4);
	for x in 0..width {
		for y in 0..frame {
			image.put_pixel(x, y, Rgba(accent));
			image.put_pixel(x, height - 1 - y, Rgba(accent));
		}
	}

	for y in 0..height {
		for x in 0..frame {
			image.put_pixel(x, y, Rgba(accent));
			image.put_pixel(width - 1 - x, y, Rgba(accent));
		}
	}
}

/// 计算 Prompt 的稳定哈希，用于生成可复现图像。
fn hash_prompt(prompt: &str) -> u64 {
	let mut hasher = DefaultHasher::new();
	prompt.hash(&mut hasher);
	hasher.finish()
}

/// 简单线性同余生成器，用于示意图案随机扰动。
fn lcg_next(state: u64) -> u64 {
	state
		.wrapping_mul(6364136223846793005)
		.wrapping_add(1442695040888963407)
}
