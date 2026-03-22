use std::path::{Path, PathBuf};

use crate::error::DeepLearningError;

/// 深度学习能力类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelCapability {
	/// 本地翻译。
	Translation,

	/// 人声分离。
	Separation,

	/// Whisper 识别。
	Whisper,

	/// 图像生成。
	ImageGeneration,

	/// 语音生成。
	Tts,
}

/// 模型描述结构。
#[derive(Debug, Clone)]
pub struct ModelDescriptor {
	/// 模型唯一标识。
	pub id: &'static str,

	/// 模型所属能力。
	pub capability: ModelCapability,

	/// 模型所在子目录。
	pub model_subdir: &'static str,

	/// 主权重文件相对路径。
	pub weights_relative_path: &'static str,
}

/// 获取项目根目录下的模型根目录。
pub fn model_root_dir() -> PathBuf {
	workspace_root_dir().join("deepl_models")
}

/// 获取模型能力对应的目录名。
pub fn capability_dir_name(capability: ModelCapability) -> &'static str {
	match capability {
		ModelCapability::Translation => "translation",
		ModelCapability::Separation => "separation",
		ModelCapability::Whisper => "whisper",
		ModelCapability::ImageGeneration => "image_generation",
		ModelCapability::Tts => "tts",
	}
}

/// 根据模型描述获取模型目录。
pub fn model_dir(descriptor: &ModelDescriptor) -> PathBuf {
	model_root_dir()
		.join(capability_dir_name(descriptor.capability))
		.join(descriptor.model_subdir)
}

/// 根据模型描述获取主权重文件路径。
pub fn model_weights_path(descriptor: &ModelDescriptor) -> PathBuf {
	model_dir(descriptor).join(descriptor.weights_relative_path)
}

/// 创建所有模型能力目录。
pub fn ensure_model_directories() -> Result<(), DeepLearningError> {
	let root = model_root_dir();
	std::fs::create_dir_all(root.join("translation"))?;
	std::fs::create_dir_all(root.join("separation"))?;
	std::fs::create_dir_all(root.join("whisper"))?;
	std::fs::create_dir_all(root.join("image_generation"))?;
	std::fs::create_dir_all(root.join("tts"))?;
	Ok(())
}

/// 校验模型目录是否存在。
pub fn ensure_model_directory_exists(path: &Path) -> Result<(), DeepLearningError> {
	if path.exists() {
		return Ok(());
	}

	Err(DeepLearningError::ModelDirectoryMissing {
		path: path.to_path_buf(),
	})
}

/// 校验模型主权重文件是否存在。
pub fn ensure_model_weights_exist(path: &Path) -> Result<(), DeepLearningError> {
	if path.exists() {
		return Ok(());
	}

	Err(DeepLearningError::ModelFileMissing {
		path: path.to_path_buf(),
	})
}

/// 定位当前工作区根目录。
///
/// 优先从当前工作目录向上寻找，兼容通过编辑器直接运行 `entry` 的场景；
/// 如果当前目录链路找不到，再回退到可执行文件所在目录向上寻找。
pub fn workspace_root_dir() -> PathBuf {
	if let Ok(current_dir) = std::env::current_dir()
		&& let Some(root) = find_workspace_root_from(&current_dir)
	{
		return root;
	}

	if let Ok(current_exe) = std::env::current_exe()
		&& let Some(exe_dir) = current_exe.parent()
		&& let Some(root) = find_workspace_root_from(exe_dir)
	{
		return root;
	}

	PathBuf::from(".")
}

/// 从指定目录向上查找工作区根目录。
fn find_workspace_root_from(start: &Path) -> Option<PathBuf> {
	for directory in start.ancestors() {
		let cargo_toml = directory.join("Cargo.toml");
		let entry_manifest = directory.join("entry").join("Cargo.toml");
		let deep_learning_manifest = directory.join("deep_learning").join("Cargo.toml");
		if cargo_toml.exists() && entry_manifest.exists() && deep_learning_manifest.exists() {
			return Some(directory.to_path_buf());
		}
	}

	None
}
