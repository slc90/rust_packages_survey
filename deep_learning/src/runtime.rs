use std::path::PathBuf;

use candle_core::{Device, Tensor};

use crate::{
	error::DeepLearningError,
	image_generation::run_image_generation_inference,
	model::{ensure_model_directories, model_root_dir},
	output::ensure_output_directories,
	output::output_root_dir,
	separation::run_separation_inference,
	task::DlTaskPayload,
	translation::run_translation_inference,
	tts::run_tts_inference,
	whisper::run_whisper_inference,
};

/// 深度学习运行时目录信息。
#[derive(Debug, Clone)]
pub struct RuntimeDirectories {
	/// 模型根目录。
	pub model_root: PathBuf,

	/// 输出根目录。
	pub output_root: PathBuf,
}

/// 运行时设备类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDeviceKind {
	/// CUDA 设备。
	Cuda,
}

impl RuntimeDeviceKind {
	/// 获取设备标签。
	pub fn as_label(self) -> &'static str {
		match self {
			Self::Cuda => "cuda",
		}
	}
}

/// Candle 推理运行时。
#[derive(Debug, Clone)]
pub struct CandleRuntime {
	/// 目录信息。
	pub directories: RuntimeDirectories,

	/// 设备类型。
	pub device_kind: RuntimeDeviceKind,

	/// 设备标签文本。
	pub device_label: String,

	device: Device,
}

impl CandleRuntime {
	/// 获取当前运行时设备。
	pub fn device(&self) -> &Device {
		&self.device
	}
}

/// 推理输出摘要。
#[derive(Debug, Clone)]
pub struct InferenceOutput {
	/// 结果摘要。
	pub summary: String,

	/// 主要输出路径。
	pub output_path: Option<PathBuf>,
}

/// 张量统计特征。
#[derive(Debug, Clone, Copy)]
pub struct TensorSignature {
	/// 样本长度。
	pub length: usize,

	/// 平均值。
	pub mean: f32,

	/// 均方能量。
	pub energy: f32,

	/// 峰值。
	pub peak: f32,
}

/// 初始化 Phase 1 所需的运行时目录。
pub fn initialize_runtime_directories() -> Result<RuntimeDirectories, DeepLearningError> {
	ensure_model_directories()?;
	ensure_output_directories()?;

	Ok(RuntimeDirectories {
		model_root: model_root_dir(),
		output_root: output_root_dir(),
	})
}

/// 初始化 Candle 运行时。
pub fn initialize_runtime() -> Result<CandleRuntime, DeepLearningError> {
	let directories = initialize_runtime_directories()?;
	let device =
		Device::cuda_if_available(0).map_err(|error| DeepLearningError::InferenceFailed {
			message: format!("初始化 CUDA 设备失败: {error}"),
		})?;
	let device_kind = RuntimeDeviceKind::Cuda;
	let device_label = "cuda:0".to_string();

	Ok(CandleRuntime {
		directories,
		device_kind,
		device_label,
		device,
	})
}

/// 执行统一推理任务入口。
pub fn execute_task(payload: DlTaskPayload) -> Result<InferenceOutput, DeepLearningError> {
	let runtime = initialize_runtime()?;

	match payload {
		DlTaskPayload::SmokeTest => run_smoke_test(&runtime),
		DlTaskPayload::Translation(request) => run_translation_inference(&request, &runtime),
		DlTaskPayload::Separation(request) => run_separation_inference(&request, &runtime),
		DlTaskPayload::Whisper(request) => run_whisper_inference(&request, &runtime),
		DlTaskPayload::ImageGeneration(request) => {
			run_image_generation_inference(&request, &runtime)
		}
		DlTaskPayload::Tts(request) => run_tts_inference(&request, &runtime),
	}
}

/// 执行运行时冒烟测试。
pub fn run_smoke_test(runtime: &CandleRuntime) -> Result<InferenceOutput, DeepLearningError> {
	let tensor = Tensor::from_vec(vec![1_f32, 2.0, 3.0, 4.0], (2, 2), runtime.device())
		.map_err(|error| map_candle_error("构建冒烟测试张量", error))?;
	let sum = tensor
		.sum_all()
		.map_err(|error| map_candle_error("执行冒烟测试求和", error))?
		.to_scalar::<f32>()
		.map_err(|error| map_candle_error("读取冒烟测试结果", error))?;

	Ok(InferenceOutput {
		summary: format!(
			"Candle 运行时已完成冒烟测试，device={}，sum={sum:.1}",
			runtime.device_label
		),
		output_path: None,
	})
}

/// 对文本进行张量化统计，用于最小推理闭环。
pub fn analyze_text(
	text: &str,
	runtime: &CandleRuntime,
) -> Result<TensorSignature, DeepLearningError> {
	let values = text
		.chars()
		.map(|character| (character as u32 % 1024) as f32 / 1024.0)
		.collect::<Vec<_>>();
	analyze_f32_series(&values, runtime)
}

/// 对字节序列进行张量化统计，用于最小推理闭环。
pub fn analyze_bytes(
	bytes: &[u8],
	runtime: &CandleRuntime,
) -> Result<TensorSignature, DeepLearningError> {
	let values = bytes
		.iter()
		.map(|byte| f32::from(*byte) / 255.0)
		.collect::<Vec<_>>();
	analyze_f32_series(&values, runtime)
}

/// 对连续浮点序列执行基础张量统计。
pub fn analyze_f32_series(
	values: &[f32],
	runtime: &CandleRuntime,
) -> Result<TensorSignature, DeepLearningError> {
	let normalized_values = if values.is_empty() {
		vec![0.0_f32]
	} else {
		values.to_vec()
	};
	let length = normalized_values.len();
	let peak = normalized_values
		.iter()
		.fold(0.0_f32, |current, value| current.max(value.abs()));
	let tensor = Tensor::from_vec(normalized_values, length, runtime.device())
		.map_err(|error| map_candle_error("创建统计张量", error))?;
	let sum = tensor
		.sum_all()
		.map_err(|error| map_candle_error("计算张量和", error))?
		.to_scalar::<f32>()
		.map_err(|error| map_candle_error("读取张量和", error))?;
	let energy = tensor
		.sqr()
		.map_err(|error| map_candle_error("计算张量平方", error))?
		.sum_all()
		.map_err(|error| map_candle_error("计算张量能量", error))?
		.to_scalar::<f32>()
		.map_err(|error| map_candle_error("读取张量能量", error))?;

	Ok(TensorSignature {
		length,
		mean: sum / length as f32,
		energy: energy / length as f32,
		peak,
	})
}

/// 将 Candle 错误映射为项目统一错误。
pub fn map_candle_error(action: &str, error: candle_core::Error) -> DeepLearningError {
	DeepLearningError::InferenceFailed {
		message: format!("{action}失败: {error}"),
	}
}
