use std::{
	fs,
	path::{Path, PathBuf},
	time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::Image;

use crate::{error::ScreenshotError, request::CaptureOutputKind, result::ScreenshotResult};

/// 解析截图输出根目录。
///
/// 开发环境优先回到工作区根目录；
/// 安装版环境统一落到 `LOCALAPPDATA/rust_packages_survey/screenshots/`，
/// 避免继续写到开发目录或 `Program Files`。
fn screenshot_root_dir() -> Result<PathBuf, ScreenshotError> {
	if let Ok(current_dir) = std::env::current_dir()
		&& let Some(root) = find_workspace_root_from(&current_dir)
	{
		return Ok(root);
	}

	if let Ok(current_exe) = std::env::current_exe()
		&& let Some(exe_dir) = current_exe.parent()
	{
		if let Some(root) = find_workspace_root_from(exe_dir) {
			return Ok(root);
		}

		if let Some(local_app_data_dir) = local_app_data_root_dir() {
			return Ok(local_app_data_dir);
		}

		return Ok(exe_dir.to_path_buf());
	}

	let fallback_dir = PathBuf::from(".");
	Err(ScreenshotError::InvalidOutputDirectory(fallback_dir))
}

/// 获取当前用户的本地应用数据目录。
fn local_app_data_root_dir() -> Option<PathBuf> {
	let local_app_data = std::env::var_os("LOCALAPPDATA")?;
	Some(PathBuf::from(local_app_data).join("rust_packages_survey"))
}

/// 从指定目录向上查找工作区根目录。
fn find_workspace_root_from(start: &Path) -> Option<PathBuf> {
	for directory in start.ancestors() {
		let cargo_toml = directory.join("Cargo.toml");
		let entry_manifest = directory.join("entry").join("Cargo.toml");
		let screenshot_manifest = directory.join("screenshot").join("Cargo.toml");
		if cargo_toml.exists() && entry_manifest.exists() && screenshot_manifest.exists() {
			return Some(directory.to_path_buf());
		}
	}

	None
}

pub fn screenshots_dir() -> Result<PathBuf, ScreenshotError> {
	let path = screenshot_root_dir()?.join("screenshots");
	if !path.exists() {
		fs::create_dir_all(&path)?;
	}
	Ok(path)
}

pub fn build_output_path(kind: CaptureOutputKind) -> Result<PathBuf, ScreenshotError> {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_err(std::io::Error::other)?
		.as_millis();
	Ok(screenshots_dir()?.join(format!("{}_{}.png", kind.prefix(), timestamp)))
}

pub fn save_bevy_image(image: &Image, path: &Path) -> Result<ScreenshotResult, ScreenshotError> {
	let dynamic = image
		.clone()
		.try_into_dynamic()
		.map_err(|error| ScreenshotError::DynamicImage(error.to_string()))?;
	let rgb = dynamic.to_rgb8();
	let (width, height) = rgb.dimensions();
	rgb.save(path)?;
	Ok(ScreenshotResult {
		width,
		height,
		path: path.to_path_buf(),
	})
}

pub fn save_dynamic_image(
	image: &image::DynamicImage,
	path: &Path,
) -> Result<ScreenshotResult, ScreenshotError> {
	let rgb = image.to_rgb8();
	let (width, height) = rgb.dimensions();
	rgb.save(path)?;
	Ok(ScreenshotResult {
		width,
		height,
		path: path.to_path_buf(),
	})
}

/// ä¿å­˜ RGBA åŽŸå§‹åƒç´ æ•°æ®åˆ° PNGã€‚
pub fn save_rgba_pixels(
	width: u32,
	height: u32,
	pixels: Vec<u8>,
	path: &Path,
) -> Result<ScreenshotResult, ScreenshotError> {
	let image = image::RgbaImage::from_raw(width, height, pixels)
		.ok_or(ScreenshotError::InvalidImageBuffer { width, height })?;
	let dynamic = image::DynamicImage::ImageRgba8(image);
	save_dynamic_image(&dynamic, path)
}
