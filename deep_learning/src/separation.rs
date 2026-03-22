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
