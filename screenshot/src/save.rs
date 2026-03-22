use std::{
	fs,
	path::{Path, PathBuf},
	time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::Image;

use crate::{error::ScreenshotError, request::CaptureOutputKind, result::ScreenshotResult};

fn workspace_root() -> Result<PathBuf, ScreenshotError> {
	let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	manifest_dir
		.parent()
		.map(Path::to_path_buf)
		.ok_or(ScreenshotError::InvalidOutputDirectory(manifest_dir))
}

pub fn screenshots_dir() -> Result<PathBuf, ScreenshotError> {
	let path = workspace_root()?.join("screenshots");
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
